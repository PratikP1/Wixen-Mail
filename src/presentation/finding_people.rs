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
use crate::presentation::wx_compose::FindingPeople;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Build what the compose window uses to find people to write to.
///
/// `account_ids` is in the same order as the names in the From list, because
/// that list is what somebody picks from and its position is all the window
/// knows. Both come out of one read of the accounts, so they cannot describe
/// two different lists.
pub fn through(account_ids: Vec<String>, runtime: &Arc<Runtime>) -> FindingPeople {
    let (answers_go_to, answers) = async_channel::unbounded::<looking::WhoWasFound>();
    let runtime = runtime.clone();

    FindingPeople {
        answers,
        start: Box::new(move |asked| {
            let account_id = asked
                .from_account
                .and_then(|position| account_ids.get(position as usize))
                .cloned();
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
    let from_your_contacts = the_contacts_here(account_id, &asked.name, &mut trouble);
    let from_the_directory = the_organisation(account_id, &asked.name, handle, &mut trouble);

    looking::WhoWasFound {
        search: asked.search,
        name: asked.name.clone(),
        everybody: looking::everybody_found(from_your_contacts, from_the_directory),
        trouble: match trouble.is_empty() {
            true => None,
            false => Some(trouble.join(" ")),
        },
    }
}

/// The people in the address book on this computer who match.
///
/// The worker opens the cache itself rather than carrying the window's, which
/// is how every other worker here reaches it: a database handle belongs to the
/// thread that opened it.
fn the_contacts_here(
    account_id: &str,
    name: &str,
    trouble: &mut Vec<String>,
) -> Vec<looking::Somebody> {
    let Some(dir) = AppPaths::resolve().ok().map(|paths| paths.cache_dir()) else {
        trouble.push("There is nowhere on this computer to keep contacts yet.".to_string());
        return Vec::new();
    };
    let cache = match crate::data::message_cache::MessageCache::new(dir, None) {
        Ok(cache) => cache,
        Err(why) => {
            tracing::warn!("The contacts on this computer could not be opened: {why}");
            trouble.push("Your contacts on this computer could not be read.".to_string());
            return Vec::new();
        }
    };
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

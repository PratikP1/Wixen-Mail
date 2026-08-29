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

    // The password is deliberately not fetched. Nothing in this application
    // stores one for a directory yet, and a sign-in with a name and an empty
    // password is one many directory servers accept and treat as anonymous, so
    // `service::directory` refuses that outright and says so. An account whose
    // directory needs a sign-in therefore gets a sentence rather than a search
    // that quietly returns less than the directory holds.
    let asked = handle.block_on(crate::service::directory::look_up(
        Some(directory),
        None,
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

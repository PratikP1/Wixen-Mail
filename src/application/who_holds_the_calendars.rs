//! Whether an account's calendars, contacts and tasks may be asked of Google,
//! and when they may not, why (#22, REAL-01).
//!
//! Each of the three syncs used to ask itself whether this copy held a Google
//! sign-in key, and when it did not, skipped Google without a line in the log,
//! a sentence or an error, and then said "0 created, 0 updated, 0 deleted",
//! which sounds like a sync that found nothing. A Gmail account on an app
//! password met the same silence for a different reason, and an account whose
//! mail is not at Google asked Google anyway whenever a key was present.
//!
//! So the question is answered here, once, for all three: ask Google; not
//! Google's to ask, because the account's mail is somewhere else; or nothing
//! asked, with the reason. The reasons are said in the order a person can act
//! on them (D-06): no account open, no key in this copy, an account on an app
//! password, and a browser sign-in that is missing or has run out, which is
//! only learned by asking for a token.

use crate::application::summing_up::SummingUp;
use crate::application::who_runs_the_mail::WhoRunsTheMail;
use crate::common::{Error, Result};
use crate::data::account::Account;

/// The name Google's sign-in key and an account's Google tokens are filed
/// under, as [`crate::application::who_runs_the_mail::WhoRunsTheMail`] names
/// it for a Gmail account.
const GOOGLE: &str = "gmail";

/// The part of the program whose sync is asking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    Calendar,
    Contacts,
    Tasks,
}

impl Module {
    /// What Google holds for this module, as a person says it.
    fn what_google_holds(self) -> &'static str {
        match self {
            Module::Calendar => "calendars",
            Module::Contacts => "contacts",
            Module::Tasks => "tasks",
        }
    }

    /// The module as one word in the log.
    pub fn word(self) -> &'static str {
        match self {
            Module::Calendar => "calendar",
            Module::Contacts => "contacts",
            Module::Tasks => "tasks",
        }
    }
}

/// Why nothing was asked of Google, in the order the reasons are said when
/// more than one holds: each later one cannot be acted on until the earlier
/// is settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WhyNothingWasAsked {
    NoAccountIsOpen,
    NoGoogleSignInKey,
    SignsInWithAnAppPassword,
    TheBrowserSignInRanOut,
}

impl WhyNothingWasAsked {
    /// The reason as one word in the log, with nothing in it a reader of the
    /// log has to parse around.
    pub fn word(self) -> &'static str {
        match self {
            WhyNothingWasAsked::NoAccountIsOpen => "no_account_open",
            WhyNothingWasAsked::NoGoogleSignInKey => "no_google_sign_in_key",
            WhyNothingWasAsked::SignsInWithAnAppPassword => "app_password",
            WhyNothingWasAsked::TheBrowserSignInRanOut => "browser_sign_in_ran_out",
        }
    }

    /// The reason in a sentence about one module, for the status bar and the
    /// screen reader.
    ///
    /// Each one names what to do, or that nothing here can be done yet: the
    /// separate browser sign-in an app-password account needs arrives with
    /// 14-03, and a sentence naming a control that does not exist would be
    /// untrue until then. "Google sign-in key" is Pratik's word for what
    /// Google's console calls a client.
    pub fn sentence(self, module: Module) -> String {
        let holds = module.what_google_holds();
        match self {
            WhyNothingWasAsked::NoAccountIsOpen => {
                format!("Nothing was asked of Google for {holds}, because no account is open.")
            }
            WhyNothingWasAsked::NoGoogleSignInKey => format!(
                "Nothing was asked of Google for this account's {holds}, because this copy of \
                 Wixen Mail has no Google sign-in key. See Setting up a provider in Help."
            ),
            WhyNothingWasAsked::SignsInWithAnAppPassword => format!(
                "Nothing was asked of Google for this account's {holds}. The account signs in \
                 with an app password, and Google gives {holds} only to a browser sign-in."
            ),
            WhyNothingWasAsked::TheBrowserSignInRanOut => format!(
                "Nothing was asked of Google for this account's {holds}, because its browser \
                 sign-in is missing or has run out. Open the Account Manager with Ctrl+Shift+A \
                 and choose Sign In Again."
            ),
        }
    }
}

/// Whether Google may be asked for this account's calendars, contacts and
/// tasks, carrying the key when it may.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MayGoogleBeAsked<Key> {
    Yes(Key),
    /// The account's mail is not at Google, so Google holds nothing of it.
    NotGooglesToAsk,
    No(WhyNothingWasAsked),
}

/// The answer from what is known before anything is asked: who runs the
/// account's mail, whether it signs in through a browser, and the key this
/// copy holds, if any.
pub fn may_google_be_asked<Key>(account: &Account, key: Option<Key>) -> MayGoogleBeAsked<Key> {
    if WhoRunsTheMail::of(account) != WhoRunsTheMail::Gmail {
        return MayGoogleBeAsked::NotGooglesToAsk;
    }
    match key {
        None => MayGoogleBeAsked::No(WhyNothingWasAsked::NoGoogleSignInKey),
        Some(_) if !account.use_oauth => {
            MayGoogleBeAsked::No(WhyNothingWasAsked::SignsInWithAnAppPassword)
        }
        Some(key) => MayGoogleBeAsked::Yes(key),
    }
}

/// What the three syncs get when they ask for a Google token.
///
/// No `Debug`, because it carries a token and `Debug` is what a log line
/// prints.
pub enum GooglesAnswer {
    Token(String),
    NotGooglesToAsk,
    NothingAsked(WhyNothingWasAsked),
}

/// A Google token for this account's calendars, contacts and tasks, or why
/// there is none.
///
/// Thin glue over [`may_google_be_asked`]: the decisions are that answer's.
/// An error is a token request that failed for a reason other than the
/// sign-in, such as the network, and is the sync's to report.
pub async fn a_google_token(account: Option<&Account>, module: Module) -> Result<GooglesAnswer> {
    let answer = asked_of_google(account).await?;
    // One line per sync that asked nothing, so the log of a Refresh says why
    // nothing came. The module and the reason as words: never an address, a
    // token or a sentence somebody typed.
    if let GooglesAnswer::NothingAsked(why) = &answer {
        tracing::info!("Google was not asked for {}: {}", module.word(), why.word());
    }
    Ok(answer)
}

/// The token, or why there is none, without the log line.
async fn asked_of_google(account: Option<&Account>) -> Result<GooglesAnswer> {
    let Some(account) = account else {
        return Ok(GooglesAnswer::NothingAsked(
            WhyNothingWasAsked::NoAccountIsOpen,
        ));
    };
    let key = crate::service::oauth_credentials::credentials_for(GOOGLE);
    let key = match may_google_be_asked(account, key) {
        MayGoogleBeAsked::Yes(key) => key,
        MayGoogleBeAsked::NotGooglesToAsk => return Ok(GooglesAnswer::NotGooglesToAsk),
        MayGoogleBeAsked::No(why) => return Ok(GooglesAnswer::NothingAsked(why)),
    };
    let sign_in = crate::service::oauth::AuthManager::new(
        &account.id,
        GOOGLE,
        &key.client_id,
        key.client_secret.as_deref(),
    );
    match sign_in.get_valid_token().await {
        Ok(token) => Ok(GooglesAnswer::Token(token)),
        // No token stored, none that can be read, or a refresh Google
        // refused: each is answered by signing in again. The network failing
        // is not, and stays the sync's error to count.
        Err(Error::Authentication(_)) => Ok(GooglesAnswer::NothingAsked(
            WhyNothingWasAsked::TheBrowserSignInRanOut,
        )),
        Err(other) => Err(other),
    }
}

/// Whether a sync asked anybody, and why it did not ask Google, carried on
/// each module's result so its summary can say it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WhatWasAsked {
    /// A pass asked somebody: Google, Microsoft, a server or a feed.
    pub somebody: bool,
    /// Why Google was not asked, when it was not.
    pub why_not_google: Option<WhyNothingWasAsked>,
}

impl WhatWasAsked {
    /// The running total with one pass's result folded in. A result that came
    /// back is somebody having been asked; the first reason held is kept.
    pub fn with_a_pass(self, pass: WhatWasAsked) -> Self {
        Self {
            somebody: true,
            why_not_google: self.why_not_google.or(pass.why_not_google),
        }
    }

    /// The reason to say on its own: nobody at all was asked and nothing
    /// went wrong, so there are no counts worth hearing (D-08).
    fn alone(self, nothing_went_wrong: bool) -> Option<WhyNothingWasAsked> {
        self.why_not_google
            .filter(|_| !self.somebody && nothing_went_wrong)
    }

    /// How a summary opens: its counts, or the reason alone when nobody at
    /// all was asked and nothing went wrong.
    pub fn opening(
        self,
        module: Module,
        counts: impl Into<String>,
        nothing_went_wrong: bool,
    ) -> SummingUp {
        match self.alone(nothing_went_wrong) {
            Some(why) => SummingUp::opening_sentence(why.sentence(module)),
            None => SummingUp::opening(counts),
        }
    }

    /// The reason as a sentence after the counts, when somebody else was
    /// asked or something went wrong; nothing when it was said alone.
    pub fn after_the_counts(self, module: Module, nothing_went_wrong: bool) -> Option<String> {
        match self.alone(nothing_went_wrong) {
            Some(_) => None,
            None => self.why_not_google.map(|why| why.sentence(module)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::calendar::{CalendarSyncResult, what_the_calendar_sync_did};
    use crate::application::status_sentences::{Voice, reads_as_a_persons_sentence};

    const EVERY_MODULE: [Module; 3] = [Module::Calendar, Module::Contacts, Module::Tasks];
    const EVERY_REASON: [WhyNothingWasAsked; 4] = [
        WhyNothingWasAsked::NoAccountIsOpen,
        WhyNothingWasAsked::NoGoogleSignInKey,
        WhyNothingWasAsked::SignsInWithAnAppPassword,
        WhyNothingWasAsked::TheBrowserSignInRanOut,
    ];

    /// The key a copy may hold, as the answer sees it: there or not.
    const A_KEY: Option<&str> = Some("a key");
    const NO_KEY: Option<&str> = None;

    fn a_gmail_account(signs_in_through_a_browser: bool) -> Account {
        Account {
            id: "a1".into(),
            name: "Home".into(),
            email: "me@gmail.com".into(),
            imap_server: "imap.gmail.com".into(),
            use_oauth: signs_in_through_a_browser,
            ..Account::default()
        }
    }

    fn an_account_at(server: &str, address: &str) -> Account {
        Account {
            email: address.into(),
            imap_server: server.into(),
            use_oauth: true,
            ..Account::default()
        }
    }

    #[test]
    fn test_a_gmail_account_asks_google_only_with_a_key_and_a_browser_sign_in() {
        // The order of D-06: no key first, whatever the sign-in, because a
        // browser sign-in cannot run without one.
        assert_eq!(
            may_google_be_asked(&a_gmail_account(true), NO_KEY),
            MayGoogleBeAsked::No(WhyNothingWasAsked::NoGoogleSignInKey)
        );
        assert_eq!(
            may_google_be_asked(&a_gmail_account(false), NO_KEY),
            MayGoogleBeAsked::No(WhyNothingWasAsked::NoGoogleSignInKey)
        );
        // Pratik's account on 2026-10-04: a key, and an app password, which
        // Google takes for mail and never for calendars, contacts or tasks.
        assert_eq!(
            may_google_be_asked(&a_gmail_account(false), A_KEY),
            MayGoogleBeAsked::No(WhyNothingWasAsked::SignsInWithAnAppPassword)
        );
        assert_eq!(
            may_google_be_asked(&a_gmail_account(true), A_KEY),
            MayGoogleBeAsked::Yes("a key")
        );
    }

    #[test]
    fn test_an_account_whose_mail_is_not_at_google_never_asks_google() {
        // Whatever keys this copy holds: until 14-01 the syncs asked Google
        // for any account the moment a Google key was present.
        for account in [
            an_account_at("outlook.office365.com", "me@outlook.com"),
            an_account_at("imap.example.com", "me@example.com"),
        ] {
            for key in [A_KEY, NO_KEY] {
                assert_eq!(
                    may_google_be_asked(&account, key),
                    MayGoogleBeAsked::NotGooglesToAsk,
                    "{} asked Google with key {key:?}",
                    account.email
                );
            }
        }
        // A Workspace account on its own domain is Google's by its server.
        assert_eq!(
            may_google_be_asked(&an_account_at("imap.gmail.com", "me@mycompany.com"), A_KEY),
            MayGoogleBeAsked::Yes("a key")
        );
    }

    #[test]
    fn test_every_reason_is_a_persons_sentence_naming_what_google_holds() {
        for module in EVERY_MODULE {
            for why in EVERY_REASON {
                let said = why.sentence(module);
                assert_eq!(
                    reads_as_a_persons_sentence(&said, Voice::Answer),
                    Ok(()),
                    "{why:?} for {module:?}"
                );
                assert!(
                    said.contains(module.what_google_holds()),
                    "{why:?} does not name the {module:?}: {said}"
                );
                assert!(said.contains("Google"), "Google is not named: {said}");
            }
        }
    }

    #[test]
    fn test_each_reason_names_its_way_out() {
        let module = Module::Calendar;
        let no_key = WhyNothingWasAsked::NoGoogleSignInKey.sentence(module);
        assert!(no_key.contains("Google sign-in key"), "{no_key}");
        assert!(no_key.contains("Setting up a provider in Help"), "{no_key}");
        let password = WhyNothingWasAsked::SignsInWithAnAppPassword.sentence(module);
        assert!(password.contains("app password"), "{password}");
        assert!(password.contains("browser sign-in"), "{password}");
        let ran_out = WhyNothingWasAsked::TheBrowserSignInRanOut.sentence(module);
        assert!(ran_out.contains("Ctrl+Shift+A"), "{ran_out}");
        assert!(ran_out.contains("Sign In Again"), "{ran_out}");
        let no_account = WhyNothingWasAsked::NoAccountIsOpen.sentence(module);
        assert!(no_account.contains("no account is open"), "{no_account}");
        for said in [no_key, password, ran_out, no_account] {
            assert!(
                !said.to_lowercase().contains("oauth"),
                "jargon survives: {said}"
            );
        }
    }

    #[test]
    fn test_the_logs_words_are_single_words_and_distinct() {
        let mut words: Vec<&str> = EVERY_REASON.iter().map(|why| why.word()).collect();
        words.extend(EVERY_MODULE.iter().map(|module| module.word()));
        for word in &words {
            assert!(!word.is_empty(), "a word is empty: {words:?}");
            assert!(
                word.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "{word} has something in it a reader of the log parses around"
            );
        }
        let mut distinct = words.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), words.len(), "two words are one: {words:?}");
    }

    #[test]
    fn test_the_calendar_summary_says_the_reason_alone_when_nobody_was_asked() {
        // #22: Pratik's Refresh said "Calendar sync: 0 created, 0 updated, 0
        // deleted", with the success cue, for a sync that asked nobody.
        let why = WhyNothingWasAsked::NoGoogleSignInKey;
        let said = what_the_calendar_sync_did(&CalendarSyncResult {
            what_was_asked: WhatWasAsked {
                somebody: false,
                why_not_google: Some(why),
            },
            ..CalendarSyncResult::default()
        });
        assert_eq!(said, why.sentence(Module::Calendar));
        assert!(!said.contains("0 created"), "{said}");
    }

    #[test]
    fn test_the_calendar_summary_says_the_reason_after_the_counts_when_a_server_was_asked() {
        // D-08: a calendar server's pass ran, so its counts are said and the
        // reason Google was not asked follows them.
        let why = WhyNothingWasAsked::SignsInWithAnAppPassword;
        let mut total = CalendarSyncResult {
            what_was_asked: WhatWasAsked {
                somebody: false,
                why_not_google: Some(why),
            },
            ..CalendarSyncResult::default()
        };
        total.absorb(CalendarSyncResult {
            created: 2,
            ..CalendarSyncResult::default()
        });
        let said = what_the_calendar_sync_did(&total);
        assert_eq!(
            said,
            format!(
                "Calendar sync: 2 created, 0 updated, 0 deleted. {}",
                why.sentence(Module::Calendar)
            )
        );

        // And a pass that failed was a pass that asked: its error is counted
        // and the reason still follows.
        let failed = CalendarSyncResult {
            what_was_asked: WhatWasAsked {
                somebody: false,
                why_not_google: Some(why),
            },
            errors: vec!["the server said no".to_string()],
            ..CalendarSyncResult::default()
        };
        let said = what_the_calendar_sync_did(&failed);
        assert!(said.starts_with("Calendar sync: 0 created"), "{said}");
        assert!(said.contains("1 error"), "{said}");
        assert!(said.ends_with(&why.sentence(Module::Calendar)), "{said}");
    }

    #[test]
    fn test_a_calendar_sync_that_asked_google_says_no_reason() {
        let mut total = CalendarSyncResult::default();
        total.absorb(CalendarSyncResult {
            created: 1,
            ..CalendarSyncResult::default()
        });
        assert_eq!(
            what_the_calendar_sync_did(&total),
            "Calendar sync: 1 created, 0 updated, 0 deleted"
        );
    }

    #[test]
    fn test_an_item_held_for_a_choice_is_counted_once_when_two_passes_are_folded() {
        // The window added a pass's held count twice, for Google and for
        // Microsoft alike, so one held event was said as two. Folded through
        // `absorb` it is counted once. Here rather than beside `absorb` in
        // calendar.rs, whose test count eighty guard records fingerprint.
        let mut total = CalendarSyncResult::default();
        total.absorb(CalendarSyncResult {
            held_for_you_to_choose: 1,
            ..CalendarSyncResult::default()
        });
        total.absorb(CalendarSyncResult::default());
        assert_eq!(total.held_for_you_to_choose, 1);
    }

    #[test]
    fn test_folding_two_calendar_passes_keeps_every_count() {
        // Compared whole, so a count added later cannot be dropped between a
        // pass and the window without this going red.
        let mut total = CalendarSyncResult {
            what_was_asked: WhatWasAsked {
                somebody: false,
                why_not_google: Some(WhyNothingWasAsked::NoGoogleSignInKey),
            },
            ..CalendarSyncResult::default()
        };
        total.absorb(CalendarSyncResult {
            created: 1,
            updated: 2,
            deleted: 3,
            sent: 4,
            waiting_on_the_setting: 5,
            changes_that_cannot_be_saved: vec!["one".to_string()],
            days_that_may_be_shown_twice: 6,
            held_for_you_to_choose: 7,
            what_was_asked: WhatWasAsked::default(),
            errors: vec!["first".to_string()],
        });
        total.absorb(CalendarSyncResult {
            created: 10,
            updated: 20,
            deleted: 30,
            sent: 40,
            waiting_on_the_setting: 50,
            changes_that_cannot_be_saved: vec!["two".to_string()],
            days_that_may_be_shown_twice: 60,
            held_for_you_to_choose: 70,
            what_was_asked: WhatWasAsked {
                somebody: false,
                why_not_google: Some(WhyNothingWasAsked::TheBrowserSignInRanOut),
            },
            errors: vec!["second".to_string()],
        });
        assert_eq!(
            total,
            CalendarSyncResult {
                created: 11,
                updated: 22,
                deleted: 33,
                sent: 44,
                waiting_on_the_setting: 55,
                changes_that_cannot_be_saved: vec!["one".to_string(), "two".to_string()],
                days_that_may_be_shown_twice: 66,
                held_for_you_to_choose: 77,
                what_was_asked: WhatWasAsked {
                    somebody: true,
                    why_not_google: Some(WhyNothingWasAsked::NoGoogleSignInKey),
                },
                errors: vec!["first".to_string(), "second".to_string()],
            }
        );
    }

    /// The text of one arm of the window's update handler, from its heading
    /// to the arm after it.
    fn the_windows_arm(heading: &str, next: &str) -> String {
        let path = "src/presentation/wx_app.rs";
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let (_, after) = source
            .split_once(heading)
            .unwrap_or_else(|| panic!("{heading} is not in {path}, so this reads nothing"));
        let (arm, _) = after
            .split_once(next)
            .unwrap_or_else(|| panic!("{next} no longer follows {heading} in {path}"));
        arm.to_string()
    }

    #[test]
    fn test_a_calendar_sync_that_did_not_ask_google_is_signalled_as_needing_attention() {
        // D-07: the success cue for a sync that did nothing is the defect #22
        // reports. The sentence is the answer to the key somebody pressed, so
        // it is said whatever level was chosen while fetching.
        let arm = the_windows_arm(
            "UIUpdate::CalendarSyncComplete(result) => {",
            "UIUpdate::ModuleChanged(",
        );
        assert!(
            arm.contains("what_was_asked.why_not_google.is_some()"),
            "the calendar's finish does not look for the reason: {arm}"
        );
        assert!(
            arm.contains("a11y.signal(FeedbackEvent::AccountNeedsAttention, &msg)"),
            "a calendar sync that asked Google nothing is not signalled as needing attention, \
             whole: {arm}"
        );
        assert!(
            arm.contains("a11y.signal(FeedbackEvent::SyncComplete, detail)"),
            "a calendar sync that ran no longer finishes as one: {arm}"
        );
    }
}

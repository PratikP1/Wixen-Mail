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
//!
//! Since 14-03 an account on an app password is asked with a browser sign-in
//! of its own for these three alone (route B), which the reasons name.
//!
//! Since 14-02 the same is answered for Microsoft, which every sync asked
//! whenever this copy held a Microsoft key, whatever the account; and an
//! account at neither provider, with no calendar server, feed or address book
//! of its own, hears why there is nothing to bring. Each finished sync writes
//! one line built by [`the_finish_line`].

use crate::application::status_sentences::{Thing, nothing_chosen};
use crate::application::summing_up::SummingUp;
use crate::application::who_runs_the_mail::WhoRunsTheMail;
use crate::common::types::PimModule;
use crate::common::{Error, Result};
use crate::data::account::Account;
use crate::service::oauth;

/// The name Google's sign-in key and an account's Google tokens are filed
/// under, as [`crate::application::who_runs_the_mail::WhoRunsTheMail`] names
/// it for a Gmail account.
const GOOGLE: &str = "gmail";

/// The name Microsoft's sign-in key is filed under, as
/// [`crate::application::who_runs_the_mail::WhoRunsTheMail`] names it for a
/// Microsoft account.
const MICROSOFT: &str = "outlook";

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
    TheSeparateSignInRanOut,
    NoMicrosoftSignInKey,
    AtNeitherProvider,
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
            WhyNothingWasAsked::TheSeparateSignInRanOut => "separate_sign_in_ran_out",
            WhyNothingWasAsked::NoMicrosoftSignInKey => "no_microsoft_sign_in_key",
            WhyNothingWasAsked::AtNeitherProvider => "at_neither_provider",
        }
    }

    /// The reason in a sentence about one module, for the status bar and the
    /// screen reader.
    ///
    /// Each one names what to do. An account on an app password, with no
    /// separate browser sign-in or one that has run out, is sent to the
    /// Account Manager's Sign In for Calendars, Contacts and Tasks, which
    /// 14-03 built; until then the sentence said only why. "Google sign-in
    /// key" is Pratik's word for what Google's console calls a client.
    pub fn sentence(self, module: Module) -> String {
        self.sentence_about(module.what_google_holds())
    }

    /// The reason in one sentence for all three modules, said once when an
    /// account is added and none of them can be brought (14-02 choice 3).
    pub fn sentence_for_every_module(self) -> String {
        self.sentence_about(EVERY_MODULE_HOLDS)
    }

    /// The reason in a sentence about what a provider holds, as a person
    /// says it.
    fn sentence_about(self, holds: &str) -> String {
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
                 to mail with an app password, and Google gives {holds} only to a browser \
                 sign-in. {THE_WAY_TO_THE_SEPARATE_SIGN_IN}"
            ),
            WhyNothingWasAsked::TheBrowserSignInRanOut => format!(
                "Nothing was asked of Google for this account's {holds}, because its browser \
                 sign-in is missing or has run out. Open the Account Manager with Ctrl+Shift+A \
                 and choose Sign In Again."
            ),
            WhyNothingWasAsked::TheSeparateSignInRanOut => format!(
                "Nothing was asked of Google for this account's {holds}, because the browser \
                 sign-in it keeps for them has run out or no longer works. \
                 {THE_WAY_TO_THE_SEPARATE_SIGN_IN}"
            ),
            WhyNothingWasAsked::NoMicrosoftSignInKey => format!(
                "Nothing was asked of Microsoft for this account's {holds}, because this copy of \
                 Wixen Mail has no Microsoft sign-in key. See Setting up a provider in Help."
            ),
            WhyNothingWasAsked::AtNeitherProvider => format!(
                "This account's mail is at neither Google nor Microsoft, so there are no {holds} \
                 there to bring."
            ),
        }
    }

    /// The sign-in that has run out when a token is refused: the mail's own
    /// for an account whose mail signs in through the browser, the separate
    /// one for an account on an app password. Each is made again by a
    /// different control.
    fn the_sign_in_that_ran_out(account: &Account) -> Self {
        match account.use_oauth {
            true => WhyNothingWasAsked::TheBrowserSignInRanOut,
            false => WhyNothingWasAsked::TheSeparateSignInRanOut,
        }
    }
}

/// What a provider holds for all three modules, as a person says it.
const EVERY_MODULE_HOLDS: &str = "calendars, contacts and tasks";

/// Where the separate browser sign-in for calendars, contacts and tasks is
/// made, as the reasons that need it say it.
const THE_WAY_TO_THE_SEPARATE_SIGN_IN: &str = "Open the Account Manager with Ctrl+Shift+A, \
     choose the account and press Sign In for Calendars, Contacts and Tasks.";

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
pub fn may_google_be_asked<Key>(
    account: &Account,
    key: Option<Key>,
    holds_the_separate_sign_in: bool,
) -> MayGoogleBeAsked<Key> {
    if WhoRunsTheMail::of(account) != WhoRunsTheMail::Gmail {
        return MayGoogleBeAsked::NotGooglesToAsk;
    }
    match key {
        None => MayGoogleBeAsked::No(WhyNothingWasAsked::NoGoogleSignInKey),
        // Mail on an app password and no separate browser sign-in made yet
        // for the calendars, contacts and tasks (route B).
        Some(_) if !account.use_oauth && !holds_the_separate_sign_in => {
            MayGoogleBeAsked::No(WhyNothingWasAsked::SignsInWithAnAppPassword)
        }
        Some(key) => MayGoogleBeAsked::Yes(key),
    }
}

/// Whether Microsoft may be asked for this account's calendars, contacts and
/// tasks, carrying the key when it may.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MayMicrosoftBeAsked<Key> {
    Yes(Key),
    /// The account's mail is not at Microsoft, so Microsoft holds nothing of
    /// it.
    NotMicrosoftsToAsk,
    No(WhyNothingWasAsked),
}

/// The answer from who runs the account's mail and the key this copy holds.
pub fn may_microsoft_be_asked<Key>(
    account: &Account,
    key: Option<Key>,
) -> MayMicrosoftBeAsked<Key> {
    if WhoRunsTheMail::of(account) != WhoRunsTheMail::Microsoft {
        return MayMicrosoftBeAsked::NotMicrosoftsToAsk;
    }
    match key {
        None => MayMicrosoftBeAsked::No(WhyNothingWasAsked::NoMicrosoftSignInKey),
        Some(key) => MayMicrosoftBeAsked::Yes(key),
    }
}

/// The key this copy holds for asking Microsoft about this account, or why
/// there is none.
///
/// Thin glue over [`may_microsoft_be_asked`]. No account open is not
/// Microsoft's to answer: Google's answer already gives that reason.
pub fn a_microsoft_key(
    account: Option<&Account>,
) -> MayMicrosoftBeAsked<crate::service::oauth_credentials::ClientCredentials> {
    match account {
        Some(account) => may_microsoft_be_asked(
            account,
            crate::service::oauth_credentials::credentials_for(MICROSOFT),
        ),
        None => MayMicrosoftBeAsked::NotMicrosoftsToAsk,
    }
}

/// Why there is nothing to bring for an account whose mail is at neither
/// Google nor Microsoft and which has nothing of its own to ask, such as a
/// calendar server, a feed or an address book.
pub fn at_neither_with_nothing_of_its_own(
    account: Option<&Account>,
    has_something_of_its_own: bool,
) -> Option<WhyNothingWasAsked> {
    let at_neither =
        account.is_some_and(|account| WhoRunsTheMail::of(account) == WhoRunsTheMail::SomebodyElse);
    (at_neither && !has_something_of_its_own).then_some(WhyNothingWasAsked::AtNeitherProvider)
}

/// The line a finished sync writes to the log at info: the module, who runs
/// the account's mail, whether its provider answered or why it was not asked,
/// and the counts. Numbers and words only, never a sentence the sync says.
pub fn the_finish_line(
    module: Module,
    account: Option<&Account>,
    asked: WhatWasAsked,
    counts: &[(&str, usize)],
) -> String {
    let whose = match account.map(WhoRunsTheMail::of) {
        Some(WhoRunsTheMail::Gmail) => "account at google",
        Some(WhoRunsTheMail::Microsoft) => "account at microsoft",
        Some(WhoRunsTheMail::SomebodyElse) => "account at neither",
        None => "no account",
    };
    let answered = match asked.why_not_asked {
        Some(why) => format!("not asked: {}", why.word()),
        None if asked.somebody => "answered".to_string(),
        None => "nobody answered".to_string(),
    };
    let mut line = format!("{} sync finished, {whose}, {answered}", module.word());
    for (what, how_many) in counts {
        line.push_str(&format!(", {what} {how_many}"));
    }
    line
}

/// What adding an account starts (14-02 choice 3, REAL-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatAddingAnAccountStarts {
    /// Its calendars, contacts and tasks are brought once, and each module
    /// says at most one sentence when its sync finishes.
    TheThreeSyncs,
    /// Nothing can be asked, for this reason, said once for all three
    /// modules.
    OneReason(WhyNothingWasAsked),
    /// An account at neither provider has nothing yet to bring, and nothing
    /// is said.
    Nothing,
}

/// What adding this account starts, from who runs its mail and the keys
/// this copy holds.
pub fn what_adding_an_account_starts<GoogleKey, MicrosoftKey>(
    account: &Account,
    google_key: Option<GoogleKey>,
    microsoft_key: Option<MicrosoftKey>,
    holds_the_separate_sign_in: bool,
) -> WhatAddingAnAccountStarts {
    use WhatAddingAnAccountStarts::{Nothing, OneReason, TheThreeSyncs};
    match WhoRunsTheMail::of(account) {
        WhoRunsTheMail::Gmail => {
            match may_google_be_asked(account, google_key, holds_the_separate_sign_in) {
                MayGoogleBeAsked::Yes(_) => TheThreeSyncs,
                MayGoogleBeAsked::No(why) => OneReason(why),
                MayGoogleBeAsked::NotGooglesToAsk => Nothing,
            }
        }
        WhoRunsTheMail::Microsoft => match may_microsoft_be_asked(account, microsoft_key) {
            MayMicrosoftBeAsked::Yes(_) => TheThreeSyncs,
            MayMicrosoftBeAsked::No(why) => OneReason(why),
            MayMicrosoftBeAsked::NotMicrosoftsToAsk => Nothing,
        },
        WhoRunsTheMail::SomebodyElse => Nothing,
    }
}

/// What adding this account starts, with the keys this copy holds and the
/// separate sign-in it holds, if any.
///
/// Thin glue over [`what_adding_an_account_starts`]. A browser sign-in that
/// has run out is learned only by asking, so an account whose sign-in has
/// gone starts the three syncs and each says so. An account signed in for
/// its calendars, contacts and tasks in the visit it was added in holds that
/// sign-in by now, so its three are brought rather than the reason said.
pub fn what_adding_this_account_starts(account: &Account) -> WhatAddingAnAccountStarts {
    what_adding_an_account_starts(
        account,
        crate::service::oauth_credentials::credentials_for(GOOGLE),
        crate::service::oauth_credentials::credentials_for(MICROSOFT),
        oauth::a_sign_in_is_held(oauth::GOOGLE_CALENDARS_CONTACTS_AND_TASKS, &account.id),
    )
}

/// The accounts in `after` that were not in `before`, by their identity: the
/// accounts added in one visit to the Account Manager.
pub fn added_in_this_visit<'a>(before: &[Account], after: &'a [Account]) -> Vec<&'a Account> {
    after
        .iter()
        .filter(|account| !before.iter().any(|held| held.id == account.id))
        .collect()
}

/// The accounts added in a visit, and after them each account signed in for
/// its calendars, contacts and tasks in the same visit, every one once.
pub fn with_those_signed_in_for_calendars<'a>(
    added: Vec<&'a Account>,
    after: &'a [Account],
    signed_in_for_calendars: &[String],
) -> Vec<&'a Account> {
    let signed_in = after.iter().filter(|account| {
        signed_in_for_calendars.contains(&account.id)
            && !added.iter().any(|held| held.id == account.id)
    });
    let mut to_bring = added.clone();
    to_bring.extend(signed_in);
    to_bring
}

/// What the Account Manager's Sign In for Calendars, Contacts and Tasks does
/// for the account chosen in its list (14-03 choices 1 and 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatTheSeparateSignInDoes<Key> {
    NothingChosen,
    /// The account's mail is not at Google, so Google holds none of the three.
    NotAtGoogle,
    /// Mail already signs in to Google through the browser, and that sign-in
    /// carries all four permissions (D-14).
    TheMailSignInCoversIt,
    NoGoogleSignInKey,
    /// Sign in, with the key this copy holds.
    SignsIn(Key),
}

/// What the button does for this account, with the key this copy holds.
pub fn what_the_separate_sign_in_does<Key>(
    account: Option<&Account>,
    key: Option<Key>,
) -> WhatTheSeparateSignInDoes<Key> {
    let Some(account) = account else {
        return WhatTheSeparateSignInDoes::NothingChosen;
    };
    if WhoRunsTheMail::of(account) != WhoRunsTheMail::Gmail {
        return WhatTheSeparateSignInDoes::NotAtGoogle;
    }
    if account.use_oauth {
        return WhatTheSeparateSignInDoes::TheMailSignInCoversIt;
    }
    match key {
        None => WhatTheSeparateSignInDoes::NoGoogleSignInKey,
        Some(key) => WhatTheSeparateSignInDoes::SignsIn(key),
    }
}

/// What the button does for this account, with the key this copy holds.
///
/// Thin glue over [`what_the_separate_sign_in_does`].
pub fn what_the_separate_sign_in_does_here(
    account: Option<&Account>,
) -> WhatTheSeparateSignInDoes<crate::service::oauth_credentials::ClientCredentials> {
    what_the_separate_sign_in_does(
        account,
        crate::service::oauth_credentials::credentials_for(GOOGLE),
    )
}

impl<Key> WhatTheSeparateSignInDoes<Key> {
    /// What the button says when it is pressed: the whole answer, or that
    /// signing in has begun.
    pub fn sentence(&self) -> String {
        match self {
            WhatTheSeparateSignInDoes::NothingChosen => nothing_chosen(Thing::ACCOUNT),
            WhatTheSeparateSignInDoes::NotAtGoogle => "This account's mail is not at Google, \
                 so there is no Google sign-in to make for its calendars, contacts and tasks."
                .to_string(),
            WhatTheSeparateSignInDoes::TheMailSignInCoversIt => "This account already signs in \
                 to Google through the browser, and that sign-in covers its calendars, contacts \
                 and tasks. Choose Sign In Again to renew it."
                .to_string(),
            WhatTheSeparateSignInDoes::NoGoogleSignInKey => "This copy of Wixen Mail has no \
                 Google sign-in key, so signing in through the browser cannot run. See Setting \
                 up a provider in Help."
                .to_string(),
            WhatTheSeparateSignInDoes::SignsIn(_) => "Signing in to Google for the account's \
                 calendars, contacts and tasks. Finish in the browser."
                .to_string(),
        }
    }
}

/// What the button says when signing in worked.
pub const SIGNED_IN_FOR_CALENDARS: &str = "Signed in to Google for this account's calendars, \
     contacts and tasks. They are brought when you close the Account Manager.";

/// What the button says when signing in failed, after the reason the sign-in
/// gave, which is ended with a full stop when it has none.
pub fn signing_in_for_calendars_failed(why: &str) -> String {
    let why = why.trim_end();
    let stop = match why.ends_with(['.', '?']) {
        true => "",
        false => ".",
    };
    format!("Signing in to Google for calendars, contacts and tasks failed. {why}{stop}")
}

/// What Sign In Again says for an account whose mail signs in with a
/// password: there is nothing to sign in to again, and a Gmail account is
/// pointed at the sign-in its calendars, contacts and tasks need.
pub fn what_sign_in_again_says_for_a_password_account(account: &Account) -> &'static str {
    match WhoRunsTheMail::of(account) {
        WhoRunsTheMail::Gmail => {
            "This account signs in to mail with a password, so there is nothing to authorise \
             for mail. Edit it to change its password. For its calendars, contacts and tasks, \
             choose Sign In for Calendars, Contacts and Tasks."
        }
        WhoRunsTheMail::Microsoft | WhoRunsTheMail::SomebodyElse => {
            "This account signs in with a password, so there is nothing to authorise. Edit it \
             to change its password."
        }
    }
}

/// What Refresh, `F5`, does with a module showing (D-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatRefreshDoes {
    /// Reads the open folder or saved search again, in Mail.
    ReadsTheFolder,
    /// Does what the sidebar's Sync Now does in that module.
    SyncsTheModule,
}

/// What `F5` does with this module showing.
pub fn what_refresh_does(module: PimModule) -> WhatRefreshDoes {
    match module {
        PimModule::Mail => WhatRefreshDoes::ReadsTheFolder,
        PimModule::Contacts
        | PimModule::Calendar
        | PimModule::Reminders
        | PimModule::Tasks
        | PimModule::Notes => WhatRefreshDoes::SyncsTheModule,
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
    a_google_token_with(
        account,
        crate::service::oauth_credentials::credentials_for(GOOGLE),
    )
    .await
}

/// The token, or why there is none, with the key this copy holds passed in,
/// so the decision is read without the machine's `oauth.toml`.
///
/// The token comes from the sign-in [`oauth::the_google_sign_in_for`] names:
/// the mail sign-in for an account whose mail signs in through the browser,
/// the separate one for an account on an app password (route B).
pub async fn a_google_token_with(
    account: Option<&Account>,
    key: Option<crate::service::oauth_credentials::ClientCredentials>,
) -> Result<GooglesAnswer> {
    let Some(account) = account else {
        return Ok(GooglesAnswer::NothingAsked(
            WhyNothingWasAsked::NoAccountIsOpen,
        ));
    };
    let holds_the_separate_sign_in = !account.use_oauth
        && oauth::a_sign_in_is_held(oauth::GOOGLE_CALENDARS_CONTACTS_AND_TASKS, &account.id);
    let key = match may_google_be_asked(account, key, holds_the_separate_sign_in) {
        MayGoogleBeAsked::Yes(key) => key,
        MayGoogleBeAsked::NotGooglesToAsk => return Ok(GooglesAnswer::NotGooglesToAsk),
        MayGoogleBeAsked::No(why) => return Ok(GooglesAnswer::NothingAsked(why)),
    };
    let sign_in = oauth::the_google_sign_in_for(account.use_oauth);
    match oauth::a_google_token_from(&account.id, sign_in, &key).await {
        Ok(token) => Ok(GooglesAnswer::Token(token)),
        // No token stored, none that can be read, or a refresh Google
        // refused: each is answered by signing in again, with whichever
        // control makes that sign-in. The network failing is not, and stays
        // the sync's error to count.
        Err(Error::Authentication(_)) => Ok(GooglesAnswer::NothingAsked(
            WhyNothingWasAsked::the_sign_in_that_ran_out(account),
        )),
        Err(other) => Err(other),
    }
}

/// Whether a sync asked anybody, and why it did not ask the account's own
/// provider, carried on each module's result so its summary can say it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WhatWasAsked {
    /// A pass asked somebody: Google, Microsoft, a server or a feed.
    pub somebody: bool,
    /// Why the account's provider, Google or Microsoft, was not asked, or why
    /// an account at neither has nothing to bring. Named `why_not_google`
    /// until 14-02, when Microsoft and accounts at neither gained reasons.
    pub why_not_asked: Option<WhyNothingWasAsked>,
}

impl WhatWasAsked {
    /// The running total with one pass's result folded in. A result that came
    /// back is somebody having been asked; the first reason held is kept.
    pub fn with_a_pass(self, pass: WhatWasAsked) -> Self {
        Self {
            somebody: true,
            why_not_asked: self.why_not_asked.or(pass.why_not_asked),
        }
    }

    /// The reason to say on its own: nobody at all was asked and nothing
    /// went wrong, so there are no counts worth hearing (D-08).
    fn alone(self, nothing_went_wrong: bool) -> Option<WhyNothingWasAsked> {
        self.why_not_asked
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
            None => self.why_not_asked.map(|why| why.sentence(module)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::calendar::{CalendarSyncResult, what_the_calendar_sync_did};
    use crate::application::contacts_sync::{Contacts, SyncResult, what_the_contacts_sync_did};
    use crate::application::status_sentences::{Voice, reads_as_a_persons_sentence};
    use crate::application::tasks_sync::TaskSyncResult;

    const EVERY_MODULE: [Module; 3] = [Module::Calendar, Module::Contacts, Module::Tasks];
    const EVERY_REASON: [WhyNothingWasAsked; 7] = [
        WhyNothingWasAsked::NoAccountIsOpen,
        WhyNothingWasAsked::NoGoogleSignInKey,
        WhyNothingWasAsked::SignsInWithAnAppPassword,
        WhyNothingWasAsked::TheBrowserSignInRanOut,
        WhyNothingWasAsked::TheSeparateSignInRanOut,
        WhyNothingWasAsked::NoMicrosoftSignInKey,
        WhyNothingWasAsked::AtNeitherProvider,
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
            may_google_be_asked(&a_gmail_account(true), NO_KEY, false),
            MayGoogleBeAsked::No(WhyNothingWasAsked::NoGoogleSignInKey)
        );
        assert_eq!(
            may_google_be_asked(&a_gmail_account(false), NO_KEY, true),
            MayGoogleBeAsked::No(WhyNothingWasAsked::NoGoogleSignInKey)
        );
        // Pratik's account on 2026-10-04: a key, and an app password, which
        // Google takes for mail and never for calendars, contacts or tasks.
        assert_eq!(
            may_google_be_asked(&a_gmail_account(false), A_KEY, false),
            MayGoogleBeAsked::No(WhyNothingWasAsked::SignsInWithAnAppPassword)
        );
        // Route B: the same account once it holds the separate browser
        // sign-in for its calendars, contacts and tasks.
        assert_eq!(
            may_google_be_asked(&a_gmail_account(false), A_KEY, true),
            MayGoogleBeAsked::Yes("a key")
        );
        assert_eq!(
            may_google_be_asked(&a_gmail_account(true), A_KEY, false),
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
                    may_google_be_asked(&account, key, true),
                    MayGoogleBeAsked::NotGooglesToAsk,
                    "{} asked Google with key {key:?}",
                    account.email
                );
            }
        }
        // A Workspace account on its own domain is Google's by its server.
        assert_eq!(
            may_google_be_asked(
                &an_account_at("imap.gmail.com", "me@mycompany.com"),
                A_KEY,
                false
            ),
            MayGoogleBeAsked::Yes("a key")
        );
    }

    fn a_microsoft_account() -> Account {
        an_account_at("outlook.office365.com", "me@outlook.com")
    }

    fn an_account_at_neither() -> Account {
        an_account_at("imap.example.com", "me@example.com")
    }

    #[test]
    fn test_a_microsoft_account_asks_microsoft_only_with_a_key_and_nobody_else_ever_does() {
        // Until 14-02 every sync asked Microsoft whenever this copy held a
        // Microsoft key, so a Gmail account on a copy holding both keys
        // reported "Microsoft auth" as an error on every sync.
        assert_eq!(
            may_microsoft_be_asked(&a_microsoft_account(), A_KEY),
            MayMicrosoftBeAsked::Yes("a key")
        );
        assert_eq!(
            may_microsoft_be_asked(&a_microsoft_account(), NO_KEY),
            MayMicrosoftBeAsked::No(WhyNothingWasAsked::NoMicrosoftSignInKey)
        );
        for account in [a_gmail_account(true), an_account_at_neither()] {
            assert_eq!(
                may_microsoft_be_asked(&account, A_KEY),
                MayMicrosoftBeAsked::NotMicrosoftsToAsk,
                "{} asked Microsoft",
                account.email
            );
        }
    }

    #[test]
    fn test_an_account_at_neither_with_nothing_of_its_own_says_why_there_is_nothing_to_bring() {
        assert_eq!(
            at_neither_with_nothing_of_its_own(Some(&an_account_at_neither()), false),
            Some(WhyNothingWasAsked::AtNeitherProvider)
        );
        // A calendar server, a feed or an address book of its own is asked,
        // and its counts are what is said.
        assert_eq!(
            at_neither_with_nothing_of_its_own(Some(&an_account_at_neither()), true),
            None
        );
        // An account at a provider has that provider's own answer, and no
        // account at all has its own reason.
        for account in [a_gmail_account(false), a_microsoft_account()] {
            assert_eq!(
                at_neither_with_nothing_of_its_own(Some(&account), false),
                None,
                "{}",
                account.email
            );
        }
        assert_eq!(at_neither_with_nothing_of_its_own(None, false), None);
    }

    #[test]
    fn test_the_microsoft_and_neither_reasons_name_their_way_out() {
        let module = Module::Contacts;
        let no_key = WhyNothingWasAsked::NoMicrosoftSignInKey.sentence(module);
        assert_eq!(
            no_key,
            "Nothing was asked of Microsoft for this account's contacts, because this copy of \
             Wixen Mail has no Microsoft sign-in key. See Setting up a provider in Help."
        );
        let neither = WhyNothingWasAsked::AtNeitherProvider.sentence(Module::Calendar);
        assert_eq!(
            neither,
            "This account's mail is at neither Google nor Microsoft, so there are no calendars \
             there to bring."
        );
    }

    #[test]
    fn test_the_finish_line_names_the_module_the_provider_and_the_counts_in_words() {
        let counts = [
            ("created", 2),
            ("updated", 0),
            ("deleted", 1),
            ("errors", 0),
        ];
        assert_eq!(
            the_finish_line(
                Module::Calendar,
                Some(&a_gmail_account(false)),
                not_asked(WhyNothingWasAsked::SignsInWithAnAppPassword),
                &counts,
            ),
            "calendar sync finished, account at google, not asked: app_password, created 2, \
             updated 0, deleted 1, errors 0"
        );
        let answered = WhatWasAsked {
            somebody: true,
            why_not_asked: None,
        };
        assert_eq!(
            the_finish_line(
                Module::Tasks,
                Some(&a_microsoft_account()),
                answered,
                &[("stored", 4), ("errors", 1)]
            ),
            "tasks sync finished, account at microsoft, answered, stored 4, errors 1"
        );
        assert_eq!(
            the_finish_line(
                Module::Contacts,
                Some(&an_account_at_neither()),
                WhatWasAsked::default(),
                &[("errors", 2)]
            ),
            "contacts sync finished, account at neither, nobody answered, errors 2"
        );
        assert_eq!(
            the_finish_line(
                Module::Contacts,
                None,
                not_asked(WhyNothingWasAsked::NoAccountIsOpen),
                &[]
            ),
            "contacts sync finished, no account, not asked: no_account_open"
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
                assert!(
                    said.contains("Google") || said.contains("Microsoft"),
                    "no provider is named: {said}"
                );
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
                why_not_asked: Some(why),
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
                why_not_asked: Some(why),
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
                why_not_asked: Some(why),
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
                why_not_asked: Some(WhyNothingWasAsked::NoGoogleSignInKey),
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
                why_not_asked: Some(WhyNothingWasAsked::TheBrowserSignInRanOut),
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
                    why_not_asked: Some(WhyNothingWasAsked::NoGoogleSignInKey),
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
            arm.contains("what_was_asked.why_not_asked.is_some()"),
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

    // ── Contacts and tasks say the same reason (14-01 task 2) ──────────────
    //
    // Here rather than beside each summary, whose files 76 and 22 guard
    // records fingerprint by their test counts.

    fn not_asked(why: WhyNothingWasAsked) -> WhatWasAsked {
        WhatWasAsked {
            somebody: false,
            why_not_asked: Some(why),
        }
    }

    fn a_contact_created() -> SyncResult {
        let mut created = Contacts::default();
        created.note("ann");
        SyncResult {
            created_local: created,
            ..SyncResult::default()
        }
    }

    #[test]
    fn test_the_contacts_summary_says_the_reason_alone_when_nobody_was_asked() {
        let why = WhyNothingWasAsked::NoGoogleSignInKey;
        let said = what_the_contacts_sync_did(&SyncResult {
            what_was_asked: not_asked(why),
            ..SyncResult::default()
        });
        assert_eq!(said, why.sentence(Module::Contacts));
    }

    #[test]
    fn test_the_contacts_summary_says_the_reason_after_the_counts_when_an_address_book_was_asked() {
        let why = WhyNothingWasAsked::SignsInWithAnAppPassword;
        let mut total = SyncResult {
            what_was_asked: not_asked(why),
            ..SyncResult::default()
        };
        total.absorb(a_contact_created());
        assert_eq!(
            what_the_contacts_sync_did(&total),
            format!(
                "Contacts sync: 1 created, 0 updated, 0 deleted. {}",
                why.sentence(Module::Contacts)
            )
        );
    }

    #[test]
    fn test_folding_contacts_results_keeps_the_first_reason_and_says_somebody_was_asked() {
        let mut total = SyncResult::default();
        total.absorb(SyncResult {
            what_was_asked: not_asked(WhyNothingWasAsked::TheBrowserSignInRanOut),
            ..SyncResult::default()
        });
        total.absorb(SyncResult {
            what_was_asked: not_asked(WhyNothingWasAsked::NoGoogleSignInKey),
            ..SyncResult::default()
        });
        assert_eq!(
            total.what_was_asked,
            WhatWasAsked {
                somebody: true,
                why_not_asked: Some(WhyNothingWasAsked::TheBrowserSignInRanOut),
            }
        );
    }

    #[test]
    fn test_the_tasks_summary_says_the_reason_alone_when_nobody_was_asked() {
        let why = WhyNothingWasAsked::TheBrowserSignInRanOut;
        let said = TaskSyncResult {
            what_was_asked: not_asked(why),
            ..TaskSyncResult::default()
        }
        .summary();
        assert_eq!(said, why.sentence(Module::Tasks));
    }

    #[test]
    fn test_the_tasks_summary_says_the_reason_after_the_counts_when_a_provider_was_asked() {
        let why = WhyNothingWasAsked::NoGoogleSignInKey;
        let mut total = TaskSyncResult {
            what_was_asked: not_asked(why),
            ..TaskSyncResult::default()
        };
        total.absorb(TaskSyncResult {
            stored: 2,
            lists: 1,
            ..TaskSyncResult::default()
        });
        assert_eq!(
            total.summary(),
            format!("2 tasks in 1 list. {}", why.sentence(Module::Tasks))
        );
    }

    #[test]
    fn test_folding_tasks_results_keeps_the_first_reason_and_says_somebody_was_asked() {
        let mut total = TaskSyncResult::default();
        total.absorb(TaskSyncResult {
            what_was_asked: not_asked(WhyNothingWasAsked::SignsInWithAnAppPassword),
            ..TaskSyncResult::default()
        });
        total.absorb(TaskSyncResult::default());
        assert_eq!(
            total.what_was_asked,
            WhatWasAsked {
                somebody: true,
                why_not_asked: Some(WhyNothingWasAsked::SignsInWithAnAppPassword),
            }
        );
    }

    #[test]
    fn test_a_contacts_or_tasks_sync_that_did_not_ask_google_is_signalled_as_needing_attention() {
        // D-07 for the other two modules, as for the calendar above.
        let contacts = the_windows_arm(
            "UIUpdate::ContactsSyncComplete(result) => {",
            "UIUpdate::WorkingDayChanged(",
        );
        assert!(
            contacts.contains("what_was_asked.why_not_asked.is_some()")
                && contacts.contains("a11y.signal(FeedbackEvent::AccountNeedsAttention, &msg)"),
            "a contacts sync that asked Google nothing is not signalled as needing attention: \
             {contacts}"
        );
        assert!(
            contacts.contains("a11y.signal(FeedbackEvent::SyncComplete, detail)"),
            "a contacts sync that ran no longer finishes as one: {contacts}"
        );

        // The tasks sync's sentence travels as a string, so the worker says
        // which finish it is and the window's arm signals it.
        let path = "src/presentation/wx_app.rs";
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let (_, spawn) = source
            .split_once("fn spawn_tasks_sync(")
            .unwrap_or_else(|| panic!("spawn_tasks_sync is not in {path}"));
        let (spawn, _) = spawn
            .split_once("\n}\n")
            .unwrap_or_else(|| panic!("spawn_tasks_sync does not end in {path}"));
        assert!(
            spawn.contains("what_was_asked.why_not_asked.is_some()")
                && spawn.contains("UIUpdate::ModuleSyncNeedsAttention("),
            "the tasks sync finishes as a completed sync when it asked Google nothing: {spawn}"
        );
        let tasks = the_windows_arm(
            "UIUpdate::ModuleSyncNeedsAttention(said) => {",
            "UIUpdate::WhatCouldBeFetched(",
        );
        assert!(
            tasks.contains("a11y.signal(FeedbackEvent::AccountNeedsAttention, said)"),
            "a tasks sync that asked Google nothing is not signalled as needing attention: \
             {tasks}"
        );
    }

    // ── Adding an account, and F5 in a module (14-02 task 3) ───────────────

    #[test]
    fn test_adding_an_account_starts_the_three_syncs_or_says_one_reason_or_nothing() {
        use WhatAddingAnAccountStarts::{Nothing, OneReason, TheThreeSyncs};
        let rows = [
            (a_gmail_account(true), A_KEY, NO_KEY, TheThreeSyncs),
            (
                a_gmail_account(false),
                A_KEY,
                A_KEY,
                OneReason(WhyNothingWasAsked::SignsInWithAnAppPassword),
            ),
            (
                a_gmail_account(true),
                NO_KEY,
                A_KEY,
                OneReason(WhyNothingWasAsked::NoGoogleSignInKey),
            ),
            (a_microsoft_account(), NO_KEY, A_KEY, TheThreeSyncs),
            (
                a_microsoft_account(),
                A_KEY,
                NO_KEY,
                OneReason(WhyNothingWasAsked::NoMicrosoftSignInKey),
            ),
            // Nothing of its own yet, and nothing to say about that until
            // somebody asks for a sync.
            (an_account_at_neither(), A_KEY, A_KEY, Nothing),
        ];
        for (account, google_key, microsoft_key, expected) in rows {
            assert_eq!(
                what_adding_an_account_starts(&account, google_key, microsoft_key, false),
                expected,
                "{} with Google key {google_key:?} and Microsoft key {microsoft_key:?}",
                account.email
            );
        }
        // Route B (14-03): the app-password account signed in for its
        // calendars, contacts and tasks in the visit it was added in.
        assert_eq!(
            what_adding_an_account_starts(&a_gmail_account(false), A_KEY, NO_KEY, true),
            TheThreeSyncs
        );
    }

    #[test]
    fn test_the_reason_said_on_adding_an_account_names_all_three_modules_once() {
        assert_eq!(
            WhyNothingWasAsked::SignsInWithAnAppPassword.sentence_for_every_module(),
            "Nothing was asked of Google for this account's calendars, contacts and tasks. The \
             account signs in to mail with an app password, and Google gives calendars, \
             contacts and tasks only to a browser sign-in. Open the Account Manager with \
             Ctrl+Shift+A, choose the account and press Sign In for Calendars, Contacts and \
             Tasks."
        );
        for why in EVERY_REASON {
            let said = why.sentence_for_every_module();
            assert_eq!(
                reads_as_a_persons_sentence(&said, Voice::Answer),
                Ok(()),
                "{why:?}"
            );
            assert!(said.contains("calendars, contacts and tasks"), "{said}");
        }
    }

    #[test]
    fn test_the_accounts_added_in_a_visit_are_those_that_were_not_there_before() {
        let with_id = |id: &str| Account {
            id: id.into(),
            ..Account::default()
        };
        let before = [with_id("a1"), with_id("a2")];
        // A2 renamed in the same visit is the same account, not a new one.
        let after = [
            Account {
                name: "Renamed".into(),
                ..with_id("a2")
            },
            with_id("a3"),
            with_id("a4"),
        ];
        let added: Vec<&str> = added_in_this_visit(&before, &after)
            .iter()
            .map(|account| account.id.as_str())
            .collect();
        assert_eq!(added, ["a3", "a4"]);
    }

    #[test]
    fn test_f5_reads_the_folder_in_mail_and_syncs_every_other_module() {
        use WhatRefreshDoes::{ReadsTheFolder, SyncsTheModule};
        let rows = [
            (PimModule::Mail, ReadsTheFolder),
            (PimModule::Contacts, SyncsTheModule),
            (PimModule::Calendar, SyncsTheModule),
            (PimModule::Reminders, SyncsTheModule),
            (PimModule::Tasks, SyncsTheModule),
            (PimModule::Notes, SyncsTheModule),
        ];
        assert_eq!(rows.len(), PimModule::ALL.len(), "a module has no row");
        for (module, expected) in rows {
            assert_eq!(what_refresh_does(module), expected, "{module:?}");
        }
    }

    /// A reading's answer: nothing, or the complaint naming what is wrong.
    type Reading<T> = std::result::Result<T, String>;

    /// The window's source, read whole.
    fn the_window() -> String {
        let path = "src/presentation/wx_app.rs";
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
    }

    /// One function's text, from its signature to the brace closing it at
    /// column nought.
    fn the_function(source: &str, signature: &str) -> Reading<String> {
        let (_, after) = source.split_once(signature).ok_or(format!(
            "{signature} is not in the window, so this reads nothing"
        ))?;
        let (body, _) = after
            .split_once("\n}\n")
            .ok_or(format!("{signature} does not end"))?;
        Ok(body.to_string())
    }

    /// The Account Manager starts the first sync of each account added in a
    /// visit, through the answer, and says one reason when it cannot (REAL-01:
    /// "the sync run on account creation").
    fn the_account_manager_starts_an_added_accounts_first_sync(window: &str) -> Reading<()> {
        let manager = the_function(window, "fn handle_account_mgr(")?;
        if !manager.contains("tx: &Sender<UIUpdate>") {
            return Err("handle_account_mgr holds no sender, so it can start no sync".into());
        }
        if !manager.contains("added_in_this_visit(")
            || !manager.contains("bring_what_a_new_account_holds(")
        {
            return Err(
                "handle_account_mgr starts nothing for an account added in the visit".into(),
            );
        }
        let bring = the_function(window, "fn bring_what_a_new_account_holds(")?;
        for needed in [
            "what_adding_this_account_starts(",
            "spawn_contacts_sync(",
            "spawn_calendar_sync(",
            "spawn_tasks_sync(",
            "sentence_for_every_module()",
        ] {
            if !bring.contains(needed) {
                return Err(format!(
                    "bring_what_a_new_account_holds does not reach {needed}"
                ));
            }
        }
        Ok(())
    }

    /// The first arm `F5` reaches asks which module is showing, and in a
    /// module other than Mail reaches the one place that chooses a module's
    /// sync, which Sync Now reaches too (D-11).
    fn f5_asks_the_module_showing_before_it_reads_a_folder(window: &str) -> Reading<()> {
        let (_, after) = window
            .split_once("_ if id == ID_REFRESH_FOLDER")
            .ok_or("no arm answers F5")?;
        let (guard, rest) = after.split_once("=>").ok_or("the F5 arm has no body")?;
        if !guard.contains("what_refresh_does(")
            || !guard.contains("WhatRefreshDoes::SyncsTheModule")
        {
            return Err(
                "the first arm F5 reaches does not ask which module is showing, so F5 in \
                 Contacts reads whatever mail folder the tree still has selected"
                    .into(),
            );
        }
        let (arm, _) = rest
            .split_once("_ if id == ")
            .ok_or("the F5 arm does not end")?;
        if !arm.contains("sync_the_module(") {
            return Err("F5 in a module does not reach that module's sync".into());
        }
        let (_, sync_now) = window
            .split_once("_ if id == ID_CONTEXT_SYNC_NOW =>")
            .ok_or("no Sync Now arm")?;
        let (sync_now, _) = sync_now
            .split_once("_ if id == ")
            .ok_or("the Sync Now arm does not end")?;
        if !sync_now.contains("sync_the_module(") {
            return Err(
                "Sync Now chooses a module's sync for itself, so it and F5 can come apart".into(),
            );
        }
        Ok(())
    }

    #[test]
    fn test_the_account_manager_starts_the_first_sync_of_an_added_account() {
        the_account_manager_starts_an_added_accounts_first_sync(&the_window())
            .unwrap_or_else(|why| panic!("{why}"));
    }

    #[test]
    fn test_f5_asks_the_module_showing_before_it_reads_a_folder() {
        f5_asks_the_module_showing_before_it_reads_a_folder(&the_window())
            .unwrap_or_else(|why| panic!("{why}"));
    }

    /// The window with one substring of one function replaced, so a plant
    /// lands inside the region a reading reads.
    fn planted_in(window: &str, signature: &str, from: &str, to: &str) -> String {
        let body = the_function(window, signature).unwrap_or_else(|why| panic!("{why}"));
        let changed = body.replacen(from, to, 1);
        assert_ne!(changed, body, "the plant changed nothing in {signature}");
        window.replacen(&body, &changed, 1)
    }

    #[test]
    fn test_the_reading_of_the_account_manager_sees_a_visit_that_starts_nothing() {
        let window = the_window();
        let reading = the_account_manager_starts_an_added_accounts_first_sync;
        let starts_nothing = planted_in(
            &window,
            "fn handle_account_mgr(",
            "bring_what_a_new_account_holds(",
            "let _ = (",
        );
        let why = reading(&starts_nothing).expect_err("a visit that starts nothing passed");
        assert!(why.contains("starts nothing"), "{why}");
        let says_nothing = planted_in(
            &window,
            "fn bring_what_a_new_account_holds(",
            "sentence_for_every_module()",
            "to_string()",
        );
        let why = reading(&says_nothing).expect_err("a reason never said passed");
        assert!(why.contains("sentence_for_every_module()"), "{why}");
    }

    #[test]
    fn test_the_reading_of_f5_sees_an_arm_that_reads_the_folder_whatever_is_showing() {
        let window = the_window();
        let asks_nothing = window.replacen("what_refresh_does(", "a_module_nobody_asks(", 1);
        assert_ne!(asks_nothing, window, "the plant changed nothing");
        let why = f5_asks_the_module_showing_before_it_reads_a_folder(&asks_nothing)
            .expect_err("an F5 that never asks the module passed");
        assert!(why.contains("does not ask which module"), "{why}");
    }

    // ── The separate sign-in for calendars, contacts and tasks (14-03) ────
    //
    // Route B of #22: mail keeps its app password, and Google is signed in to
    // through the browser for these three alone. Under test the credential
    // store is a map per thread, so nothing here reaches the real one, and the
    // Google key is passed in rather than read from this machine.

    use crate::service::oauth::{GOOGLE_CALENDARS_CONTACTS_AND_TASKS, OAuthTokenSet};
    use crate::service::oauth_credentials::ClientCredentials;

    /// A Google sign-in key as a copy might hold one.
    fn a_google_key() -> ClientCredentials {
        ClientCredentials {
            client_id: "a-client".into(),
            client_secret: Some("a-secret".into()),
            tenant_id: None,
        }
    }

    /// Keep a token under one of an account's Google sign-ins, good for the
    /// time given from now, or run out when it is negative.
    fn holding(sign_in: &str, account: &Account, token: &str, good_for: chrono::TimeDelta) {
        let kept = OAuthTokenSet {
            access_token: token.into(),
            refresh_token: None,
            token_type: "Bearer".into(),
            scope: None,
            expires_at: Some((chrono::Utc::now() + good_for).to_rfc3339()),
        };
        crate::service::secret_store::write(
            &crate::service::oauth::keyring_service(sign_in),
            &account.id,
            &serde_json::to_string(&kept).expect("a token set to write"),
        )
        .expect("the test store keeps it");
    }

    /// What came back, in words, since the answer prints nothing of itself.
    fn what_came_back(answer: &GooglesAnswer) -> String {
        match answer {
            GooglesAnswer::Token(_) => "a token".into(),
            GooglesAnswer::NotGooglesToAsk => "not Google's to ask".into(),
            GooglesAnswer::NothingAsked(why) => format!("nothing asked: {why:?}"),
        }
    }

    /// The Account Manager's button the reasons name, as it is heard.
    const THE_BUTTON: &str = "Sign In for Calendars, Contacts and Tasks";

    #[tokio::test]
    async fn test_an_app_password_gmail_account_holding_the_separate_sign_in_asks_google_with_it() {
        use crate::common::answering::{answering, asked_for, heard};
        use crate::service::google_api::GoogleApiClient;

        let account = a_gmail_account(false);
        holding(
            GOOGLE_CALENDARS_CONTACTS_AND_TASKS,
            &account,
            "the-separate-token",
            chrono::TimeDelta::days(1),
        );

        let answer = a_google_token_with(Some(&account), Some(a_google_key()))
            .await
            .expect("nothing here meets the network");
        let GooglesAnswer::Token(token) = answer else {
            panic!(
                "the separate sign-in was not used: {}",
                what_came_back(&answer)
            );
        };

        let (address, listening) = answering("200 OK", "application/json", "{}".into()).await;
        GoogleApiClient::new()
            .pointed_at(&format!("http://{address}"))
            .list_events(&token, None, None, None, "primary")
            .await
            .expect("the stand-in answers");
        let request = heard(listening, "the calendar's events")
            .await
            .unwrap_or_else(|why| panic!("{why}"));
        assert!(
            asked_for(&request).contains("/calendars/primary/events"),
            "{}",
            asked_for(&request)
        );
        assert!(
            request
                .to_ascii_lowercase()
                .contains("authorization: bearer the-separate-token"),
            "Google was not asked with the separate sign-in's token"
        );
    }

    #[tokio::test]
    async fn test_a_browser_gmail_account_asks_google_with_its_mail_sign_in() {
        // D-14: the mail sign-in already carries all four permissions, so a
        // separate one, if somehow held, is not the one used.
        let account = a_gmail_account(true);
        holding(
            GOOGLE,
            &account,
            "the-mail-token",
            chrono::TimeDelta::days(1),
        );
        holding(
            GOOGLE_CALENDARS_CONTACTS_AND_TASKS,
            &account,
            "the-separate-token",
            chrono::TimeDelta::days(1),
        );

        let answer = a_google_token_with(Some(&account), Some(a_google_key()))
            .await
            .expect("nothing here meets the network");

        match answer {
            GooglesAnswer::Token(token) => assert_eq!(token, "the-mail-token"),
            other => panic!("{}", what_came_back(&other)),
        }
    }

    #[tokio::test]
    async fn test_an_app_password_gmail_account_with_no_separate_sign_in_names_the_button() {
        let answer = a_google_token_with(Some(&a_gmail_account(false)), Some(a_google_key()))
            .await
            .expect("nothing here meets the network");

        let GooglesAnswer::NothingAsked(why) = answer else {
            panic!("{}", what_came_back(&answer));
        };
        assert_eq!(why, WhyNothingWasAsked::SignsInWithAnAppPassword);
        let said = why.sentence(Module::Calendar);
        assert!(said.contains(THE_BUTTON), "{said}");
        assert!(said.contains("Ctrl+Shift+A"), "{said}");
        assert!(said.contains("app password"), "{said}");
    }

    #[tokio::test]
    async fn test_an_app_password_gmail_account_whose_separate_sign_in_ran_out_names_the_button() {
        let account = a_gmail_account(false);
        // Run out an hour ago, with no refresh token to renew it: what a key
        // in Testing leaves after its week.
        holding(
            GOOGLE_CALENDARS_CONTACTS_AND_TASKS,
            &account,
            "an-old-token",
            chrono::TimeDelta::hours(-1),
        );

        let answer = a_google_token_with(Some(&account), Some(a_google_key()))
            .await
            .expect("nothing here meets the network");

        let GooglesAnswer::NothingAsked(why) = answer else {
            panic!("{}", what_came_back(&answer));
        };
        assert_eq!(why, WhyNothingWasAsked::TheSeparateSignInRanOut);
        let said = why.sentence(Module::Tasks);
        assert!(said.contains(THE_BUTTON), "{said}");
        assert!(said.contains("Ctrl+Shift+A"), "{said}");
        assert!(!said.contains("Sign In Again"), "{said}");
    }

    // ── The Account Manager's button (14-03 task 2) ────────────────────────

    #[test]
    fn test_the_separate_sign_in_button_answers_each_kind_of_account() {
        use WhatTheSeparateSignInDoes::{
            NoGoogleSignInKey, NotAtGoogle, NothingChosen, SignsIn, TheMailSignInCoversIt,
        };
        assert_eq!(what_the_separate_sign_in_does(None, A_KEY), NothingChosen);
        for account in [a_microsoft_account(), an_account_at_neither()] {
            assert_eq!(
                what_the_separate_sign_in_does(Some(&account), A_KEY),
                NotAtGoogle,
                "{}",
                account.email
            );
        }
        // Choice 3: the mail sign-in already carries all four permissions,
        // so the button says so rather than greying out.
        assert_eq!(
            what_the_separate_sign_in_does(Some(&a_gmail_account(true)), A_KEY),
            TheMailSignInCoversIt
        );
        assert_eq!(
            what_the_separate_sign_in_does(Some(&a_gmail_account(false)), NO_KEY),
            NoGoogleSignInKey
        );
        assert_eq!(
            what_the_separate_sign_in_does(Some(&a_gmail_account(false)), A_KEY),
            SignsIn("a key")
        );
    }

    #[test]
    fn test_what_the_separate_sign_in_button_says_names_its_way_out() {
        use WhatTheSeparateSignInDoes::{
            NoGoogleSignInKey, NotAtGoogle, NothingChosen, SignsIn, TheMailSignInCoversIt,
        };
        assert_eq!(NothingChosen::<&str>.sentence(), "Choose an account first.");
        assert_eq!(
            SignsIn("a key").sentence(),
            "Signing in to Google for the account's calendars, contacts and tasks. Finish in \
             the browser."
        );
        let covered = TheMailSignInCoversIt::<&str>.sentence();
        assert!(covered.contains("Sign In Again"), "{covered}");
        let no_key = NoGoogleSignInKey::<&str>.sentence();
        assert!(no_key.contains("Google sign-in key"), "{no_key}");
        assert!(no_key.contains("Setting up a provider in Help"), "{no_key}");
        let not_google = NotAtGoogle::<&str>.sentence();
        assert!(not_google.contains("not at Google"), "{not_google}");
        let every_answer: [WhatTheSeparateSignInDoes<&str>; 5] = [
            NothingChosen,
            NotAtGoogle,
            TheMailSignInCoversIt,
            NoGoogleSignInKey,
            SignsIn("a key"),
        ];
        for said in every_answer.iter().map(WhatTheSeparateSignInDoes::sentence) {
            assert_eq!(
                reads_as_a_persons_sentence(&said, Voice::Answer),
                Ok(()),
                "{said}"
            );
            assert!(!said.to_lowercase().contains("oauth"), "jargon: {said}");
        }
    }

    #[test]
    fn test_signing_in_for_calendars_says_it_worked_or_why_not() {
        assert_eq!(
            reads_as_a_persons_sentence(SIGNED_IN_FOR_CALENDARS, Voice::Answer),
            Ok(())
        );
        assert!(
            SIGNED_IN_FOR_CALENDARS.contains("calendars, contacts and tasks"),
            "{SIGNED_IN_FOR_CALENDARS}"
        );
        // When they are brought, so nobody waits for them with the window open.
        assert!(
            SIGNED_IN_FOR_CALENDARS.contains("close the Account Manager"),
            "{SIGNED_IN_FOR_CALENDARS}"
        );
        let failed = signing_in_for_calendars_failed("The browser was closed.");
        assert_eq!(
            failed,
            "Signing in to Google for calendars, contacts and tasks failed. The browser was \
             closed."
        );
        assert_eq!(reads_as_a_persons_sentence(&failed, Voice::Answer), Ok(()));
    }

    #[test]
    fn test_sign_in_again_on_a_password_gmail_account_names_the_separate_sign_in() {
        let gmail = what_sign_in_again_says_for_a_password_account(&a_gmail_account(false));
        assert!(gmail.contains(THE_BUTTON), "{gmail}");
        assert!(gmail.contains("Edit it to change its password"), "{gmail}");
        // Any other password account hears what it heard before.
        assert_eq!(
            what_sign_in_again_says_for_a_password_account(&an_account_at_neither()),
            "This account signs in with a password, so there is nothing to authorise. Edit it \
             to change its password."
        );
    }

    #[test]
    fn test_an_account_signed_in_for_calendars_in_a_visit_is_brought_once() {
        let with_id = |id: &str| Account {
            id: id.into(),
            ..Account::default()
        };
        let after = [with_id("a1"), with_id("a2"), with_id("a3")];
        let added = vec![&after[2]];
        // A3 was added and signed in, so it is brought once; a2 was only
        // signed in; an id the list no longer holds is passed over.
        let signed_in = ["a3".to_string(), "a2".to_string(), "gone".to_string()];
        let brought: Vec<&str> = with_those_signed_in_for_calendars(added, &after, &signed_in)
            .iter()
            .map(|account| account.id.as_str())
            .collect();
        assert_eq!(brought, ["a3", "a2"]);
    }

    /// The Account Manager's source, read whole.
    fn the_account_manager() -> String {
        let path = "src/presentation/wx_account_manager.rs";
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
    }

    /// The button signs in under the separate sign-in's name, the account it
    /// signed in is carried out of the Account Manager whether or not
    /// anything else changed, and the window brings it once (D-13).
    fn the_account_manager_carries_out_an_account_signed_in_for_calendars(
        manager: &str,
        window: &str,
    ) -> Reading<()> {
        let sign_in = the_function(manager, "fn sign_in_for_calendars(")?;
        if !sign_in.contains("GOOGLE_CALENDARS_CONTACTS_AND_TASKS")
            || !sign_in.contains(".authorize()")
        {
            return Err("the button does not sign in under the separate sign-in's name".into());
        }
        let wired = the_function(manager, "fn wire_account_manager_actions(")?;
        if !wired.contains("sign_in_for_calendars_selected(") {
            return Err("nothing presses the button's function".into());
        }
        let shown = the_function(manager, "pub fn show_account_manager_dialog(")?;
        if !shown.contains("outcome.changed || !signed_in_for_calendars.is_empty()") {
            return Err(
                "an account signed in for calendars is left behind when nothing else changed"
                    .into(),
            );
        }
        let handler = the_function(window, "fn handle_account_mgr(")?;
        if !handler.contains("with_those_signed_in_for_calendars(") {
            return Err("the window brings nothing for an account signed in for calendars".into());
        }
        Ok(())
    }

    #[test]
    fn test_the_account_manager_carries_out_an_account_signed_in_for_calendars() {
        the_account_manager_carries_out_an_account_signed_in_for_calendars(
            &the_account_manager(),
            &the_window(),
        )
        .unwrap_or_else(|why| panic!("{why}"));
    }

    #[test]
    fn test_the_reading_of_the_account_manager_sees_a_sign_in_left_behind() {
        let manager = the_account_manager();
        let left_behind = planted_in(
            &manager,
            "pub fn show_account_manager_dialog(",
            "outcome.changed || !signed_in_for_calendars.is_empty()",
            "outcome.changed",
        );
        let why = the_account_manager_carries_out_an_account_signed_in_for_calendars(
            &left_behind,
            &the_window(),
        )
        .expect_err("a sign-in left behind passed");
        assert!(why.contains("left behind"), "{why}");
    }
}

//! Where a new thing goes when somebody presses a key to make one.
//!
//! Six commands create six kinds of item, and each has to land somewhere
//! without asking. The rule is short: the default account if it can hold that
//! kind of thing, and this computer if it cannot.
//!
//! What an account can hold comes from its provider rather than from how its
//! mail arrives. Google and Microsoft accounts sync contacts and calendars, so
//! a contact made in one turns up on the phone. An account that is only a mail
//! server, which is every POP or plain IMAP account, syncs nothing else, so a
//! contact made there would live in a database on one computer while pretending
//! to belong to an account. It goes to the local account instead, which is the
//! honest version of the same thing.
//!
//! Tasks, notes and reminders are local for everybody today, because nothing
//! here syncs them anywhere yet.

use crate::data::account::Account;

/// The kinds of thing a "new" command makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Mail,
    Contact,
    Event,
    Reminder,
    Task,
    Note,
}

impl ItemKind {
    /// Every kind, so menus and tests cover the whole set.
    pub const ALL: [ItemKind; 6] = [
        ItemKind::Mail,
        ItemKind::Contact,
        ItemKind::Event,
        ItemKind::Reminder,
        ItemKind::Task,
        ItemKind::Note,
    ];

    /// What it is called in a menu, after "New".
    pub fn label(self) -> &'static str {
        match self {
            ItemKind::Mail => "Message",
            ItemKind::Contact => "Contact",
            ItemKind::Event => "Event",
            ItemKind::Reminder => "Reminder",
            ItemKind::Task => "Task",
            ItemKind::Note => "Note",
        }
    }

    /// The key that makes one from anywhere.
    ///
    /// `Ctrl+Shift+R` is missing on purpose. It is Reply All here, as it is in
    /// Outlook, Thunderbird and Gmail, so a reminder takes `Ctrl+Shift+D` for
    /// "due" rather than taking a key every user arriving from another client
    /// already knows.
    pub fn shortcut(self) -> &'static str {
        match self {
            ItemKind::Mail => "Ctrl+Shift+M",
            ItemKind::Contact => "Ctrl+Shift+C",
            ItemKind::Event => "Ctrl+Shift+E",
            ItemKind::Reminder => "Ctrl+Shift+D",
            ItemKind::Task => "Ctrl+Shift+T",
            ItemKind::Note => "Ctrl+Shift+N",
        }
    }
}

/// Where a new item is created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// An account that will sync it to the provider.
    Account(String),
    /// This computer only. Nothing carries it anywhere else.
    Local,
}

impl Destination {
    /// How the destination is announced when the item is made.
    ///
    /// Said every time rather than only when it is surprising, because the
    /// interesting case is the one somebody did not expect and there is no way
    /// to know in advance which that is.
    pub fn spoken(&self, accounts: &[Account]) -> String {
        match self {
            Destination::Account(id) => accounts
                .iter()
                .find(|account| &account.id == id)
                .map(|account| account.display_name())
                .unwrap_or_else(|| "the default account".to_string()),
            // Named as a place rather than as an absence. "On this computer"
            // says where it is; "no account" says where it is not.
            Destination::Local => "On this computer".to_string(),
        }
    }
}

/// Providers whose contacts we can sync.
const CONTACT_PROVIDERS: [&str; 2] = ["gmail", "outlook"];
/// Providers whose calendars we can sync.
const CALENDAR_PROVIDERS: [&str; 2] = ["gmail", "outlook"];

/// Whether this account's provider syncs this kind of item.
///
/// Mail is the one every account can hold, because being a mail account is
/// what makes it an account.
pub fn supports(account: &Account, kind: ItemKind) -> bool {
    let provider = provider_key(account);
    match kind {
        ItemKind::Mail => true,
        ItemKind::Contact => provider.is_some_and(|p| CONTACT_PROVIDERS.contains(&p.as_str())),
        ItemKind::Event => provider.is_some_and(|p| CALENDAR_PROVIDERS.contains(&p.as_str())),
        // Nothing syncs these anywhere yet. Offering an account for them would
        // promise a sync that does not exist.
        ItemKind::Reminder | ItemKind::Task | ItemKind::Note => false,
    }
}

/// The provider name as the sync code spells it.
fn provider_key(account: &Account) -> Option<String> {
    crate::application::mail_auth::provider_of(account)
}

/// Where a new item of this kind belongs.
///
/// `None` only for mail with no account at all, because a message cannot be
/// sent from this computer alone. Everything else always has somewhere to go.
pub fn destination(
    kind: ItemKind,
    accounts: &[Account],
    default_id: Option<&str>,
) -> Option<Destination> {
    let default = default_id.and_then(|id| accounts.iter().find(|account| account.id == id));

    if kind == ItemKind::Mail {
        // Falling back to the first account rather than to nothing: somebody
        // with one account and no default set still expects Ctrl+Shift+M to
        // open a message.
        return default
            .or_else(|| accounts.first())
            .map(|account| Destination::Account(account.id.clone()));
    }

    match default {
        // Deliberately not "the first account that supports it". A contact
        // appearing in an account somebody was not thinking about is worse
        // than one they can find on this computer, where they put it.
        Some(account) if supports(account, kind) => Some(Destination::Account(account.id.clone())),
        _ => Some(Destination::Local),
    }
}

/// Which account should be the default, given what is configured.
///
/// The first account somebody sets up becomes the default without being asked,
/// because for most people it is the only one and choosing is a question with
/// one answer. It can be reassigned in the accounts dialog once there is
/// something to reassign it to.
pub fn default_after_change(accounts: &[Account], current: Option<&str>) -> Option<String> {
    if let Some(id) = current
        && accounts.iter().any(|account| account.id == id)
    {
        return Some(id.to_string());
    }
    // The current default has been deleted, or there never was one.
    accounts.first().map(|account| account.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: &str, email: &str) -> Account {
        let mut account = Account::new("Work".to_string(), email.to_string());
        account.id = id.to_string();
        account
    }

    #[test]
    fn test_every_kind_has_its_own_key() {
        let mut keys: Vec<&str> = ItemKind::ALL.iter().map(|kind| kind.shortcut()).collect();
        keys.sort_unstable();
        keys.dedup();

        assert_eq!(keys.len(), ItemKind::ALL.len(), "two kinds share a key");
    }

    #[test]
    fn test_no_new_item_key_takes_reply_all() {
        // Ctrl+Shift+R is Reply All in Outlook, Thunderbird and Gmail, and
        // here. A reminder is not worth making every user relearn it.
        for kind in ItemKind::ALL {
            assert_ne!(kind.shortcut(), "Ctrl+Shift+R", "{kind:?} took Reply All");
        }
    }

    #[test]
    fn test_a_gmail_account_holds_contacts_and_events() {
        let gmail = account("a1", "me@gmail.com");

        assert!(supports(&gmail, ItemKind::Contact));
        assert!(supports(&gmail, ItemKind::Event));
    }

    #[test]
    fn test_a_mail_only_account_holds_nothing_but_mail() {
        // A POP or plain IMAP account is a mail server and nothing else. A
        // contact "in" it would live in one database on one computer while
        // claiming to belong to an account.
        let plain = account("a1", "me@myhost.example");

        assert!(supports(&plain, ItemKind::Mail));
        for kind in [ItemKind::Contact, ItemKind::Event] {
            assert!(!supports(&plain, kind), "{kind:?} should not be offered");
        }
    }

    #[test]
    fn test_nothing_syncs_tasks_notes_or_reminders_yet() {
        // Saying an account holds them would promise a sync that does not
        // exist, and the promise only breaks on a second device.
        let gmail = account("a1", "me@gmail.com");

        for kind in [ItemKind::Task, ItemKind::Note, ItemKind::Reminder] {
            assert!(!supports(&gmail, kind), "{kind:?} claimed to sync");
        }
    }

    #[test]
    fn test_a_new_item_goes_to_the_default_account_when_it_can_hold_it() {
        let accounts = vec![
            account("a1", "me@gmail.com"),
            account("a2", "me@work.example"),
        ];

        let where_to = destination(ItemKind::Contact, &accounts, Some("a1"));

        assert_eq!(where_to, Some(Destination::Account("a1".to_string())));
    }

    #[test]
    fn test_an_item_the_default_account_cannot_hold_goes_local() {
        let accounts = vec![account("a1", "me@myhost.example")];

        let where_to = destination(ItemKind::Contact, &accounts, Some("a1"));

        assert_eq!(where_to, Some(Destination::Local));
    }

    #[test]
    fn test_it_does_not_wander_off_to_another_account_that_could_hold_it() {
        // A contact turning up in an account somebody was not thinking about
        // is worse than one on this computer, where they put it.
        let accounts = vec![
            account("a1", "me@myhost.example"),
            account("a2", "me@gmail.com"),
        ];

        let where_to = destination(ItemKind::Contact, &accounts, Some("a1"));

        assert_eq!(where_to, Some(Destination::Local));
    }

    #[test]
    fn test_a_task_is_local_even_on_a_gmail_default() {
        let accounts = vec![account("a1", "me@gmail.com")];

        for kind in [ItemKind::Task, ItemKind::Note, ItemKind::Reminder] {
            assert_eq!(
                destination(kind, &accounts, Some("a1")),
                Some(Destination::Local),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn test_mail_falls_back_to_the_only_account_when_no_default_is_set() {
        // Somebody with one account still expects the key to open a message.
        let accounts = vec![account("a1", "me@gmail.com")];

        let where_to = destination(ItemKind::Mail, &accounts, None);

        assert_eq!(where_to, Some(Destination::Account("a1".to_string())));
    }

    #[test]
    fn test_mail_with_no_account_at_all_has_nowhere_to_go() {
        // Unlike every other kind: a message cannot be sent from this
        // computer alone, so this is the one case that has to say no.
        assert_eq!(destination(ItemKind::Mail, &[], None), None);
    }

    #[test]
    fn test_a_note_can_be_made_before_any_account_exists() {
        assert_eq!(
            destination(ItemKind::Note, &[], None),
            Some(Destination::Local)
        );
    }

    #[test]
    fn test_the_first_account_becomes_the_default_without_being_asked() {
        let accounts = vec![account("a1", "me@gmail.com")];

        assert_eq!(default_after_change(&accounts, None).as_deref(), Some("a1"));
    }

    #[test]
    fn test_adding_a_second_account_does_not_move_the_default() {
        let accounts = vec![
            account("a1", "me@gmail.com"),
            account("a2", "me@work.example"),
        ];

        assert_eq!(
            default_after_change(&accounts, Some("a1")).as_deref(),
            Some("a1")
        );
    }

    #[test]
    fn test_deleting_the_default_account_hands_it_to_another() {
        // Otherwise every new-item key would report a default that is not
        // there, and the fix would be somewhere nobody would look.
        let remaining = vec![account("a2", "me@work.example")];

        assert_eq!(
            default_after_change(&remaining, Some("a1")).as_deref(),
            Some("a2")
        );
    }

    #[test]
    fn test_deleting_the_last_account_leaves_no_default() {
        assert_eq!(default_after_change(&[], Some("a1")), None);
    }

    #[test]
    fn test_the_destination_is_named_in_words_somebody_can_act_on() {
        let accounts = vec![account("a1", "me@gmail.com")];

        assert!(
            Destination::Account("a1".to_string())
                .spoken(&accounts)
                .contains("Work")
        );
        assert_eq!(Destination::Local.spoken(&accounts), "On this computer");
    }
}

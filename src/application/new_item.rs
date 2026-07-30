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
//! Tasks now sync too, on both providers, through `application::tasks_sync`.
//! Notes and reminders stay local, and for reasons worth writing down because
//! the obvious reading of each is wrong:
//!
//! - **Notes could sync on Microsoft** through OneNote in Graph, and it is not
//!   written. A OneNote page is an HTML document inside a section inside a
//!   notebook rather than a title and a body, so the mapping is a decision
//!   somebody has to make rather than an afternoon's work. Google Keep has an
//!   API and it is Workspace only, so a consumer Gmail account cannot use it at
//!   all and Google notes stay here whatever happens.
//! - **Reminders are not a thing to sync on either.** In Outlook and Exchange a
//!   reminder is a property of an event or a task rather than an item of its
//!   own, and Google folded its Reminders into Tasks in 2023. So a standalone
//!   reminder is ours to keep however good the sync gets.
//!
//! What decides the rule below is what we actually sync, not what could be
//! synced. Putting a note in a Gmail account today would promise something that
//! never happens, and the promise only breaks on a second device, which is the
//! worst place to find out.

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

    /// What several of them are called.
    ///
    /// Written out rather than adding an "s", because a confirmation that says
    /// "the 12 Tasks in it" reads as a proper noun and one that says "12
    /// Messages" is not what anybody calls them.
    pub fn plural(self) -> &'static str {
        match self {
            ItemKind::Mail => "messages",
            ItemKind::Contact => "contacts",
            ItemKind::Event => "events",
            ItemKind::Reminder => "reminders",
            ItemKind::Task => "tasks",
            ItemKind::Note => "notes",
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

/// The containers items are kept in.
///
/// Separate from [`ItemKind`] because these are made and named rather than
/// filled in, but they follow the same placement rule through [`Self::holds`]:
/// a calendar belongs wherever the events in it would belong, so a container
/// and its contents can never end up in different accounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerKind {
    Calendar,
    TaskList,
    NoteFolder,
    ContactGroup,
}

impl ContainerKind {
    /// Every container, so menus and tests cover the whole set.
    pub const ALL: [ContainerKind; 4] = [
        ContainerKind::Calendar,
        ContainerKind::TaskList,
        ContainerKind::NoteFolder,
        ContainerKind::ContactGroup,
    ];

    /// Which kind of container holds one of these, if any does.
    ///
    /// `None` for mail, which lives in IMAP folders rather than in one of
    /// these, and for a reminder, which belongs to an account and nothing
    /// smaller.
    pub const fn holding(kind: ItemKind) -> Option<Self> {
        match kind {
            ItemKind::Event => Some(ContainerKind::Calendar),
            ItemKind::Task => Some(ContainerKind::TaskList),
            ItemKind::Note => Some(ContainerKind::NoteFolder),
            ItemKind::Contact => Some(ContainerKind::ContactGroup),
            ItemKind::Mail | ItemKind::Reminder => None,
        }
    }

    /// What it is called in a menu, after "New".
    pub fn label(self) -> &'static str {
        match self {
            ContainerKind::Calendar => "Calendar",
            ContainerKind::TaskList => "Task list",
            ContainerKind::NoteFolder => "Note folder",
            ContainerKind::ContactGroup => "Contact group",
        }
    }

    /// The kind of item it holds, which is what decides where it goes.
    pub fn holds(self) -> ItemKind {
        match self {
            ContainerKind::Calendar => ItemKind::Event,
            ContainerKind::TaskList => ItemKind::Task,
            ContainerKind::NoteFolder => ItemKind::Note,
            ContainerKind::ContactGroup => ItemKind::Contact,
        }
    }
}

/// The account id items kept only on this computer are filed under.
///
/// A reserved id rather than a null, because every stored item is scoped to an
/// account and a null would mean touching six tables to allow one. It is not a
/// mail account and never appears in the account list; the panels show it as a
/// second source alongside whichever account is being looked at.
pub const LOCAL_ACCOUNT_ID: &str = "local";

/// Where a new item is created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// An account that will sync it to the provider.
    Account(String),
    /// This computer only. Nothing carries it anywhere else.
    Local,
}

impl Destination {
    /// The account id to file the item under.
    pub fn account_id(&self) -> &str {
        match self {
            Destination::Account(id) => id,
            Destination::Local => LOCAL_ACCOUNT_ID,
        }
    }

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
/// Providers whose tasks we can sync.
///
/// Google Tasks and Microsoft To Do, both synced in each direction by
/// `application::tasks_sync`. A task made here goes into the account's first
/// list, which is the provider's default one, and is sent up at the next sync.
const TASK_PROVIDERS: [&str; 2] = ["gmail", "outlook"];

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
        ItemKind::Task => provider.is_some_and(|p| TASK_PROVIDERS.contains(&p.as_str())),
        // Notes are false for a reason per provider rather than one reason.
        // Google Keep's API is Workspace-only, so a consumer Gmail account
        // cannot use it at all. OneNote could carry them, and has not been
        // written: a OneNote page is an HTML document in a section in a
        // notebook rather than a title and a body, so the mapping is a
        // decision rather than an afternoon.
        //
        // Reminders stay false everywhere and always will. In Outlook and
        // Exchange a reminder is a property of an event or a task rather than
        // an item, and Google folded Reminders into Tasks in 2023, so there is
        // nothing on either side to sync a standalone reminder to.
        ItemKind::Reminder | ItemKind::Note => false,
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
        for kind in [ItemKind::Contact, ItemKind::Event, ItemKind::Task] {
            assert!(!supports(&plain, kind), "{kind:?} should not be offered");
        }
    }

    #[test]
    fn test_tasks_sync_on_both_providers_that_have_them() {
        // Google Tasks and Microsoft To Do, read into the local lists by
        // application::tasks_sync. This was false until that existed, which is
        // the rule: the flag and the sync behind it move together.
        for address in ["me@gmail.com", "me@outlook.com"] {
            assert!(
                supports(&account("a1", address), ItemKind::Task),
                "{address} should sync tasks"
            );
        }
    }

    #[test]
    fn test_notes_and_reminders_still_stay_on_this_computer() {
        // Notes for a reason per provider: Google Keep's API is Workspace-only
        // so a consumer account cannot use it, and OneNote could work but the
        // mapping from a page in a section in a notebook to a title and a body
        // is a decision nobody has made.
        //
        // Reminders because there is nothing on either side to sync one to. In
        // Outlook and Exchange a reminder is a property of an event or a task,
        // and Google folded Reminders into Tasks in 2023.
        for address in ["me@gmail.com", "me@outlook.com"] {
            let account = account("a1", address);
            for kind in [ItemKind::Note, ItemKind::Reminder] {
                assert!(
                    !supports(&account, kind),
                    "{address}: {kind:?} claimed to sync"
                );
            }
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
    fn test_a_note_or_reminder_is_local_even_on_a_gmail_default() {
        // A task is no longer in this list, which is the point of the sync
        // that was written for it. These two still are, and for reasons that
        // are not going to change: Google Keep's API is Workspace-only, and a
        // standalone reminder is not a thing either provider has.
        let accounts = vec![account("a1", "me@gmail.com")];

        for kind in [ItemKind::Note, ItemKind::Reminder] {
            assert_eq!(
                destination(kind, &accounts, Some("a1")),
                Some(Destination::Local),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn test_a_task_now_goes_to_the_account_that_can_hold_it() {
        // The other half of flipping the flag. A task made on a Gmail default
        // is filed under that account, so it sits in the same list the sync
        // fills rather than in a second list on this computer.
        let accounts = vec![account("a1", "me@gmail.com")];

        assert_eq!(
            destination(ItemKind::Task, &accounts, Some("a1")),
            Some(Destination::Account("a1".to_string()))
        );
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
    fn test_a_container_goes_where_the_things_it_holds_go() {
        // A calendar in one account holding events in another would be a
        // sidebar entry that never matches its list.
        let accounts = vec![account("a1", "me@myhost.example")];

        for container in ContainerKind::ALL {
            assert_eq!(
                destination(container.holds(), &accounts, Some("a1")),
                destination(container.holds(), &accounts, Some("a1")),
            );
        }

        // The mail-only account holds none of them, so all four are local.
        for container in ContainerKind::ALL {
            assert_eq!(
                destination(container.holds(), &accounts, Some("a1")),
                Some(Destination::Local),
                "{container:?}"
            );
        }
    }

    #[test]
    fn test_a_gmail_account_keeps_its_calendars_and_contact_groups() {
        let accounts = vec![account("a1", "me@gmail.com")];

        for container in [ContainerKind::Calendar, ContainerKind::ContactGroup] {
            assert_eq!(
                destination(container.holds(), &accounts, Some("a1")),
                Some(Destination::Account("a1".to_string())),
                "{container:?}"
            );
        }
    }

    #[test]
    fn test_every_container_is_named_for_a_menu() {
        for container in ContainerKind::ALL {
            let label = container.label();
            assert!(!label.is_empty(), "{container:?}");
            // Read aloud in a status line, so it is words rather than a
            // compressed identifier.
            assert!(!label.contains('_'), "{label} reads as an identifier");
        }
    }

    #[test]
    fn test_a_local_item_is_filed_under_the_reserved_account() {
        assert_eq!(Destination::Local.account_id(), "local");
        assert_eq!(Destination::Account("a1".to_string()).account_id(), "a1");
    }

    #[test]
    fn test_no_real_account_can_be_mistaken_for_the_local_one() {
        // An account whose id happened to be "local" would have its items
        // merged with everybody's local ones. Ids are generated, so this is a
        // guard on the generator rather than on user input.
        assert!(!LOCAL_ACCOUNT_ID.is_empty());
        assert!(
            !LOCAL_ACCOUNT_ID.contains('-'),
            "generated ids contain a dash"
        );
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

/// What deleting a container takes with it, said before it happens.
///
/// Named rather than counted where it can be: "Delete Shopping and the 12
/// tasks in it?" is a question somebody can answer, and "Are you sure?" is one
/// they learn to say yes to without reading.
///
/// The count matters most when it is large and when it is nought. Somebody who
/// thinks a list is empty and is told it holds forty things has just been
/// saved, and somebody told a list is empty can answer instantly.
pub fn deletion_question(kind: ContainerKind, name: &str, holding: usize) -> String {
    let held = kind.holds().plural();
    match holding {
        0 => format!(
            "Delete the {} \"{name}\"? It is empty.",
            kind.label().to_lowercase()
        ),
        1 => format!(
            "Delete the {} \"{name}\" and the 1 {} in it? This cannot be undone.",
            kind.label().to_lowercase(),
            kind.holds().label().to_lowercase()
        ),
        // Everything above one reads the same way, so the split is only
        // between none, one, and more than one.
        many => format!(
            "Delete the {} \"{name}\" and the {many} {} in it? This cannot be undone.",
            kind.label().to_lowercase(),
            held.to_lowercase()
        ),
    }
}

/// Whether deleting this container also removes it at the provider.
///
/// It does not, and saying so matters: somebody who deletes a synced list here
/// and watches it come back on the next sync will think the delete failed. It
/// deletes the local copy, and the next sync brings the provider's copy back.
pub fn deletion_reaches_provider(_kind: ContainerKind) -> bool {
    false
}

/// What to add to the question when the provider still has a copy.
pub const STILL_AT_THE_PROVIDER: &str = " It will come back at the next sync, because deleting it here does not \
     delete it at your provider yet.";

#[cfg(test)]
mod deletion_tests {
    use super::*;

    #[test]
    fn test_the_question_says_what_goes_with_it() {
        // "Are you sure?" is a question people learn to answer yes to without
        // reading. A number they did not expect is the thing that stops them.
        let asked = deletion_question(ContainerKind::TaskList, "Shopping", 12);

        assert!(asked.contains("Shopping"), "{asked}");
        assert!(asked.contains("12 tasks"), "{asked}");
        assert!(asked.contains("cannot be undone"), "{asked}");
    }

    #[test]
    fn test_an_empty_container_says_it_is_empty() {
        // Somebody told a list is empty can answer instantly, and somebody who
        // thought it was empty and is told it holds forty things has just been
        // saved. Both need the count to be accurate rather than omitted.
        let asked = deletion_question(ContainerKind::NoteFolder, "Old", 0);

        assert!(asked.contains("It is empty"), "{asked}");
        assert!(!asked.contains("cannot be undone"), "{asked}");
    }

    #[test]
    fn test_one_of_something_is_not_said_in_the_plural() {
        let asked = deletion_question(ContainerKind::Calendar, "Trips", 1);

        assert!(asked.contains("1 event in it"), "{asked}");
    }

    #[test]
    fn test_every_container_can_be_asked_about() {
        for kind in ContainerKind::ALL {
            let asked = deletion_question(kind, "Whatever", 3);
            assert!(
                asked.contains("Whatever") && asked.contains('3'),
                "{kind:?}: {asked}"
            );
        }
    }

    #[test]
    fn test_deleting_here_does_not_delete_it_at_the_provider_yet() {
        // Worth stating rather than leaving somebody to find out. Deleting a
        // synced list and watching it come back looks like the delete failed.
        for kind in ContainerKind::ALL {
            assert!(!deletion_reaches_provider(kind), "{kind:?}");
        }
        assert!(STILL_AT_THE_PROVIDER.contains("come back at the next sync"));
    }
}

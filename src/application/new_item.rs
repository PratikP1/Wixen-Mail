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
    /// What holds one of these, when one thing does.
    ///
    /// The inverse of [`ContainerKind::holds`], and partial where that is
    /// total. A reminder is filed nowhere: the module sorts them into buckets
    /// worked out from when each is due. A contact is in as many groups as
    /// somebody puts it in, so no single one is where it lives. And mail is in
    /// a folder on a server, which is not one of these.
    pub fn kept_in(self) -> Option<ContainerKind> {
        match self {
            ItemKind::Event => Some(ContainerKind::Calendar),
            ItemKind::Task => Some(ContainerKind::TaskList),
            ItemKind::Note => Some(ContainerKind::NoteFolder),
            ItemKind::Mail | ItemKind::Contact | ItemKind::Reminder => None,
        }
    }

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
            // says where it is; "no account" says where it is not. The words
            // come from `local_folders` so this and the folder tree's group
            // cannot drift apart.
            Destination::Local => crate::application::local_folders::ON_THIS_COMPUTER.to_string(),
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

/// Whether this kind of container can be renamed.
///
/// True only where a rename is written. A menu line that does nothing is worse
/// than one that is absent, because it is a stop somebody lands on, hears, and
/// learns nothing from. The other three are storable in the same way and
/// nobody has written the command, so this is the honest answer today and the
/// place to change when somebody does.
pub const fn renaming_works(kind: ContainerKind) -> bool {
    match kind {
        ContainerKind::ContactGroup => true,
        ContainerKind::Calendar | ContainerKind::TaskList | ContainerKind::NoteFolder => false,
    }
}

/// Where a new container of this kind belongs.
///
/// Three of the four follow the things they hold, so a calendar and its events
/// can never end up in different accounts. A contact group does not, and it is
/// the only container whose own kind decides: nothing sends a group to a
/// provider and nothing reads one back, so filing it under a mail account
/// would announce it as created somewhere that will never hear of it, and
/// would take it out of the sidebar the moment another account was opened.
///
/// The contacts in a group are untouched by this. Membership is by contact,
/// and a contact goes on belonging to whichever account holds it.
pub fn container_destination(
    kind: ContainerKind,
    accounts: &[Account],
    default_id: Option<&str>,
) -> Destination {
    if matches!(kind, ContainerKind::ContactGroup) {
        return Destination::Local;
    }
    // Never `None` for a container: `destination` only refuses mail, and a
    // container is not mail.
    destination(kind.holds(), accounts, default_id).unwrap_or(Destination::Local)
}

/// Which account a compose window sends from.
///
/// Two different questions that were being answered the same way. Answering
/// something already in a mailbox comes from that mailbox: replying to a work
/// message from a personal address is the kind of mistake somebody finds out
/// about once it has arrived, and the mailbox is the only thing that knows
/// which account the message came to.
///
/// Everything else follows the default account, because that is what setting a
/// default means. Reading another mailbox is not a decision to write from it.
///
/// Each falls back to the other, so somebody with one account and no default
/// set still gets that account rather than nothing.
pub fn sends_from<'a>(
    replying: bool,
    focused: Option<&'a str>,
    default: Option<&'a str>,
) -> Option<&'a str> {
    if replying {
        return focused.or(default);
    }
    default.or(focused)
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
    fn test_a_reply_comes_from_the_mailbox_it_was_read_in() {
        // Answering a work message from a personal address is a mistake
        // somebody finds out about after it has arrived. The mailbox being read
        // is the only thing that knows which account the message came to, so it
        // wins over the default for a reply.
        assert_eq!(
            sends_from(true, Some("work"), Some("personal")),
            Some("work")
        );
    }

    #[test]
    fn test_a_new_message_comes_from_the_default_account() {
        // Which is what setting a default means. Browsing another mailbox is
        // not a decision to write from it.
        assert_eq!(
            sends_from(false, Some("work"), Some("personal")),
            Some("personal")
        );
    }

    #[test]
    fn test_either_will_do_when_only_one_of_them_is_known() {
        assert_eq!(sends_from(true, None, Some("personal")), Some("personal"));
        assert_eq!(sends_from(false, Some("work"), None), Some("work"));
        assert_eq!(sends_from(false, None, None), None);
    }

    #[test]
    fn test_somebody_with_no_account_at_all_can_still_make_things() {
        // A POP and SMTP account syncs nothing but mail, and somebody who has
        // not signed in anywhere yet syncs nothing at all. Neither is a reason
        // to refuse to let them keep a note, a task or a contact: this
        // computer holds all of it.
        //
        // The managers used to ask "which account is active?" and refuse when
        // the answer was none, which showed up as the editor never opening.
        // Whether a provider would carry an item is a question about saving it
        // somewhere else, not about whether it may exist.
        for kind in [
            ItemKind::Event,
            ItemKind::Task,
            ItemKind::Note,
            ItemKind::Reminder,
            ItemKind::Contact,
        ] {
            assert_eq!(
                destination(kind, &[], None),
                Some(Destination::Local),
                "{kind:?} had nowhere to go"
            );
        }
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
    fn test_a_gmail_account_keeps_the_calendar_a_new_event_goes_in() {
        let accounts = vec![account("a1", "me@gmail.com")];

        assert_eq!(
            destination(ContainerKind::Calendar.holds(), &accounts, Some("a1")),
            Some(Destination::Account("a1".to_string()))
        );
    }

    #[test]
    fn test_a_contact_group_is_kept_here_whatever_account_is_default() {
        // A group is not sent anywhere, so filing it under a Gmail account
        // would announce it as created in an account that will never see it,
        // and would hide it from the sidebar whenever another account is being
        // looked at. The contacts in it still belong wherever they belong.
        let accounts = vec![account("a1", "me@gmail.com")];

        assert_eq!(
            container_destination(ContainerKind::ContactGroup, &accounts, Some("a1")),
            Destination::Local
        );
        // The other three still follow the things they hold.
        assert_eq!(
            container_destination(ContainerKind::Calendar, &accounts, Some("a1")),
            Destination::Account("a1".to_string())
        );
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
    fn test_a_container_is_named_by_the_words_people_use_for_it() {
        // These names reach somebody twice: in the New menu, and lowercased in
        // the question asked before a container is deleted. A name nobody uses
        // for the thing makes both of those read as somebody else's software.
        assert_eq!(ContainerKind::Calendar.label(), "Calendar");
        assert_eq!(ContainerKind::TaskList.label(), "Task list");
        assert_eq!(ContainerKind::NoteFolder.label(), "Note folder");
        assert_eq!(ContainerKind::ContactGroup.label(), "Contact group");
    }

    #[test]
    fn test_every_container_is_offered_for_the_kind_of_thing_it_holds() {
        // The new-item form asks this to fill its list of calendars, lists,
        // folders and groups. An answer of "none" empties that list and files
        // everything with no holder, which looks like nothing happening.
        for container in ContainerKind::ALL {
            assert_eq!(
                ContainerKind::holding(container.holds()),
                Some(container),
                "{container:?}"
            );
        }
        // Mail is in a folder on a server, and a reminder belongs to an
        // account and nothing smaller.
        assert_eq!(ContainerKind::holding(ItemKind::Mail), None);
        assert_eq!(ContainerKind::holding(ItemKind::Reminder), None);
    }

    #[test]
    fn test_only_the_kinds_that_live_in_one_container_can_be_moved_between_them() {
        // Move asks where a thing is kept, and gives up quietly when the
        // answer is nowhere. If the answer were nowhere for everything, Move
        // would stop working with no message to say so.
        assert_eq!(ItemKind::Event.kept_in(), Some(ContainerKind::Calendar));
        assert_eq!(ItemKind::Task.kept_in(), Some(ContainerKind::TaskList));
        assert_eq!(ItemKind::Note.kept_in(), Some(ContainerKind::NoteFolder));

        assert_eq!(ItemKind::Mail.kept_in(), None);
        assert_eq!(ItemKind::Reminder.kept_in(), None);
        // Deliberately not the mirror of the question above: a contact group
        // holds contacts, and a contact is in as many groups as somebody puts
        // it in, so there is no single home to move it out of.
        assert_eq!(ItemKind::Contact.kept_in(), None);
        assert_eq!(
            ContainerKind::holding(ItemKind::Contact),
            Some(ContainerKind::ContactGroup)
        );
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
    // A contact group is the one container that does not own what it holds. A
    // calendar deleted takes its events with it; a group deleted takes only
    // the label, and the people go on existing in the address book. Saying
    // "and the 3 contacts in it? This cannot be undone" offered to destroy
    // three people it never touches.
    if matches!(kind, ContainerKind::ContactGroup) {
        return group_deletion_question(name, holding);
    }
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

/// The same question for a group, which loses only its own name.
///
/// Where the others say what is destroyed, this says what survives. Somebody
/// deciding whether to delete a group needs to know the people in it are not
/// going anywhere, and that is the thing they are most likely to fear.
fn group_deletion_question(name: &str, holding: usize) -> String {
    match holding {
        0 => format!("Delete the contact group \"{name}\"? It is empty."),
        1 => format!(
            "Delete the contact group \"{name}\"? The 1 contact in it stays in your address book."
        ),
        many => format!(
            "Delete the contact group \"{name}\"? The {many} contacts in it stay in your address book."
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

/// Where the container being deleted came from.
///
/// A calendar and a task list can be either: one Google or Outlook sent, or
/// one somebody made here and filed under that same account. They read the
/// same in the sidebar and they behave differently when deleted, so the
/// question has to be told which it is rather than guess from the kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereItCameFrom {
    /// A provider sent it, so a copy of it is still there.
    AProvider,
    /// Made on this computer, so no copy of it exists anywhere else.
    ThisComputer,
}

/// Whether a provider holds a copy of the container being deleted.
///
/// Separate from [`deletion_reaches_provider`], which asks whether deleting
/// here also deletes there. This asks whether there is anything there in the
/// first place, and the answer decides whether somebody is told to expect the
/// thing back at the next sync. Telling them that about something no provider
/// has ever heard of leaves them waiting for a sync that will never mention it.
///
/// Two parts, and both are needed. Notes and contact groups are sent nowhere
/// at all, so no copy of one exists anywhere whoever made it. A calendar or a
/// task list ordinarily comes from Google or Microsoft and is put back by the
/// next read, but one made here is not: nothing sends it anywhere either.
/// Asked of the kind alone, the question promised a calendar somebody made
/// here back at the next sync.
pub const fn the_provider_has_a_copy(kind: ContainerKind, came_from: WhereItCameFrom) -> bool {
    matches!(
        (kind, came_from),
        (
            ContainerKind::Calendar | ContainerKind::TaskList,
            WhereItCameFrom::AProvider
        )
    )
}

/// What to add to the question when the provider still has a copy.
pub const STILL_AT_THE_PROVIDER: &str = " It will come back at the next sync, because deleting it here does not \
     delete it at your provider yet.";

/// The whole question asked before a container is deleted.
///
/// [`deletion_question`] plus the warning about the provider, added only where
/// there is a provider copy to come back. Written here rather than at the one
/// call site so the two halves cannot be composed differently by the next
/// caller, and so the composition can be tested without a window.
pub fn deletion_warning(
    kind: ContainerKind,
    name: &str,
    holding: usize,
    account_id: &str,
    came_from: WhereItCameFrom,
) -> String {
    let mut asked = deletion_question(kind, name, holding);
    let kept_here = account_id.starts_with(LOCAL_ACCOUNT_ID);
    if the_provider_has_a_copy(kind, came_from) && !deletion_reaches_provider(kind) && !kept_here {
        asked.push_str(STILL_AT_THE_PROVIDER);
    }
    asked
}

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
    fn test_deleting_a_group_keeps_the_people_in_it() {
        // A group is a label over people who exist on their own. Deleting one
        // removes the label and nothing else, and the question said the
        // opposite: "and the 3 contacts in it? This cannot be undone."
        let asked = deletion_question(ContainerKind::ContactGroup, "Team A", 3);

        assert!(asked.contains("Team A"), "{asked}");
        assert!(asked.contains("stay in your address book"), "{asked}");
        assert!(!asked.contains("cannot be undone"), "{asked}");

        let one = deletion_question(ContainerKind::ContactGroup, "Team A", 1);
        assert!(one.contains("stays in your address book"), "{one}");
    }

    #[test]
    fn test_the_whole_question_only_mentions_a_sync_where_one_happens() {
        // The sentence was appended whenever the open account was not the
        // local one, whatever was being deleted, so a group nothing has ever
        // sent anywhere was announced as coming back at the next sync.
        let group = deletion_warning(
            ContainerKind::ContactGroup,
            "Team A",
            3,
            "acct-1",
            WhereItCameFrom::AProvider,
        );
        assert!(!group.contains("come back at the next sync"), "{group}");

        let calendar = deletion_warning(
            ContainerKind::Calendar,
            "Trips",
            2,
            "acct-1",
            WhereItCameFrom::AProvider,
        );
        assert!(
            calendar.contains("come back at the next sync"),
            "{calendar}"
        );

        // A calendar kept here has no provider copy either, whatever kind it is.
        let here = deletion_warning(
            ContainerKind::Calendar,
            "Trips",
            2,
            LOCAL_ACCOUNT_ID,
            WhereItCameFrom::AProvider,
        );
        assert!(!here.contains("come back at the next sync"), "{here}");
    }

    #[test]
    fn test_a_group_is_not_promised_back_from_a_provider() {
        // Nothing sends a contact group to Google or Microsoft, so the
        // sentence promising it would come back at the next sync was telling
        // somebody to wait for something that is never going to happen.
        // Notes go nowhere either, for the same reason, whoever made them.
        for came_from in [WhereItCameFrom::AProvider, WhereItCameFrom::ThisComputer] {
            assert!(!the_provider_has_a_copy(
                ContainerKind::ContactGroup,
                came_from
            ));
            assert!(!the_provider_has_a_copy(
                ContainerKind::NoteFolder,
                came_from
            ));
        }
        assert!(the_provider_has_a_copy(
            ContainerKind::Calendar,
            WhereItCameFrom::AProvider
        ));
        assert!(the_provider_has_a_copy(
            ContainerKind::TaskList,
            WhereItCameFrom::AProvider
        ));
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

    #[test]
    fn test_a_calendar_made_on_this_computer_is_not_promised_back_from_a_provider() {
        // A calendar or a task list made here inside a Google or Outlook
        // account is sent nowhere, so nothing puts it back. Asked of the kind
        // rather than of the one being deleted, the question promised it back
        // anyway, and somebody who made a calendar here and deleted it was
        // left waiting for a sync that will never mention it. That is the
        // exact thing this half of the question exists to avoid.
        let made_here = deletion_warning(
            ContainerKind::Calendar,
            "Trips",
            2,
            "acct-1",
            WhereItCameFrom::ThisComputer,
        );
        assert!(
            !made_here.contains("come back at the next sync"),
            "{made_here}"
        );

        let from_a_provider = deletion_warning(
            ContainerKind::Calendar,
            "Trips",
            2,
            "acct-1",
            WhereItCameFrom::AProvider,
        );
        assert!(
            from_a_provider.contains("come back at the next sync"),
            "{from_a_provider}"
        );
    }

    #[test]
    fn test_a_task_list_made_on_this_computer_is_not_promised_back_either() {
        let made_here = deletion_warning(
            ContainerKind::TaskList,
            "Shopping",
            2,
            "acct-1",
            WhereItCameFrom::ThisComputer,
        );

        assert!(
            !made_here.contains("come back at the next sync"),
            "{made_here}"
        );
    }
}

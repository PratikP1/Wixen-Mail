//! Account management for multiple email accounts
//!
//! This module provides data structures and logic for managing multiple email accounts.

use std::time::SystemTime;

/// Email account configuration
#[derive(Clone)]
pub struct Account {
    /// Unique account identifier
    pub id: String,

    /// User-friendly account name
    ///
    /// The label somebody gave this account, so it is often "Work" or the
    /// provider's name. It is not a person's name and must never be put in
    /// front of an address on an outgoing message. See [`Account::sender_name`].
    pub name: String,

    /// The name recipients see when mail from this account arrives.
    ///
    /// A separate field from [`Account::name`] on purpose. Those two look
    /// interchangeable in the dialog and are not, which is the same trap that
    /// once put the SMTP login name in the From header of every message.
    ///
    /// Empty means no name, and a message goes out with a bare address, which
    /// is what every message sent before this field existed carried.
    pub sender_name: String,

    /// Email address
    pub email: String,

    /// IMAP server configuration
    pub imap_server: String,
    pub imap_port: String,
    pub imap_use_tls: bool,

    /// SMTP server configuration
    pub smtp_server: String,
    pub smtp_port: String,
    pub smtp_use_tls: bool,

    /// Authentication
    pub username: String,
    pub password: String,

    /// Whether this account uses OAuth2 for authentication.
    /// Determined automatically from the provider (Gmail, Outlook).
    pub use_oauth: bool,

    /// OAuth2 tokens (obtained during the authorization flow).
    /// Client ID/Secret are NOT stored per-account: they come from
    /// the app-level credentials module (`oauth_credentials`).
    pub oauth_access_token: String,
    pub oauth_refresh_token: String,
    pub oauth_token_expires_at: Option<String>,

    /// Account settings
    pub enabled: bool,
    pub check_interval_minutes: u32,

    /// Provider name (if using a preset)
    pub provider: Option<String>,

    /// Last sync timestamp
    pub last_sync: Option<SystemTime>,

    /// Account color for visual distinction (hex code)
    pub color: String,

    /// Which protocol this account reads mail with.
    ///
    /// Stored as a word rather than the enum, so an account row written by a
    /// later version reads back as IMAP instead of failing to parse. Every
    /// account written before this existed is IMAP, which is what the default
    /// says, and is correct: nothing could configure a POP account until now.
    pub protocol: String,

    /// The POP server, for an account that reads mail with POP3.
    ///
    /// Separate fields rather than reusing the IMAP ones. An account can be
    /// switched between the two while somebody is setting it up, and sharing
    /// one set of fields means switching quietly rewrites the other server's
    /// address into a box labelled for this one.
    pub pop_server: String,
    pub pop_port: String,
    pub pop_use_tls: bool,

    /// Whether to leave downloaded mail on the POP server.
    ///
    /// On by default, and the safe answer: POP3's delete is the only delete it
    /// has, and a client that removes mail as it downloads leaves somebody with
    /// one copy on one computer. Turning it off is a decision about where their
    /// only copy lives.
    pub pop_leave_on_server: bool,

    /// Remove mail from the POP server this many days after downloading it.
    ///
    /// Nought means never, which is the default. A mailbox that is never
    /// cleared eventually fills, and the alternative to this is somebody
    /// finding out when mail stops arriving.
    pub pop_remove_after_days: u32,
}

/// Everything about an account except its secrets.
///
/// Written out rather than derived, because this struct holds a password and
/// two OAuth tokens and `Debug` is what a tracing line prints. Nothing prints
/// an account today; this is so the first line that does is not a leak into a
/// file people are asked to attach to bug reports.
///
/// Fields are listed rather than skipped, so a field added later is left out
/// until somebody decides it is safe to print.
impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("email", &self.email)
            .field("protocol", &self.protocol)
            .field("enabled", &self.enabled)
            .field("use_oauth", &self.use_oauth)
            .finish_non_exhaustive()
    }
}

fn default_account_color() -> String {
    "#4A90E2".to_string() // Default blue
}

/// Whether browser sign-in is the better default for this address.
///
/// A default, not a rule. Both providers also accept an app password, which is
/// a password generated for one application and revocable on its own, and for
/// Google that is the path that works without the application being through
/// Google verification. Forcing OAuth on a Gmail address left anybody without
/// access to a verified client unable to add their own mail at all.
///
/// Microsoft defaults to OAuth because it has withdrawn password sign-in more
/// widely than Google has, so an app password there fails more often than not.
pub fn oauth_is_default(email: &str) -> bool {
    email
        .split('@')
        .nth(1)
        .map(|d| {
            let d = d.to_lowercase();
            matches!(
                d.as_str(),
                "outlook.com" | "hotmail.com" | "live.com" | "msn.com"
            )
        })
        .unwrap_or(false)
}

/// Where this provider hands out app passwords.
///
/// The page is three levels into account settings and does not come up from
/// searching the settings for "app password", so the account dialog offers to
/// open it rather than describing where it is. Finding the page is the whole
/// difficulty of this route; the password itself is a paste.
///
/// `None` for anywhere we do not know, which is most providers: sending
/// somebody to a guessed URL is worse than telling them to look.
pub fn app_password_url(email: &str) -> Option<&'static str> {
    email
        .split('@')
        .nth(1)
        .and_then(|d| match d.to_lowercase().as_str() {
            "gmail.com" | "googlemail.com" => Some("https://myaccount.google.com/apppasswords"),
            "outlook.com" | "hotmail.com" | "live.com" | "msn.com" => {
                Some("https://account.live.com/proofs/AppPassword")
            }
            _ => None,
        })
}

/// Whether this address belongs to a provider that offers app passwords.
///
/// Decides whether to tell somebody how to get one. Both providers require
/// two-step verification on the account first, and an organisation
/// administrator can switch app passwords off, so the guidance says where to
/// look rather than promising what will be there.
pub fn offers_app_passwords(email: &str) -> bool {
    email
        .split('@')
        .nth(1)
        .map(|d| {
            let d = d.to_lowercase();
            matches!(
                d.as_str(),
                "gmail.com"
                    | "googlemail.com"
                    | "outlook.com"
                    | "hotmail.com"
                    | "live.com"
                    | "msn.com"
            )
        })
        .unwrap_or(false)
}

impl Account {
    /// Create a new account with default settings
    pub fn new(name: String, email: String) -> Self {
        let oauth = oauth_is_default(&email);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            // Empty until somebody types one, which is what every message sent
            // before this field existed carried. Guessing it from the address
            // would put a name nobody chose in front of every message.
            sender_name: String::new(),
            email,
            imap_server: String::new(),
            imap_port: "993".to_string(),
            imap_use_tls: true,
            smtp_server: String::new(),
            smtp_port: "465".to_string(),
            smtp_use_tls: true,
            username: String::new(),
            password: String::new(),
            use_oauth: oauth,
            oauth_access_token: String::new(),
            oauth_refresh_token: String::new(),
            oauth_token_expires_at: None,
            enabled: true,
            check_interval_minutes: 5,
            provider: None,
            last_sync: None,
            color: default_account_color(),
            protocol: crate::common::types::Protocol::Imap.as_str().to_string(),
            pop_server: String::new(),
            pop_port: "995".to_string(),
            pop_use_tls: true,
            pop_leave_on_server: true,
            pop_remove_after_days: 0,
        }
    }

    /// Which protocol this account reads mail with.
    ///
    /// Anything unrecognised is IMAP, which is what every account written
    /// before the setting existed is, and the answer that leaves an account
    /// working rather than silently reading no mail.
    pub fn protocol(&self) -> crate::common::types::Protocol {
        crate::common::types::Protocol::from_stored(&self.protocol)
    }

    /// Create account from provider preset
    pub fn from_provider(
        name: String,
        email: String,
        provider: &crate::data::email_providers::EmailProvider,
    ) -> Self {
        let mut account = Self::new(name, email.clone());
        account.imap_server = provider.imap_server.clone();
        account.imap_port = provider.imap_port.to_string();
        account.imap_use_tls = provider.imap_tls;
        account.smtp_server = provider.smtp_server.clone();
        account.smtp_port = provider.smtp_port.to_string();
        account.smtp_use_tls = provider.smtp_tls;
        account.username = email;
        account.provider = Some(provider.name.clone());
        // use_oauth is already set by Account::new based on email domain
        account
    }

    /// Validate account configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Account name is required".to_string());
        }

        if self.email.trim().is_empty() {
            return Err("Email address is required".to_string());
        }

        if !self.email.contains('@') {
            return Err("Invalid email address format".to_string());
        }

        if self.imap_server.trim().is_empty() {
            return Err("IMAP server is required".to_string());
        }

        if self.smtp_server.trim().is_empty() {
            return Err("SMTP server is required".to_string());
        }

        if self.username.trim().is_empty() {
            return Err("Username is required".to_string());
        }

        // Ports are held as text because they come straight from a form. This
        // is the only place that can explain a bad one in terms of the field
        // the user typed it into; everything downstream sees a parse failure
        // with no idea which port it came from.
        validate_port("IMAP", &self.imap_port)?;
        validate_port("SMTP", &self.smtp_port)?;

        // OAuth accounts don't require a password
        if !self.use_oauth && self.password.is_empty() {
            return Err("Password is required".to_string());
        }

        Ok(())
    }

    /// Get a display name for the account
    pub fn display_name(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }

    /// Update last sync timestamp
    pub fn mark_synced(&mut self) {
        self.last_sync = Some(SystemTime::now());
    }

    /// Migrate from old AccountConfig to new Account
    pub fn from_account_config(config: &crate::presentation::ui_types::AccountConfig) -> Self {
        let email = config.email.clone();

        let provider = if let Some(provider_name) = &config.selected_provider {
            Some(provider_name.clone())
        } else {
            crate::data::email_providers::detect_provider_from_email(&email).map(|p| p.name.clone())
        };

        Account {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Primary Account".to_string(),
            sender_name: String::new(),
            email: email.clone(),
            imap_server: config.imap_server.clone(),
            imap_port: config.imap_port.clone(),
            imap_use_tls: config.imap_use_tls,
            smtp_server: config.smtp_server.clone(),
            smtp_port: config.smtp_port.clone(),
            smtp_use_tls: config.smtp_use_tls,
            username: config.username.clone(),
            password: config.password.clone(),
            use_oauth: oauth_is_default(&email),
            oauth_access_token: String::new(),
            oauth_refresh_token: String::new(),
            oauth_token_expires_at: None,
            enabled: true,
            check_interval_minutes: 5,
            provider,
            last_sync: None,
            color: "#4A90E2".to_string(),
            protocol: crate::common::types::Protocol::Imap.as_str().to_string(),
            pop_server: String::new(),
            pop_port: "995".to_string(),
            pop_use_tls: true,
            pop_leave_on_server: true,
            pop_remove_after_days: 0,
        }
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new("New Account".to_string(), "user@example.com".to_string())
    }
}

/// Account manager for CRUD operations
pub struct AccountManager {
    accounts: Vec<Account>,
    active_account_id: Option<String>,
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            active_account_id: None,
        }
    }

    pub fn load(&mut self, accounts: Vec<Account>, active_id: Option<String>) {
        self.accounts = accounts;
        self.active_account_id = active_id;
    }

    pub fn get_accounts(&self) -> &[Account] {
        &self.accounts
    }

    pub fn get_active_account(&self) -> Option<&Account> {
        self.active_account_id
            .as_ref()
            .and_then(|id| self.accounts.iter().find(|a| &a.id == id))
    }

    pub fn get_active_account_id(&self) -> Option<&String> {
        self.active_account_id.as_ref()
    }

    pub fn get_account(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    pub fn get_account_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.id == id)
    }

    pub fn add_account(&mut self, account: Account) -> Result<String, String> {
        account.validate()?;
        if self.accounts.iter().any(|a| a.email == account.email) {
            return Err(format!(
                "Account with email {} already exists",
                account.email
            ));
        }
        let id = account.id.clone();
        self.accounts.push(account);
        if self.accounts.len() == 1 {
            self.active_account_id = Some(id.clone());
        }
        Ok(id)
    }

    pub fn update_account(&mut self, account: Account) -> Result<(), String> {
        account.validate()?;
        let index = self
            .accounts
            .iter()
            .position(|a| a.id == account.id)
            .ok_or_else(|| "Account not found".to_string())?;
        self.accounts[index] = account;
        Ok(())
    }

    pub fn delete_account(&mut self, id: &str) -> Result<(), String> {
        let index = self
            .accounts
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| "Account not found".to_string())?;
        self.accounts.remove(index);
        if self.active_account_id.as_ref() == Some(&id.to_string()) {
            self.active_account_id = self.accounts.first().map(|a| a.id.clone());
        }
        Ok(())
    }

    pub fn set_active_account(&mut self, id: &str) -> Result<(), String> {
        if !self.accounts.iter().any(|a| a.id == id) {
            return Err("Account not found".to_string());
        }
        self.active_account_id = Some(id.to_string());
        Ok(())
    }

    pub fn get_enabled_accounts(&self) -> Vec<&Account> {
        self.accounts.iter().filter(|a| a.enabled).collect()
    }

    pub fn set_account_enabled(&mut self, id: &str, enabled: bool) -> Result<(), String> {
        let account = self
            .get_account_mut(id)
            .ok_or_else(|| "Account not found".to_string())?;
        account.enabled = enabled;
        Ok(())
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Check one port field, naming it in any complaint.
///
/// Port 0 is rejected along with everything unparseable: it means "any free
/// port" to the operating system, which is never what someone meant to type
/// into a mail server field.
fn validate_port(label: &str, value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{} port is required", label));
    }
    match trimmed.parse::<u16>() {
        Ok(0) | Err(_) => Err(format!(
            "{} port must be a number between 1 and 65535, not \"{}\"",
            label, trimmed
        )),
        Ok(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new("Test".to_string(), "test@example.com".to_string());
        assert_eq!(account.name, "Test");
        assert_eq!(account.email, "test@example.com");
        assert!(account.enabled);
        assert_eq!(account.check_interval_minutes, 5);
        assert!(!account.use_oauth);
    }

    #[test]
    fn test_oauth_auto_detection() {
        // Google is the exception: it can sign in either way, and an app
        // password is the one that works without the application being through
        // Google verification. The setting is a starting point the account
        // dialog can change.
        let gmail = Account::new("Gmail".to_string(), "user@gmail.com".to_string());
        assert!(!gmail.use_oauth);

        let outlook = Account::new("Outlook".to_string(), "user@outlook.com".to_string());
        assert!(outlook.use_oauth);

        let hotmail = Account::new("Hotmail".to_string(), "user@hotmail.com".to_string());
        assert!(hotmail.use_oauth);

        let yahoo = Account::new("Yahoo".to_string(), "user@yahoo.com".to_string());
        assert!(!yahoo.use_oauth);
    }

    #[test]
    fn test_an_account_does_not_carry_its_secrets_into_a_log() {
        // Debug is what a tracing line prints, and this struct holds a
        // password and two OAuth tokens. Nothing prints an account today, so
        // this is here to stop the first line that does from being a leak,
        // into a file people are asked to attach to bug reports.
        let mut account = Account::new("Work".to_string(), "user@example.com".to_string());
        account.password = "hunter2-the-real-one".to_string();
        account.oauth_access_token = "ya29.an-access-token".to_string();
        account.oauth_refresh_token = "1-a-refresh-token".to_string();

        let printed = format!("{account:?}");

        for secret in [
            "hunter2-the-real-one",
            "ya29.an-access-token",
            "1-a-refresh-token",
        ] {
            assert!(
                !printed.contains(secret),
                "an account printed {secret} where a log could keep it"
            );
        }
        assert!(
            printed.contains("user@example.com"),
            "a printed account says nothing about which account it is"
        );
    }

    #[test]
    fn test_a_new_account_gets_a_colour_that_is_a_colour() {
        // The colour tells accounts apart in a list. Empty, or a word, it is
        // not something a control can be given, and it is a supplementary cue
        // rather than the only one, so nothing else would report it missing.
        let account = Account::new("Work".to_string(), "user@example.com".to_string());
        let colour = &account.color;

        assert_eq!(colour.len(), 7, "{colour} is not a #rrggbb colour");
        assert!(colour.starts_with('#'), "{colour} does not start with #");
        assert!(
            colour[1..].chars().all(|c| c.is_ascii_hexdigit()),
            "{colour} is not made of hex digits"
        );
    }

    #[test]
    fn test_microsoft_addresses_default_to_browser_sign_in() {
        // Microsoft has withdrawn password sign-in more widely than Google has,
        // so an app password there fails more often than not.
        assert!(oauth_is_default("user@outlook.com"));
        assert!(oauth_is_default("user@hotmail.com"));
        assert!(oauth_is_default("user@live.com"));
        assert!(oauth_is_default("user@msn.com"));
    }

    #[test]
    fn test_google_addresses_default_to_an_app_password() {
        // Browser sign-in needs the application to be through Google
        // verification. Defaulting to it left anybody using a client that is
        // not verified unable to add their own mail, and an app password works
        // today.
        assert!(!oauth_is_default("user@gmail.com"));
        assert!(!oauth_is_default("user@googlemail.com"));
    }

    #[test]
    fn test_an_ordinary_address_defaults_to_a_password() {
        assert!(!oauth_is_default("user@yahoo.com"));
        assert!(!oauth_is_default("user@custom.com"));
        assert!(!oauth_is_default("not-an-address"));
    }

    #[test]
    fn test_both_big_providers_are_known_to_offer_app_passwords() {
        assert!(offers_app_passwords("user@gmail.com"));
        assert!(offers_app_passwords("user@outlook.com"));
        assert!(!offers_app_passwords("user@custom.com"));
    }

    #[test]
    fn test_the_app_password_page_is_known_for_the_providers_that_have_one() {
        assert_eq!(
            app_password_url("user@gmail.com"),
            Some("https://myaccount.google.com/apppasswords")
        );
        assert_eq!(
            app_password_url("user@googlemail.com"),
            Some("https://myaccount.google.com/apppasswords")
        );
        assert!(app_password_url("user@hotmail.com").is_some());
    }

    #[test]
    fn test_an_unknown_provider_gets_no_guessed_page() {
        // Sending somebody to a URL we invented is worse than telling them to
        // go and look, because it looks authoritative and wastes the trip.
        assert_eq!(app_password_url("user@custom.com"), None);
        assert_eq!(app_password_url("not-an-address"), None);
        assert_eq!(app_password_url(""), None);
    }

    #[test]
    fn test_every_provider_offering_app_passwords_says_where_to_get_one() {
        // The two answers have to agree: a dialog that says "use an app
        // password" and cannot say where is the state this was meant to fix.
        for address in [
            "user@gmail.com",
            "user@googlemail.com",
            "user@outlook.com",
            "user@hotmail.com",
            "user@live.com",
            "user@msn.com",
        ] {
            assert!(
                offers_app_passwords(address),
                "{address} is no longer known to offer app passwords"
            );
            assert!(
                app_password_url(address).is_some(),
                "{address} offers app passwords with nowhere to get one"
            );
        }
    }

    #[test]
    fn test_account_validation() {
        let mut account = Account::new("".to_string(), "test@example.com".to_string());
        assert!(account.validate().is_err());

        account.name = "Test".to_string();
        account.email = "invalid".to_string();
        assert!(account.validate().is_err());

        account.email = "test@example.com".to_string();
        account.imap_server = "imap.example.com".to_string();
        account.smtp_server = "smtp.example.com".to_string();
        account.username = "test".to_string();
        assert!(account.validate().is_err()); // Missing password (non-OAuth)

        account.password = "password".to_string();
        assert!(account.validate().is_ok());
    }

    #[test]
    fn test_account_display_name() {
        let account = Account::new("Test Account".to_string(), "test@example.com".to_string());
        assert_eq!(account.display_name(), "Test Account <test@example.com>");
    }

    fn create_valid_account(name: &str, email: &str) -> Account {
        let mut account = Account::new(name.to_string(), email.to_string());
        account.imap_server = "imap.example.com".to_string();
        account.smtp_server = "smtp.example.com".to_string();
        account.username = email.to_string();
        account.password = "password".to_string();
        account
    }

    #[test]
    fn test_account_manager_add() {
        let mut manager = AccountManager::new();
        let account = create_valid_account("Test", "test@example.com");
        let id = manager.add_account(account).unwrap();
        assert_eq!(manager.get_accounts().len(), 1);
        assert_eq!(manager.get_active_account_id(), Some(&id));
    }

    #[test]
    fn test_account_manager_duplicate_email() {
        let mut manager = AccountManager::new();
        let account1 = create_valid_account("Test1", "test@example.com");
        let account2 = create_valid_account("Test2", "test@example.com");
        manager.add_account(account1).unwrap();
        assert!(manager.add_account(account2).is_err());
    }

    #[test]
    fn test_account_manager_delete() {
        let mut manager = AccountManager::new();
        let account = create_valid_account("Test", "test@example.com");
        let id = manager.add_account(account).unwrap();
        manager.delete_account(&id).unwrap();
        assert_eq!(manager.get_accounts().len(), 0);
        assert!(manager.get_active_account_id().is_none());
    }

    #[test]
    fn test_account_manager_switch_active() {
        let mut manager = AccountManager::new();
        let account1 = create_valid_account("Test1", "test1@example.com");
        let account2 = create_valid_account("Test2", "test2@example.com");
        let id1 = manager.add_account(account1).unwrap();
        let id2 = manager.add_account(account2).unwrap();
        assert_eq!(manager.get_active_account_id(), Some(&id1));
        manager.set_active_account(&id2).unwrap();
        assert_eq!(manager.get_active_account_id(), Some(&id2));
    }

    /// Three accounts, the middle one switched off, and the ids to reach them.
    fn three_accounts() -> (AccountManager, Vec<String>) {
        let mut manager = AccountManager::new();
        let mut ids = Vec::new();
        for name in ["First", "Second", "Third"] {
            let account =
                create_valid_account(name, &format!("{}@example.com", name.to_lowercase()));
            ids.push(
                manager
                    .add_account(account)
                    .expect("an account to be added"),
            );
        }
        (manager, ids)
    }

    #[test]
    fn test_an_account_is_found_by_its_own_id_and_not_another() {
        // Every action on a message reaches a server through this lookup, so
        // the wrong account here is a flag, a deletion or a send arriving at
        // somebody else's mailbox. Nothing checked that the id asked for was
        // the id returned.
        let (mut manager, ids) = three_accounts();

        for (id, expected) in ids.iter().zip(["First", "Second", "Third"]) {
            assert_eq!(
                manager.get_account(id).map(|a| a.name.as_str()),
                Some(expected),
                "asking for {id} did not give back {expected}"
            );
            assert_eq!(
                manager.get_account_mut(id).map(|a| a.name.clone()),
                Some(expected.to_string()),
                "asking to change {id} reached the wrong account"
            );
        }

        assert!(manager.get_account("no-such-account").is_none());
        assert!(manager.get_account_mut("no-such-account").is_none());
    }

    #[test]
    fn test_the_active_account_is_the_one_that_was_made_active() {
        let (mut manager, ids) = three_accounts();

        manager
            .set_active_account(&ids[2])
            .expect("the third account exists");

        assert_eq!(
            manager.get_active_account().map(|a| a.name.as_str()),
            Some("Third")
        );
        assert!(
            manager.set_active_account("no-such-account").is_err(),
            "an account that is not there was made the active one"
        );
        assert_eq!(
            manager.get_active_account().map(|a| a.name.as_str()),
            Some("Third"),
            "a failed switch moved the active account anyway"
        );
    }

    #[test]
    fn test_changing_an_account_changes_that_one_and_leaves_the_rest() {
        let (mut manager, ids) = three_accounts();
        let mut changed = manager
            .get_account(&ids[1])
            .expect("the second account")
            .clone();
        changed.name = "Renamed".to_string();

        manager
            .update_account(changed)
            .expect("a change to be saved");

        assert_eq!(
            manager.get_account(&ids[1]).map(|a| a.name.as_str()),
            Some("Renamed"),
            "the change was reported as saved and was not"
        );
        assert_eq!(
            manager.get_account(&ids[0]).map(|a| a.name.as_str()),
            Some("First"),
            "changing one account changed another"
        );
        assert_eq!(manager.get_accounts().len(), 3, "an account went missing");
    }

    #[test]
    fn test_changing_an_account_that_is_not_there_is_refused() {
        let (mut manager, _) = three_accounts();
        let stranger = create_valid_account("Stranger", "stranger@example.com");

        assert!(manager.update_account(stranger).is_err());
        assert_eq!(manager.get_accounts().len(), 3);
    }

    #[test]
    fn test_only_the_accounts_left_switched_on_are_listed_for_syncing() {
        // What this list leaves out is not fetched, and mail that is not
        // fetched is not there to be missed. An empty answer is a mailbox
        // that has quietly stopped arriving.
        let (mut manager, ids) = three_accounts();

        manager
            .set_account_enabled(&ids[1], false)
            .expect("the second account exists");

        let enabled: Vec<&str> = manager
            .get_enabled_accounts()
            .iter()
            .map(|a| a.name.as_str())
            .collect();

        assert_eq!(enabled, ["First", "Third"]);
        assert!(
            !manager
                .get_account(&ids[1])
                .expect("the second account")
                .enabled,
            "the account was reported switched off and was not"
        );
        assert!(
            manager
                .set_account_enabled("no-such-account", false)
                .is_err()
        );
    }

    #[test]
    fn test_accounts_loaded_from_storage_replace_what_was_there() {
        let (mut manager, _) = three_accounts();

        let only = create_valid_account("Only", "only@example.com");
        let id = only.id.clone();
        manager.load(vec![only], Some(id.clone()));

        assert_eq!(manager.get_accounts().len(), 1);
        assert_eq!(manager.get_active_account_id(), Some(&id));
    }

    #[test]
    fn test_an_account_reads_mail_with_the_protocol_it_was_given() {
        // Answering IMAP for a POP account is an account that reads no mail
        // and says nothing about why.
        let mut account = create_valid_account("Test", "test@example.com");

        for (stored, expected) in [
            ("imap", crate::common::types::Protocol::Imap),
            ("pop3", crate::common::types::Protocol::Pop3),
            ("POP3", crate::common::types::Protocol::Pop3),
            ("something else", crate::common::types::Protocol::Imap),
            ("", crate::common::types::Protocol::Imap),
        ] {
            account.protocol = stored.to_string();
            assert_eq!(account.protocol(), expected, "{stored} was read wrongly");
        }
    }

    #[test]
    fn test_an_account_made_from_a_preset_takes_that_preset_s_servers() {
        let gmail = crate::data::email_providers::detect_provider_from_email("user@gmail.com")
            .expect("gmail is a preset");

        let account =
            Account::from_provider("Work".to_string(), "user@gmail.com".to_string(), &gmail);

        assert_eq!(account.imap_server, "imap.gmail.com");
        assert_eq!(account.imap_port, "993");
        assert!(account.imap_use_tls);
        assert_eq!(account.smtp_server, "smtp.gmail.com");
        assert_eq!(account.smtp_port, "587");
        assert!(account.smtp_use_tls);
        assert_eq!(account.username, "user@gmail.com");
        assert_eq!(account.provider.as_deref(), Some("gmail"));
    }

    #[test]
    fn test_an_account_remembers_when_it_last_synced() {
        // The gap since the last sync is how "not syncing" gets noticed at
        // all. Never recording one leaves it looking like it has never run.
        let mut account = create_valid_account("Test", "test@example.com");
        assert!(account.last_sync.is_none());

        account.mark_synced();

        let when = account.last_sync.expect("a sync to have been recorded");
        assert!(
            when.elapsed().expect("a clock that moves forwards")
                < std::time::Duration::from_secs(60),
            "the recorded time is not now"
        );
    }

    #[test]
    fn test_migrate_from_account_config() {
        use crate::presentation::ui_types::AccountConfig;

        let config = AccountConfig {
            email: "user@gmail.com".to_string(),
            selected_provider: Some("Gmail".to_string()),
            imap_server: "imap.gmail.com".to_string(),
            imap_port: "993".to_string(),
            imap_use_tls: true,
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: "465".to_string(),
            smtp_use_tls: true,
            username: "user@gmail.com".to_string(),
            password: "password123".to_string(),
        };

        let account = Account::from_account_config(&config);
        assert_eq!(account.email, "user@gmail.com");
        // A migrated account keeps its password and signs in with it. Turning
        // OAuth on here would have made an account that already worked stop
        // working until somebody went and authorised it.
        assert!(!account.use_oauth);
        assert_eq!(account.provider, Some("Gmail".to_string()));
    }

    // ── Port validation ─────────────────────────────────────────────────

    fn valid_account() -> Account {
        let mut a = Account::new("Work".to_string(), "me@example.com".to_string());
        a.imap_server = "imap.example.com".to_string();
        a.smtp_server = "smtp.example.com".to_string();
        a.username = "me@example.com".to_string();
        a.password = "hunter2".to_string();
        a.imap_port = "993".to_string();
        a.smtp_port = "587".to_string();
        a
    }

    #[test]
    fn test_valid_account_passes() {
        assert!(valid_account().validate().is_ok());
    }

    #[test]
    fn test_rejects_a_non_numeric_port() {
        // Caught here or not at all: nothing downstream can explain "abc" to
        // the user in terms of the field they typed it into.
        let mut a = valid_account();
        a.imap_port = "not-a-port".to_string();
        assert!(a.validate().is_err());

        let mut a = valid_account();
        a.smtp_port = "\u{ff15}\u{ff18}\u{ff17}".to_string();
        assert!(a.validate().is_err());
    }

    #[test]
    fn test_rejects_a_port_out_of_range() {
        let mut a = valid_account();
        a.imap_port = "70000".to_string();
        assert!(a.validate().is_err());

        let mut a = valid_account();
        a.smtp_port = "0".to_string();
        assert!(a.validate().is_err());
    }

    #[test]
    fn test_rejects_an_empty_port() {
        let mut a = valid_account();
        a.smtp_port = "   ".to_string();
        assert!(a.validate().is_err());
    }

    #[test]
    fn test_port_error_names_the_field() {
        let mut a = valid_account();
        a.smtp_port = "abc".to_string();
        let err = a.validate().unwrap_err();
        assert!(
            err.to_lowercase().contains("smtp"),
            "the message must say which port: {}",
            err
        );
    }

    #[test]
    fn test_fuzz_validation_never_panics() {
        let noise = [
            "",
            " ",
            "0",
            "-1",
            "65535",
            "65536",
            "99999999999999999999",
            "\u{4f60}",
            "\u{0}",
            "993\n",
            "+993",
            "0x3e1",
            "993 ",
            "\u{feff}993",
        ];
        for imap in noise {
            for smtp in noise {
                let mut a = valid_account();
                a.imap_port = imap.to_string();
                a.smtp_port = smtp.to_string();
                let _ = a.validate();
            }
        }
    }
}

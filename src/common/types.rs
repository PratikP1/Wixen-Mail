//! Common types used throughout the application

use std::fmt;

/// Unique identifier for various entities
pub type Id = String;

/// Email address type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress {
    pub address: String,
    pub name: Option<String>,
}

impl EmailAddress {
    /// Create a new email address
    pub fn new(address: String, name: Option<String>) -> Self {
        Self { address, name }
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) if needs_quoting(name) => write!(f, "{} <{}>", quoted(name), self.address),
            Some(name) => write!(f, "{} <{}>", name, self.address),
            None => write!(f, "{}", self.address),
        }
    }
}

/// Whether a display name has to be wrapped as an RFC 5322 quoted-string to
/// be written back out safely.
///
/// A bare comma or semicolon reads as the separator between two recipients on
/// the way back in, and a bare angle bracket reads as the start or end of the
/// address wrapper: "Babbage, Charles <charles@example.com>" and
/// "Bob <VIP> <bob@example.com>" both corrupt on the way back in without
/// this. A quote character has to be escaped rather than left bare for the
/// same reason, once anything downstream reads this text with a
/// quote-aware parser. A backslash is included because escaping exists for
/// exactly the two characters that would otherwise end a quoted-string
/// early: itself and the quote mark.
///
/// Triggered only by characters a real defect has been traced to, not by
/// every RFC 5322 special: nothing here is proven broken by, say, a period,
/// and quoting on one would change how very ordinary names like
/// "A. Lovelace" display everywhere this is shown.
fn needs_quoting(name: &str) -> bool {
    name.contains(['"', '\\', ',', ';', '<', '>'])
}

/// A name wrapped as an RFC 5322 quoted-string, with the two characters that
/// would otherwise end the quote early escaped.
fn quoted(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 2);
    out.push('"');
    for ch in name.chars() {
        if ch == '"' || ch == '\\' {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push('"');
    out
}

/// Mail protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Imap,
    Pop3,
}

impl Default for Protocol {
    /// IMAP, which is what every account written before this was stored is.
    fn default() -> Self {
        Protocol::Imap
    }
}

impl Protocol {
    /// How it stores itself, and reads back.
    pub const fn as_str(self) -> &'static str {
        match self {
            Protocol::Imap => "imap",
            Protocol::Pop3 => "pop3",
        }
    }

    /// Read a stored value back.
    ///
    /// Anything unrecognised is IMAP. An account file written by a later
    /// version, or edited by hand, should keep reading mail rather than
    /// quietly become an account that reads none.
    pub fn from_stored(stored: &str) -> Self {
        match stored.trim().to_ascii_lowercase().as_str() {
            "pop3" | "pop" => Protocol::Pop3,
            _ => Protocol::Imap,
        }
    }

    /// What it is called where somebody reads it.
    pub const fn spoken(self) -> &'static str {
        match self {
            Protocol::Imap => "IMAP, which keeps your mail on the server",
            Protocol::Pop3 => "POP3, which downloads your mail to this computer",
        }
    }

    /// Both, so a chooser and its tests cover the set.
    pub const ALL: [Protocol; 2] = [Protocol::Imap, Protocol::Pop3];
}

/// Folder types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderType {
    Inbox,
    Sent,
    Drafts,
    /// Mail written and not yet gone.
    ///
    /// Always on this computer, whatever the account is: a message that has not
    /// been sent is on no server by definition. It was a queue with a count and
    /// no way to see what was in it before this existed.
    Outbox,
    Trash,
    Spam,
    Archive,
    Custom,
}

impl FolderType {
    /// The spelling stored in the `folders.folder_type` column.
    ///
    /// Fixed, because it is written to a database that outlives the process.
    /// Renaming one of these strings would orphan every row already holding it.
    pub fn as_str(&self) -> &'static str {
        match self {
            FolderType::Inbox => "Inbox",
            FolderType::Sent => "Sent",
            FolderType::Drafts => "Drafts",
            FolderType::Outbox => "Outbox",
            FolderType::Trash => "Trash",
            FolderType::Spam => "Spam",
            FolderType::Archive => "Archive",
            FolderType::Custom => "Custom",
        }
    }

    /// Read a stored value back.
    ///
    /// Case-insensitive, because rows written before this conversion existed
    /// used whatever spelling the caller happened to pass. Anything
    /// unrecognised is an ordinary folder rather than an error: the folder is
    /// real and the user still needs to reach it.
    pub fn from_stored(stored: &str) -> Self {
        match stored.trim().to_ascii_lowercase().as_str() {
            "inbox" => FolderType::Inbox,
            "sent" => FolderType::Sent,
            "drafts" => FolderType::Drafts,
            "outbox" => FolderType::Outbox,
            "trash" => FolderType::Trash,
            "spam" | "junk" => FolderType::Spam,
            "archive" => FolderType::Archive,
            _ => FolderType::Custom,
        }
    }

    /// Where this folder sits in the tree, before names are compared.
    ///
    /// Mail clients have put the inbox first for thirty years, and a reader
    /// arrowing down a tree should not have to pass Archive and Drafts to reach
    /// it. Ordinary folders sort last, among themselves by name.
    pub fn tree_order(&self) -> u8 {
        match self {
            FolderType::Inbox => 0,
            FolderType::Drafts => 1,
            // Above Sent, because mail waiting to go is something to act on
            // and mail already gone is something to look up.
            FolderType::Outbox => 2,
            FolderType::Sent => 3,
            FolderType::Archive => 4,
            FolderType::Spam => 5,
            FolderType::Trash => 6,
            FolderType::Custom => 7,
        }
    }
}

/// Where a folder sits in the tree: what it is for, then its name.
///
/// One answer, because there were two. The tree read out of the cache sorted
/// with a database expression that had no place for mail waiting to go, so the
/// Outbox sat at the bottom among somebody's own folders while this said it
/// belongs above Sent. Callers pass their own name field, which really does
/// differ: a folder a server lists is announced by its display path and one
/// held on this computer by its name.
pub fn tree_position(kind: FolderType, name: &str) -> (u8, String) {
    (kind.tree_order(), name.to_lowercase())
}

/// How a stored day names "no year": the birthday storage format writes
/// "--03-14" rather than a real date when nobody gave one.
///
/// A domain and storage concern rather than a display one, which is why it
/// lives here rather than with the presentation layer's date formatting.
/// `application::contacts_sync` reads and writes it as part of the stored
/// format; `presentation::date_display` uses the same constant to recognise
/// that format when it turns a stored value into words.
pub const YEAR_LEFT_OUT: &str = "--";

/// The six integrated PIM modules available in Wixen Mail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PimModule {
    Mail,
    Contacts,
    Calendar,
    Reminders,
    Tasks,
    Notes,
}

impl PimModule {
    /// Every module, so a list of them is written once rather than per caller.
    ///
    /// In `index()` order, which is the order the panels are built and
    /// switched in. Anything walking all six should use this: hand-written
    /// copies of this list in construction order are what once put calendar
    /// where contacts should have been.
    pub const ALL: [PimModule; 6] = [
        PimModule::Mail,
        PimModule::Contacts,
        PimModule::Calendar,
        PimModule::Reminders,
        PimModule::Tasks,
        PimModule::Notes,
    ];

    /// Human-readable label with accelerator hint.
    pub fn label(&self) -> &'static str {
        match self {
            PimModule::Mail => "&Mail",
            PimModule::Contacts => "&Contacts",
            PimModule::Calendar => "Ca&lendar",
            PimModule::Reminders => "&Reminders",
            PimModule::Tasks => "&Tasks",
            PimModule::Notes => "&Notes",
        }
    }

    /// Keyboard shortcut display string.
    pub fn shortcut_hint(&self) -> &'static str {
        match self {
            PimModule::Mail => "Ctrl+Shift+1",
            PimModule::Contacts => "Ctrl+Shift+2",
            PimModule::Calendar => "Ctrl+Shift+3",
            PimModule::Reminders => "Ctrl+Shift+4",
            PimModule::Tasks => "Ctrl+Shift+5",
            PimModule::Notes => "Ctrl+Shift+6",
        }
    }

    /// Zero-based index (0..5).
    pub fn index(&self) -> usize {
        match self {
            PimModule::Mail => 0,
            PimModule::Contacts => 1,
            PimModule::Calendar => 2,
            PimModule::Reminders => 3,
            PimModule::Tasks => 4,
            PimModule::Notes => 5,
        }
    }

    /// All modules in order.
    pub fn all() -> &'static [PimModule] {
        &[
            PimModule::Mail,
            PimModule::Contacts,
            PimModule::Calendar,
            PimModule::Reminders,
            PimModule::Tasks,
            PimModule::Notes,
        ]
    }

    /// Module from zero-based index.
    pub fn from_index(idx: usize) -> Option<PimModule> {
        PimModule::all().get(idx).copied()
    }
}

/// Server configuration for email protocols
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub use_tls: bool,
    pub use_starttls: bool,
}

impl ServerConfig {
    /// Create a new server configuration
    pub fn new(host: String, port: u16, use_tls: bool) -> Self {
        Self {
            host,
            port,
            use_tls,
            use_starttls: false,
        }
    }
}

/// Account credentials (plaintext in memory; encrypted at persistence boundary by MessageCache)
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    /// Create new credentials
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

/// Account-specific settings
#[derive(Debug, Clone, Default)]
pub struct AccountSettings {
    pub check_interval_minutes: u32,
    pub signature: Option<String>,
    pub default_folder: Option<String>,
}

/// Message body types
///
/// Which MIME part a body came from is not recoverable from the string. "if a
/// < b and c > d" is prose that every markup test calls markup, and running a
/// sanitiser over it deletes the middle of the sentence. So the answer travels
/// with the value from the one place that knows it, and everything downstream
/// that has to escape, sanitise or wrap the body reads it here rather than
/// guessing again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageBody {
    Plain(String),
    Html(String),
    Multipart { plain: String, html: String },
}

impl Default for MessageBody {
    /// No body is empty text, not empty markup: nothing to sanitise.
    fn default() -> Self {
        Self::Plain(String::new())
    }
}

impl MessageBody {
    /// Get plain text representation
    pub fn as_plain(&self) -> &str {
        match self {
            MessageBody::Plain(text) => text,
            MessageBody::Html(_) => "",
            MessageBody::Multipart { plain, .. } => plain,
        }
    }

    /// Get HTML representation
    pub fn as_html(&self) -> Option<&str> {
        match self {
            MessageBody::Plain(_) => None,
            MessageBody::Html(html) => Some(html),
            MessageBody::Multipart { html, .. } => Some(html),
        }
    }
}

/// Email attachment
#[derive(Debug, Clone)]
pub struct Attachment {
    pub id: Id,
    pub filename: String,
    pub mime_type: String,
    pub size: usize,
    pub content_id: Option<String>,
}

impl Attachment {
    /// Create a new attachment
    pub fn new(filename: String, mime_type: String, size: usize) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            filename,
            mime_type,
            size,
            content_id: None,
        }
    }
}

/// Folder information
#[derive(Debug, Clone)]
pub struct Folder {
    pub id: Id,
    pub account_id: Id,
    pub name: String,
    pub path: String,
    pub parent_id: Option<Id>,
    pub folder_type: FolderType,
    pub unread_count: u32,
    pub total_count: u32,
}

impl Folder {
    /// Create a new folder
    pub fn new(account_id: Id, name: String, path: String, folder_type: FolderType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            account_id,
            name,
            path,
            parent_id: None,
            folder_type,
            unread_count: 0,
            total_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_protocol_reads_back_as_what_it_was_stored_as() {
        // Found by mutation testing: nothing checked what `as_str` produced,
        // so it could have written an empty string into every account row and
        // no test would have said so. This is a storage format, and the value
        // written today has to be the value read back by a version shipped
        // next year.
        for protocol in Protocol::ALL {
            assert_eq!(
                Protocol::from_stored(protocol.as_str()),
                protocol,
                "{protocol:?} did not survive being stored"
            );
            assert!(!protocol.as_str().is_empty());
        }
        assert_eq!(Protocol::Imap.as_str(), "imap");
        assert_eq!(Protocol::Pop3.as_str(), "pop3");
    }

    #[test]
    fn test_the_older_spelling_of_pop_is_still_read() {
        // Rows written before the conversion existed used whichever spelling
        // the caller happened to pass. Dropping this arm turns those accounts
        // into IMAP accounts, which then read no mail at all.
        assert_eq!(Protocol::from_stored("pop"), Protocol::Pop3);
        assert_eq!(Protocol::from_stored("POP3"), Protocol::Pop3);
        assert_eq!(Protocol::from_stored("  pop3  "), Protocol::Pop3);
    }

    #[test]
    fn test_an_unknown_protocol_still_reads_mail() {
        // An account written by a later version, or edited by hand, should
        // keep working rather than quietly become an account that reads none.
        assert_eq!(Protocol::from_stored("jmap"), Protocol::Imap);
        assert_eq!(Protocol::from_stored(""), Protocol::Imap);
    }

    #[test]
    fn test_each_protocol_is_described_differently_where_somebody_reads_it() {
        // Both said the same thing would make the chooser useless, and neither
        // saying anything would make it unreadable.
        assert!(Protocol::Imap.spoken().contains("server"));
        assert!(Protocol::Pop3.spoken().contains("this computer"));
        assert_ne!(Protocol::Imap.spoken(), Protocol::Pop3.spoken());
    }

    #[test]
    fn test_every_kind_of_folder_reads_back_as_itself() {
        // The Outbox arm was untested, which would have made every outbox an
        // ordinary folder: no queue, and mail waiting to go out filed where
        // nothing looks for it.
        for (stored, expected) in [
            ("inbox", FolderType::Inbox),
            ("sent", FolderType::Sent),
            ("drafts", FolderType::Drafts),
            ("outbox", FolderType::Outbox),
            ("trash", FolderType::Trash),
            ("spam", FolderType::Spam),
            ("junk", FolderType::Spam),
            ("archive", FolderType::Archive),
            ("Projects", FolderType::Custom),
        ] {
            assert_eq!(
                FolderType::from_stored(stored),
                expected,
                "{stored} read back wrongly"
            );
        }
    }

    #[test]
    fn test_a_folder_kind_is_read_however_it_was_spelled() {
        // Rows written before the conversion existed used whatever case the
        // caller passed.
        assert_eq!(FolderType::from_stored("INBOX"), FolderType::Inbox);
        assert_eq!(FolderType::from_stored(" Outbox "), FolderType::Outbox);
    }

    #[test]
    fn test_email_address_display() {
        let addr = EmailAddress::new(
            "test@example.com".to_string(),
            Some("Test User".to_string()),
        );
        assert_eq!(addr.to_string(), "Test User <test@example.com>");
    }

    #[test]
    fn test_email_address_no_name() {
        let addr = EmailAddress::new("test@example.com".to_string(), None);
        assert_eq!(addr.to_string(), "test@example.com");
    }

    #[test]
    fn test_a_display_name_with_a_comma_is_quoted_so_it_reads_back_as_one_person() {
        // "Babbage, Charles" is an ordinary directory-style name. Written back
        // out unquoted, the comma reads as the separator between two
        // recipients and the address is split away from the name that goes
        // with it.
        let addr = EmailAddress::new(
            "charles@example.com".to_string(),
            Some("Babbage, Charles".to_string()),
        );
        assert_eq!(
            addr.to_string(),
            "\"Babbage, Charles\" <charles@example.com>"
        );
    }

    #[test]
    fn test_a_display_name_with_an_angle_bracket_is_quoted_so_it_does_not_look_like_a_second_address()
     {
        // A display name containing a literal angle bracket is unusual but
        // legal, and mail_parser decodes one back to this exact text on the
        // way in. Written back out unquoted, the bracket reads as the start
        // of a second address wrapper.
        let addr = EmailAddress::new("bob@example.com".to_string(), Some("Bob <VIP>".to_string()));
        assert_eq!(addr.to_string(), "\"Bob <VIP>\" <bob@example.com>");
    }

    #[test]
    fn test_a_quote_mark_inside_a_display_name_is_escaped_so_the_quoting_stays_well_formed() {
        // A name is quoted because it contains a comma, an angle bracket, or
        // a quote mark; the quote mark case needs its own proof, because an
        // unescaped one would end the quoting early and leave the rest of
        // the name sitting outside it, unprotected. Built with the standard
        // library's own `replace` rather than a hand-typed literal: two
        // independent ways of doubling the mark, so a bug in one is not
        // hidden by the same bug in the other.
        let name = r#"Bob "The Machine" Smith"#;
        let addr = EmailAddress::new("bob@example.com".to_string(), Some(name.to_string()));
        let escaped = name.replace('"', "\\\"");
        assert_eq!(addr.to_string(), format!("\"{escaped}\" <bob@example.com>"));
    }

    #[test]
    fn test_a_backslash_inside_a_display_name_is_escaped_so_it_is_not_read_as_an_escape_itself() {
        // The other character quoting has to protect: a lone backslash left
        // unescaped inside the quotes would be read as introducing an escape
        // for whatever character follows it, silently swallowing that
        // character on the way back in.
        let name = r"C:\Temp\Bob";
        let addr = EmailAddress::new("bob@example.com".to_string(), Some(name.to_string()));
        let escaped = name.replace('\\', "\\\\");
        assert_eq!(addr.to_string(), format!("\"{escaped}\" <bob@example.com>"));
    }

    #[test]
    fn test_server_config() {
        let config = ServerConfig::new("imap.example.com".to_string(), 993, true);
        assert_eq!(config.host, "imap.example.com");
        assert_eq!(config.port, 993);
        assert!(config.use_tls);
        assert!(!config.use_starttls);
    }

    #[test]
    fn test_credentials() {
        let creds = Credentials::new("user@example.com".to_string(), "password".to_string());
        assert_eq!(creds.username, "user@example.com");
        assert_eq!(creds.password, "password");
    }

    #[test]
    fn test_message_body_plain() {
        let body = MessageBody::Plain("Hello World".to_string());
        assert_eq!(body.as_plain(), "Hello World");
        assert!(body.as_html().is_none());
    }

    #[test]
    fn test_message_body_html() {
        let body = MessageBody::Html("<p>Hello World</p>".to_string());
        assert_eq!(body.as_plain(), "");
        assert_eq!(body.as_html(), Some("<p>Hello World</p>"));
    }

    #[test]
    fn test_message_body_multipart() {
        let body = MessageBody::Multipart {
            plain: "Hello World".to_string(),
            html: "<p>Hello World</p>".to_string(),
        };
        assert_eq!(body.as_plain(), "Hello World");
        assert_eq!(body.as_html(), Some("<p>Hello World</p>"));
    }

    #[test]
    fn test_attachment_creation() {
        let attachment = Attachment::new(
            "document.pdf".to_string(),
            "application/pdf".to_string(),
            1024,
        );
        assert_eq!(attachment.filename, "document.pdf");
        assert_eq!(attachment.mime_type, "application/pdf");
        assert_eq!(attachment.size, 1024);
        assert!(attachment.content_id.is_none());
    }

    #[test]
    fn test_folder_creation() {
        let folder = Folder::new(
            "account-123".to_string(),
            "Inbox".to_string(),
            "INBOX".to_string(),
            FolderType::Inbox,
        );
        assert_eq!(folder.account_id, "account-123");
        assert_eq!(folder.name, "Inbox");
        assert_eq!(folder.folder_type, FolderType::Inbox);
        assert_eq!(folder.unread_count, 0);
        assert_eq!(folder.total_count, 0);
        assert!(folder.parent_id.is_none());
    }

    #[test]
    fn test_mail_waiting_to_go_sorts_above_mail_that_has_gone() {
        // One answer for where a folder sits, asked by the tree read out of the
        // cache and by the list a server gives. Mail that has not gone anywhere
        // yet is the one folder somebody has to act on, so it sits with the
        // drafts and above what has already gone.
        assert!(
            tree_position(FolderType::Outbox, "z") < tree_position(FolderType::Sent, "a"),
            "mail waiting to go sorted below mail that has gone"
        );
        assert!(
            tree_position(FolderType::Drafts, "z") < tree_position(FolderType::Outbox, "a"),
            "mail waiting to go sorted above the drafts"
        );
        // Same kind, so the name decides, and it decides without case.
        assert!(
            tree_position(FolderType::Custom, "apple") < tree_position(FolderType::Custom, "Zebra"),
            "two ordinary folders stopped sorting by name"
        );
    }

    #[test]
    fn test_every_folder_type_survives_a_trip_through_the_database() {
        for folder_type in [
            FolderType::Inbox,
            FolderType::Sent,
            FolderType::Drafts,
            FolderType::Outbox,
            FolderType::Trash,
            FolderType::Spam,
            FolderType::Archive,
            FolderType::Custom,
        ] {
            assert_eq!(
                FolderType::from_stored(folder_type.as_str()),
                folder_type,
                "{folder_type:?} did not survive"
            );
        }
    }

    #[test]
    fn test_rows_written_before_the_spelling_was_fixed_still_read() {
        // Callers passed whatever they liked, so the column holds both cases.
        assert_eq!(FolderType::from_stored("inbox"), FolderType::Inbox);
        assert_eq!(FolderType::from_stored("INBOX"), FolderType::Inbox);
        assert_eq!(FolderType::from_stored("  Sent  "), FolderType::Sent);
        // "Junk" is what a server calls it and "Spam" is what we store.
        assert_eq!(FolderType::from_stored("Junk"), FolderType::Spam);
    }

    #[test]
    fn test_an_unknown_stored_value_is_an_ordinary_folder() {
        // The folder is real and the user still has to reach it, so an
        // unreadable type must not turn into an error or a missing row.
        assert_eq!(FolderType::from_stored("Wombat"), FolderType::Custom);
        assert_eq!(FolderType::from_stored(""), FolderType::Custom);
    }

    #[test]
    fn test_the_inbox_comes_first_and_ordinary_folders_come_last() {
        // Somebody arrowing down the tree should not pass Archive and Drafts
        // to reach their mail.
        assert_eq!(FolderType::Inbox.tree_order(), 0);
        for other in [
            FolderType::Sent,
            FolderType::Drafts,
            FolderType::Outbox,
            FolderType::Trash,
            FolderType::Spam,
            FolderType::Archive,
            FolderType::Custom,
        ] {
            assert!(
                other.tree_order() > FolderType::Inbox.tree_order(),
                "{other:?} sorted above the inbox"
            );
            assert!(
                other.tree_order() <= FolderType::Custom.tree_order(),
                "{other:?} sorted below an ordinary folder"
            );
        }
    }

    #[test]
    fn test_pim_module_labels() {
        assert_eq!(PimModule::Mail.label(), "&Mail");
        assert_eq!(PimModule::Contacts.label(), "&Contacts");
        assert_eq!(PimModule::Calendar.label(), "Ca&lendar");
        assert_eq!(PimModule::Reminders.label(), "&Reminders");
        assert_eq!(PimModule::Tasks.label(), "&Tasks");
        assert_eq!(PimModule::Notes.label(), "&Notes");
    }

    #[test]
    fn test_pim_module_shortcut_hints() {
        assert_eq!(PimModule::Mail.shortcut_hint(), "Ctrl+Shift+1");
        assert_eq!(PimModule::Notes.shortcut_hint(), "Ctrl+Shift+6");
    }

    #[test]
    fn test_pim_module_index_roundtrip() {
        for module in PimModule::all() {
            let idx = module.index();
            assert_eq!(PimModule::from_index(idx), Some(*module));
        }
    }

    #[test]
    fn test_pim_module_all_count() {
        assert_eq!(PimModule::all().len(), 6);
    }

    #[test]
    fn test_pim_module_from_index_out_of_bounds() {
        assert_eq!(PimModule::from_index(6), None);
        assert_eq!(PimModule::from_index(100), None);
    }

    #[test]
    fn test_pim_module_indices_are_contiguous() {
        let modules = PimModule::all();
        for (i, module) in modules.iter().enumerate() {
            assert_eq!(module.index(), i);
        }
    }
}

# Wixen Mail - Architecture Overview

## Design Principles

1. **Accessibility First**: Every component designed with screen reader and keyboard navigation as primary considerations
2. **Modularity**: Clean separation of concerns with well-defined interfaces
3. **Performance**: Efficient resource usage and responsive UI
4. **Security**: Secure handling of credentials and email data
5. **Testability**: Comprehensive testing at all levels

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Presentation Layer                      │
│  ┌────────────────────────────────────────────────────────┐ │
│  │          WXDragon UI Components (Windows)              │ │
│  │  - Main Window  - Message List  - Composition Window   │ │
│  │  - Folder Tree  - Reading Pane  - Settings Dialog     │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │            Accessibility Layer                          │ │
│  │  - Screen Reader Bridge  - Keyboard Handler            │ │
│  │  - Focus Manager         - Announcement Queue          │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            ↕
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────────┐  │
│  │   Account    │ │   Message    │ │   Composition     │  │
│  │   Manager    │ │   Manager    │ │   Manager         │  │
│  └──────────────┘ └──────────────┘ └───────────────────┘  │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────────┐  │
│  │   Search     │ │   Filter     │ │   Contact         │  │
│  │   Engine     │ │   Engine     │ │   Manager         │  │
│  └──────────────┘ └──────────────┘ └───────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↕
┌─────────────────────────────────────────────────────────────┐
│                      Service Layer                           │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────────┐  │
│  │     IMAP     │ │     SMTP     │ │      POP3         │  │
│  │    Client    │ │    Client    │ │     Client        │  │
│  └──────────────┘ └──────────────┘ └───────────────────┘  │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────────┐  │
│  │   Security   │ │    Cache     │ │   Attachment      │  │
│  │   Service    │ │   Service    │ │   Handler         │  │
│  └──────────────┘ └──────────────┘ └───────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↕
┌─────────────────────────────────────────────────────────────┐
│                      Data Layer                              │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────────┐  │
│  │ MessageCache │ │  Credential  │ │   Configuration   │  │
│  │   (SQLite)   │ │    Store     │ │   Manager         │  │
│  └──────────────┘ └──────────────┘ └───────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Component Details

### Presentation Layer

#### WXDragon UI Integration
- **Purpose**: Native Windows UI using WXDragon library
- **Responsibilities**:
  - Render all UI components
  - Handle user input events
  - Manage window lifecycle
  - Provide native Windows look and feel

#### Accessibility Layer
- **Screen Reader Bridge**: 
  - Interfaces with Windows UI Automation (UIA)
  - Provides descriptive labels and roles for all UI elements
  - Manages live regions for dynamic content
  
- **Keyboard Handler**:
  - Centralized keyboard shortcut management
  - Shortcuts are fixed. `register_shortcut` is called once at startup and
    nothing in Settings reaches it, so there is no way to rebind a key.
  - Focus traversal management
  
- **Focus Manager**:
  - Tracks current focus location
  - Manages focus order
  - Handles focus trapping in dialogs
  
- **Announcement Queue**:
  - Queues screen reader announcements
  - Prioritizes urgent messages
  - Prevents announcement conflicts

### Application Layer

#### Account Manager
- **Responsibilities**:
  - Manage multiple email accounts
  - Store account credentials securely
  - Handle account authentication
  - Coordinate account synchronization

#### Message Manager
- **Responsibilities**:
  - CRUD operations for messages
  - Thread management
  - Message state (read/unread, flagged, etc.)
  - Folder organization

#### Composition Manager
- **Responsibilities**:
  - Draft creation and editing
  - Rich text and HTML composition
  - Recipient management
  - Attachment handling
  - Draft auto-save

#### Search Engine
- **Responsibilities**:
  - Full-text message search
  - Advanced filtering
  - Virtual folder creation
  - Search indexing

#### Filter Engine
- **Responsibilities**:
  - Rule-based message filtering
  - Spam detection
  - Automatic message organization
  - Custom filter creation

#### Contact Manager
- **Responsibilities**:
  - Address book management
  - Contact auto-completion
  - Contact groups
  - vCard import/export

### Service Layer

#### IMAP Client
- **Features**:
  - Full IMAP4rev1 support
  - IDLE extension for push notifications
  - Folder synchronization
  - Message fetching with caching
  - Search capabilities

#### SMTP Client
- **Features**:
  - Message sending with authentication
  - TLS/SSL support
  - Send queue management
  - Offline send queue
  - Delivery status tracking

#### POP3 Client
- **Features**:
  - Basic POP3 support
  - Leave messages on server option
  - Download and delete management

#### Security Service
- **Features**:
  - Credential encryption using Windows Data Protection API (DPAPI)
  - PGP/GPG integration
  - S/MIME support
  - Certificate management
  - Phishing detection

#### Cache Service
- **Features**:
  - Message body caching
  - Header caching
  - Attachment caching
  - Cache invalidation strategy
  - Size-based eviction

#### Attachment Handler
- **Features**:
  - Attachment download and save
  - Inline image handling
  - MIME type detection
  - Preview generation
  - Virus scanning integration points

### Data Layer

#### Database (SQLite)

One database, `cache\message_cache.db`, opened through `data::message_cache`.
Schema changes are additive: tables with `CREATE TABLE IF NOT EXISTS`, columns
with `ensure_column_exists`, and no dropping or renaming of anything that has
shipped. The one exception taken so far was a table of OAuth tokens that
nothing read, which was dropped because leaving secrets nobody rotates in a
file people copy is worse than the rule it broke.

- **Schema**:
  - Accounts table
  - Messages table (with FTS for search)
  - Folders table
  - Contacts table
  - Tags table
  - Message-Tag relations
  - Filters/Rules table
  - Configuration/Settings

#### Where files live

One root, owned by `common::paths`, so there is a single answer to "where is my
mail" for backups, for support, and for the uninstaller. `WIXEN_MAIL_DATA`
moves the root, which is what a memory stick install needs.

```
%LOCALAPPDATA%\wixen-mail├── config\                 settings, one file per account, oauth.toml
├── cache\                  message_cache.db and its SQLite sidecars
└── logs\                   the running log and crash.log
```

Nothing roams. An earlier layout put the encryption key in the roaming profile
while the database it unlocked stayed local, so the key crossed the network at
every sign-in and the mail it protected did not.

**No secrets are in the database.** Account passwords go to the Windows
credential store through `service::credentials`, OAuth tokens through
`service::oauth`, and CalDAV sign-ins through `service::caldav`. Each service
name has exactly one owner, because the code that erases them on uninstall has
to name the same entries as the code that wrote them.

**The cached mail is not encrypted**, and the documentation says so rather than
implying otherwise. Windows keeps other users out of the folder; anything
running as that user can read it, and so can anyone who takes the drive out of
an unencrypted machine. See [Installing and uninstalling](installing.md).

#### Configuration Manager
- **Settings Categories**:
  - Application preferences
  - Account settings
  - UI customization
  - Accessibility options
  - Privacy settings

## Technology Stack

### Core Technologies
- **Language**: Rust (stable channel)
- **UI Framework**: WXDragon (Windows-specific)
- **Database**: SQLite with rusqlite
- **Async Runtime**: tokio

### Key Dependencies (Planned)
- **Email Protocols**:
  - `async-imap` - IMAP client
  - `lettre` - SMTP client
  - `pop3` - POP3 client (if needed)
  
- **Parsing**:
  - `mail-parser` - Email parsing
  - `html5ever` - HTML parsing
  - `mime` - MIME type handling
  
- **Security**:
  - `ring` - Cryptographic operations
  - `sequoia-pgp` - PGP support
  - `rustls` - TLS implementation
  - `winapi` - Windows DPAPI access
  
- **Storage**:
  - `rusqlite` - SQLite bindings
  - `serde` / `serde_json` - Serialization
  
- **Accessibility**:
  - `windows` crate - Windows API bindings
  - UI Automation API integration
  
- **Utilities**:
  - `tokio` - Async runtime
  - `tracing` - Logging and diagnostics
  - `anyhow` - Error handling

## Data Models

### Account
```rust
struct Account {
    id: Uuid,
    name: String,
    email_address: String,
    protocol: Protocol, // IMAP, POP3
    incoming_server: ServerConfig,
    outgoing_server: ServerConfig,
    credentials: EncryptedCredentials,
    settings: AccountSettings,
}
```

### Message
```rust
struct Message {
    id: Uuid,
    account_id: Uuid,
    folder_id: Uuid,
    message_id: String, // RFC822 Message-ID
    subject: String,
    from: Vec<EmailAddress>,
    to: Vec<EmailAddress>,
    cc: Vec<EmailAddress>,
    bcc: Vec<EmailAddress>,
    date: DateTime<Utc>,
    body: MessageBody,
    attachments: Vec<Attachment>,
    flags: MessageFlags,
    tags: Vec<String>,
}
```

### Folder
```rust
struct Folder {
    id: Uuid,
    account_id: Uuid,
    name: String,
    path: String, // Full IMAP path
    parent_id: Option<Uuid>,
    folder_type: FolderType, // Inbox, Sent, Drafts, etc.
    unread_count: u32,
    total_count: u32,
}
```

## Threading Model

### Main UI Thread
- Handles all UI rendering and user input
- Must remain responsive at all times
- Offloads heavy work to background threads

### Background Workers
- **Sync Worker**: Handles email synchronization
- **Send Worker**: Manages outgoing email queue
- **Index Worker**: Updates search indexes
- **Cache Worker**: Manages cache maintenance

### Communication
- Use channels (tokio::sync::mpsc) for thread communication
- Event-driven architecture for UI updates
- Non-blocking operations wherever possible

## Error Handling

### Strategy
- Use `anyhow::Result` for application errors
- Use `thiserror` for library errors
- Comprehensive error context
- User-friendly error messages
- Screen reader-accessible error announcements

### Logging
- Structured logging with `tracing`
- Multiple log levels (Error, Warn, Info, Debug, Trace)
- Log file rotation
- Privacy-aware logging (no passwords or sensitive data)

## Security Considerations

### Credential storage

Everything goes to the Windows credential store, which is DPAPI-backed and
per-user. No credential is written to the database or to any file this
application owns, so a database copied for a backup or handed over for support
carries no credentials with it. The mail cached in that database is a separate
question, and data protection below gives the answer.

There is no master key any more. It encrypted exactly one thing, a password
that can be typed again, and the key itself was one more thing to lose.
Uninstalling clears the credential store entries by running the application
once as the user, because an uninstaller cannot reach them.

### Network Security
- Mandatory TLS for all connections
- Certificate validation
- Optional certificate pinning

### Data protection

- The cached mail is not encrypted at rest. Encrypting it means encrypting the
  whole database, which is a decision with a build cost rather than something
  to imply in a feature list.
- Uninstalling removes the data folder and the credential store entries, and
  writes what it could not remove to the temporary folder rather than claiming
  success.
- Nothing logs a token, a password, or a message body.

## Testing Strategy

### Unit Tests
- Test individual components in isolation
- Mock external dependencies
- High code coverage (target: 80%+)

### Integration Tests
- Test component interactions
- Use test email servers
- Database migration testing

### Accessibility Tests
- Automated accessibility checks
- Screen reader compatibility tests
- Keyboard navigation tests

### Performance Tests
- Startup time benchmarks
- Message loading performance
- Memory usage profiling
- Large mailbox handling

## Build and Deployment

### Build Configuration
- Debug build for development
- Release build with optimizations
- Profile-guided optimization for final release

### Distribution
- Windows installer (MSI/EXE)
- Portable version (ZIP)
- Auto-update mechanism (future)

## Future Considerations

### Extensibility
- Plugin API for extensions
- Custom theme support
- Script automation support

### Cross-Platform
- Abstract UI layer for future Linux/macOS support
- Platform-agnostic core components
- Conditional compilation for platform-specific features

## References

- [Rust Language](https://www.rust-lang.org/)
- [Windows UI Automation](https://docs.microsoft.com/en-us/windows/win32/winauto/entry-uiauto-win32)
- [IMAP RFC 3501](https://tools.ietf.org/html/rfc3501)
- [SMTP RFC 5321](https://tools.ietf.org/html/rfc5321)
- [Email Message Format RFC 5322](https://tools.ietf.org/html/rfc5322)

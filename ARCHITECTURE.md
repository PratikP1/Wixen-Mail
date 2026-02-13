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
│  │   Database   │ │  File System │ │   Configuration   │  │
│  │   (SQLite)   │ │   Storage    │ │   Manager         │  │
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
  - Custom key binding support
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
- **Schema**:
  - Accounts table
  - Messages table (with FTS for search)
  - Folders table
  - Contacts table
  - Tags table
  - Message-Tag relations
  - Filters/Rules table
  - Configuration/Settings

#### File System Storage
- **Structure**:
  ```
  %APPDATA%/WixenMail/
  ├── accounts/
  │   ├── [account-id]/
  │   │   ├── cache/
  │   │   │   ├── messages/
  │   │   │   └── attachments/
  │   │   └── config.json
  ├── database/
  │   └── wixen-mail.db
  ├── logs/
  │   └── wixen-mail.log
  └── config.json
  ```

#### Configuration Manager
- **Settings Categories**:
  - Application preferences
  - Account settings
  - UI customization
  - Keyboard shortcuts
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

### Credential Storage
- Windows DPAPI for credential encryption
- No plaintext passwords
- Secure memory handling

### Network Security
- Mandatory TLS for all connections
- Certificate validation
- Optional certificate pinning

### Data Protection
- Database encryption option
- Secure deletion of sensitive data
- Memory scrubbing for sensitive information

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

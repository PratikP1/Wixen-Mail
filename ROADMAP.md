# Wixen Mail - Project Roadmap

_Last updated: 2026-02-27_

## Vision
Wixen Mail aims to be a fully accessible, light-weight mail client built with Rust, providing a Thunderbird-like experience with first-class support for screen readers and keyboard navigation on Windows.

## Phase 1: Foundation (Complete)

### Project Setup
- [x] Initialize Rust project with Cargo
- [x] Set up Git repository structure
- [x] Create project documentation (README, LICENSE, ROADMAP)
- [x] Set up CI/CD pipeline (GitHub Actions)
- [x] Configure Rust formatting and linting tools (rustfmt, clippy)

### Core Architecture
- [x] Design modular architecture for mail client components
- [x] Define data models for emails, accounts, folders
- [x] Implement configuration management system
- [x] Create logging framework for debugging and diagnostics

### Accessibility Framework
- [x] Integrate egui + AccessKit for Windows UIA
- [x] Implement accessibility layer for screen reader support (NVDA, JAWS, Narrator)
- [x] Define comprehensive keyboard shortcuts system (25+)
- [ ] Create accessibility testing framework
- [x] Document accessibility features and keyboard commands

## Phase 2: Mail Protocol Support (Complete)

### IMAP Implementation
- [x] Implement IMAP4rev1 async protocol client
- [x] Support for IDLE (push notifications)
- [x] Folder synchronization
- [x] Message fetching and caching
- [x] Search functionality (SQLite FTS)

### SMTP Implementation
- [x] Implement SMTP client for sending emails (lettre)
- [x] Support for authentication (PLAIN, LOGIN)
- [x] Support for TLS/SSL/STARTTLS encryption
- [x] Outbox queue infrastructure for offline sending

### POP3 Support
- [x] Implement POP3 protocol client (full command surface)
- [x] Message downloading and deletion management

## Phase 3: User Interface (Complete)

### Main Window Layout
- [x] Design three-pane layout (folder tree, message list, message preview)
- [x] Implement resizable panes with keyboard controls
- [x] Create menu bar with full keyboard navigation
- [x] Context menus with quick actions

### Folder Management
- [x] Display folder tree with keyboard navigation
- [x] Folder hierarchy with metadata
- [x] Context menus for folder operations

### Message List View
- [x] Display message list with sortable columns
- [x] Thread view with conversation grouping
- [x] Unread/starred message indicators
- [x] Quick search/filter functionality

### Message Reading Pane
- [x] Plain text email rendering
- [x] HTML email rendering with sanitization (ammonia)
- [x] Plain text fallback for screen readers
- [x] Attachment display with metadata
- [x] Navigation between messages with keyboard

## Phase 4: Composition and Editing (Complete)

### Message Composition
- [x] Compose window with To/CC/BCC/Subject/Body
- [x] HTML and plain text modes with toggle
- [x] Formatting buttons (bold, italic, underline, link)
- [ ] Spell checking integration
- [x] Draft auto-save functionality
- [x] Email signatures (multiple per account)

### Contact Management
- [x] Full CRUD address book
- [x] Auto-completion for recipients
- [ ] Contact groups/distribution lists
- [x] Import/export contacts (vCard 3.0 format)
- [x] Search and filtering (fuzzy match)

### Attachments
- [x] Add/remove attachments with file picker
- [x] Attachment size warnings (>10MB)
- [x] MIME type detection
- [ ] Drag-and-drop insertion
- [ ] Inline image insertion

## Phase 5: Advanced Features (Complete)

### Search and Filtering
- [x] Full-text search across all folders (SQLite FTS)
- [x] Advanced search filters (date range, sender, recipient, attachments)
- [x] Unread-only / starred-only filters
- [x] Tag-based filtering
- [ ] Saved search folders (virtual folders)

### Message Organization
- [x] Tagging system
- [x] Message flags and markers (read, starred, deleted)
- [ ] Color coding
- [ ] Folder favorites
- [ ] Smart folders based on rules

### Email Rules and Filters
- [x] Message filtering engine with regex support
- [x] Rule-based actions (move, tag, mark spam)
- [ ] Spam filtering integration
- [x] Filter management UI

### Security Features
- [x] PGP signature detection and status display
- [x] S/MIME signature verification
- [x] Digital signature verification
- [x] Phishing detection with risk scoring
- [x] AES-256-GCM credential encryption
- [x] HTML sanitization (XSS protection)

## Phase 6: OAuth & Multi-Account (In Progress)

### OAuth 2.0 Authentication
- [x] Authorization flow UI
- [x] Provider-specific scopes (Gmail, Outlook)
- [x] Token refresh logic
- [x] Token persistence (SQLite)
- [ ] Real HTTP token exchange (currently mock stubs)
- [ ] Local callback server for OAuth redirect

### Multiple Account Support
- [x] Account management UI (add, update, delete, enable/disable)
- [x] Account switcher with "Set Active" button
- [x] Per-account data isolation
- [x] 5 provider presets with auto-detection
- [ ] Compose from specific account (dropdown selector)
- [ ] Unified inbox across accounts

## Phase 7: Offline Mode & Polish (Planned)

### Offline Mode
- [x] SQLite message/folder/draft caching
- [x] Outbox queue table with CRUD
- [ ] Offline mode UI toggle
- [ ] Queue flush to SMTP logic
- [ ] Sync status indicators
- [ ] Network status detection
- [ ] Conflict resolution

### Performance & Polish
- [ ] Virtual scrolling for large mailboxes
- [ ] Large mailbox testing (100K+ messages)
- [ ] Memory profiling and optimization
- [ ] Startup time optimization (<2 seconds)

### Testing & Quality
- [ ] Expand unit test coverage (currently 2 tests)
- [ ] Integration tests for protocols
- [ ] Accessibility compliance testing with screen readers
- [ ] Performance benchmarking

## Phase 8: Release Preparation (Planned)

### Packaging
- [ ] Windows installer (MSI or NSIS)
- [ ] Auto-update mechanism
- [ ] Desktop shortcuts

### Beta Testing
- [ ] Internal beta testing
- [ ] Public beta program (screen reader users)
- [ ] Bug tracking and triage

### Documentation
- [x] User guide with accessibility focus
- [x] Keyboard shortcuts reference
- [x] Provider setup guide
- [x] Troubleshooting guide
- [ ] Release notes and changelog

## Future Enhancements (Post 1.0)

### Additional Features
- [ ] Theme customization (dark mode, high contrast)
- [ ] Calendar integration (CalDAV)
- [ ] Preview before send
- [ ] Contact groups/distribution lists
- [ ] Exchange Web Services (EWS)
- [ ] Microsoft Graph API
- [ ] JMAP protocol
- [ ] Plugin/extension system

### Cross-Platform
- [ ] Linux support validation
- [ ] macOS support validation

## Success Metrics
- Fast startup time (< 2 seconds)
- Low memory footprint (< 100MB idle)
- 100% keyboard accessible
- WCAG 2.1 Level AA compliance
- Support for major screen readers (NVDA, JAWS, Narrator)

## Contributing
We welcome contributions! Please see CONTRIBUTING.md for guidelines.

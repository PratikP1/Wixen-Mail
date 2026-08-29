# Wixen Mail - Project Roadmap

> **Where the plan lives now.** The working plan is `.planning/ROADMAP.md`, kept
> by GSD. It holds the eight phases of outstanding work, each requirement traced
> to a phase, and acceptance criteria marked by whether a source stated them or a
> model derived them.
>
> This page is the shipped roadmap: what a person using Wixen Mail can see is
> built and what is coming. It records state, not the plan. When the two
> disagree, `.planning/ROADMAP.md` is right, and this page is the one to correct.
> Two documents answering one question is how this repository's documentation
> drifted before; keeping their jobs apart is the fix.

_Last updated: 2026-08-23_

## Vision
Wixen Mail aims to be a fully accessible, light-weight mail client built with Rust, providing a Thunderbird/Outlook-inspired experience with first-class support for screen readers and keyboard navigation on Windows.

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
- [x] Native Windows controls with built-in Windows UIA
- [x] Implement accessibility layer for screen reader support (NVDA, JAWS, Narrator)
- [x] Define comprehensive keyboard shortcuts system (25+)
- [x] Create accessibility testing framework. `.github/workflows/accessibility.yml`
      runs Axe.Windows and an MSAA name scan; `nvda.yml` drives a real copy of
      NVDA and checks what it said aloud. The wide manual pass with a screen
      reader is a separate thing and has not happened.
- [x] Document accessibility features and keyboard commands

## Phase 2: Mail Protocol Support (Complete)

### IMAP Implementation
- [x] Implement IMAP4rev1 async protocol client
- [x] Support for IDLE (push notifications)
- [x] Folder synchronization
- [x] Message fetching and caching
- [x] Search functionality (full-text search)
- [x] Read the server's capabilities once at sign-in and behave accordingly
- [x] SPECIAL-USE for folder roles, with the folder's name as the fallback
- [x] UIDPLUS, MOVE, CONDSTORE, ID, and Gmail's X-GM-EXT-1 where offered
- [x] STATUS for folder counts, LSUB and SUBSCRIBE for which folders sync
- [x] APPEND for the Sent copy, COPY and MOVE between folders
- [ ] QRESYNC, so a folder can resume rather than re-list its UIDs
- [ ] CREATE, RENAME and DELETE, so folders can be managed here
- [ ] Gmail's X-GM-THRID for conversations, and X-GM-RAW for server-side search.
      Both are blocked on the IMAP library rather than on this code

### SMTP Implementation
- [x] Implement SMTP client for sending emails
- [x] Support for authentication (PLAIN, LOGIN)
- [x] Support for TLS/SSL/STARTTLS encryption
- [x] Outbox queue infrastructure for offline sending

### POP3 Support
- [x] Implement POP3 protocol client. What was here before was a simulation: it
      opened no socket, ignored the password, and answered every command from
      three messages it made up. This one speaks RFC 1939 over TCP
- [x] TLS on 995, STLS on 110
- [x] Sync keyed on UIDL, so mail is not downloaded twice or skipped
- [x] Leave mail on the server, and remove it after a chosen number of days
- [x] Local folders, since POP3 has none of its own
- [x] Message downloading and deletion management

## Phase 3: User Interface (Complete)

### Main Window Layout
- [x] Design three-pane layout (folder tree, message list, message preview)
- [x] Implement resizable panes with keyboard controls
- [x] Create menu bar with full keyboard navigation
- [x] Main toolbar with stock icons (Get Mail, New, Reply, Forward, Delete, Search)
- [x] Context menus with quick actions

### Folder Management
- [x] Display folder tree with keyboard navigation
- [x] Folder hierarchy with metadata
- [x] Context menus for folder operations

### Message List View
- [x] Display message list with sortable columns
- [ ] Thread view with conversation grouping. The Thread View item is on the
      View menu and disabled: the data model carries the identifiers, and
      nothing groups the list by them. Opening one conversation works. Press
      Enter on a message to see the messages around it as a tree.
- [x] Unread/starred message indicators
- [x] Quick search/filter functionality

### Message Reading Pane
- [x] Plain text email rendering
- [x] HTML email rendering with anything that could run stripped out
- [x] Plain text fallback for screen readers
- [x] Attachment display with metadata
- [x] Navigation between messages with keyboard

### Visual Styling
- [x] Folder tree sidebar tint for visual separation
- [x] Readable fonts (Swiss for lists, Roman for preview)
- [x] Three-field status bar

## Phase 4: Composition and Editing (Complete)

### Message Composition
- [x] Compose window with To/CC/BCC/Subject/Body
- [x] HTML and plain text modes with toggle
- [x] Compose toolbar (Send, Undo, Redo, Bold, Italic, Underline, Attach)
- [~] Spell checking. Windows' own checker on Windows, so it knows the words you have added in Windows Settings; the built-in list elsewhere. Messages are checked when you send one. Checking while you type, jumping between misspellings and the screen reader announcing them natively all wait on the rich editor, since they need a control that can carry the marks.
- [x] Draft auto-save functionality
- [x] Email signatures (multiple per account)
- [x] Preview-before-send confirmation dialog

### Contact Management
- [x] Full CRUD address book
- [x] Auto-completion for recipients
- [x] Contact groups / distribution lists
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
- [x] Full-text search across all folders
- [x] Advanced search filters (date range, sender, recipient, attachments)
- [x] Unread-only / starred-only filters
- [x] Tag-based filtering
- [x] Saved search folders (virtual folders)

### Message Organization
- [x] Tagging system
- [x] Message flags and markers (read, starred, deleted)
- [x] Color coding
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
- [x] Phishing detection with risk scoring
- [x] Credentials in the Windows credential store, never in a file we own
- [x] HTML sanitization (XSS protection)

## Phase 6: OAuth & Multi-Account (Complete)

### OAuth 2.0 Authentication
- [x] Authorization flow UI
- [x] Provider-specific scopes (Gmail, Outlook)
- [x] Real HTTP token exchange
- [x] Token refresh logic
- [x] Token persistence
- [x] Local callback server for OAuth redirect. `src/service/oauth.rs` serves
      `http://localhost:<port>/oauth/callback` and checks the CSRF state

### Multiple Account Support
- [x] Account management UI (add, update, delete, enable/disable)
- [x] Account switcher with "Set Active" button
- [x] Per-account data isolation
- [x] 5 provider presets with auto-detection
- [x] Compose from specific account (dropdown selector)
- [x] Unified inbox across accounts ("All Inboxes")

## Phase 7: Offline Mode & Polish (Complete)

### Offline Mode
- [x] Message, folder and draft caching on this computer
- [x] Outbox queue table with CRUD
- [x] Offline mode UI toggle (View menu)
- [x] Queue flush to SMTP on reconnect
- [x] Outbox count and sync status indicators
- [ ] Network status detection (auto-toggle)
- [ ] Conflict resolution

### Testing & Quality
- [x] 150 unit tests across all modules
- [x] 26 integration tests
- [ ] Accessibility compliance testing with screen readers
- [ ] Performance benchmarking

## Phase 8: Release Preparation (Planned)

### Packaging
- [x] Windows installer, built with Inno Setup rather than MSI or NSIS
- [ ] Auto-update mechanism
- [x] Desktop and Start menu shortcuts. The installer creates the Start menu entry and offers
      the desktop one as a task. Neither carries the application icon yet: that is one
      `IconFilename` on each `[Icons]` entry, and it is SHIP-03 in the plan.

### Performance
- [x] Virtual scrolling, for the message list and for every other list in the application
- [ ] Large mailbox testing (100K+ messages)
- [ ] Memory profiling and optimization
- [ ] Startup time optimization (<2 seconds)

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
- [x] Theme customization (light, dark, and Windows high contrast)
- [~] Calendar integration (CalDAV). The client exists, signs in, and a calendar
      can be added by its own address (`src/presentation/wx_add_calendar.rs`).
      It has never been run against a real calendar server
- [ ] Full PGP/S-MIME encryption and decryption
- [ ] Exchange Web Services (EWS)
- [x] Microsoft Graph API, for Outlook and Microsoft 365 contacts and calendars
- [ ] JMAP protocol
- [ ] Plugin/extension system

### Cross-Platform
- [ ] Linux support validation
- [ ] macOS support validation

## Success Metrics
- Fast startup time (< 2 seconds)
- Low memory footprint (< 100MB idle)
- 100% keyboard accessible
- WCAG 2.2 Level AA compliance
- Support for major screen readers (NVDA, JAWS, Narrator)

## Contributing
We welcome contributions! Please see [how to contribute](contributing.md).

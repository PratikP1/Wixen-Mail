# Wixen Mail Implementation Status

_Last updated: 2026-03-01_

This file is the canonical project-status reference.

## Summary

Wixen Mail is a fully accessible email client built in Rust with a native wxWidgets UI via wxdragon 0.9.12, targeting WCAG 2.1 Level AA compliance. All v1.0 feature gaps have been closed — the project is release-candidate ready, pending only validation and manual testing.

## What Is Complete

### Core Infrastructure
- Native wxWidgets UI via wxdragon 0.9.12 (three-pane layout, toolbars, dialogs, menus)
- Main toolbar with stock icons (Get Mail, New, Reply, Reply All, Forward, Delete, Mark Read, Search)
- Compose toolbar (Send, Undo, Redo, Bold, Italic, Underline, Attach)
- Visual styling (folder tree sidebar tint, readable fonts, 3-field status bar)
- Four-layer modular architecture (Presentation / Application / Service / Data)
- Structured logging with privacy-aware masking
- JSON-based configuration management
- CI/CD pipeline (GitHub Actions: test, fmt, clippy, build)

### Protocols
- IMAP4rev1 async client with IDLE push notifications
- SMTP client (lettre) with TLS, STARTTLS, HTML support
- POP3 full command-surface implementation
- Connection management with retry logic

### Multi-Account
- Account CRUD (add, update, delete, enable/disable)
- Account switching UI with "Set Active" button
- Per-account data isolation (folders, messages, cache)
- 5 provider presets (Gmail, Outlook, Yahoo, iCloud, ProtonMail)

### Composition & Sending
- Compose window with To/CC/BCC/Subject/Body
- Attachment management (file picker, add/remove, MIME types, size warnings)
- HTML/plain text toggle with formatting buttons (bold, italic, underline, link)
- Draft auto-save to SQLite
- Email signatures (multiple per account, auto-insert)
- Contact autocomplete in recipient fields

### Contact Management
- Full CRUD (create, read, update, delete)
- Search & filtering (fuzzy match across name, email, company, phone)
- 6 sort options (name, email, favorites, recent, company, last contacted)
- vCard 3.0 import/export with full spec compliance
- Auto-import from message history
- 14-field contact schema with avatar support

### Search & Filtering
- Full-text search with SQLite FTS indexing
- Date range filtering
- Sender/recipient filters
- Has-attachments / unread-only / starred-only filters
- Tag-based filtering
- Message rules engine with regex support and actions (move, tag, mark spam)

### Security
- AES-256-GCM credential encryption
- HTML sanitization (ammonia) for safe rendering
- PGP/S-MIME signature detection
- Phishing risk scoring with local signal detection
- Secure credential masking in logs
- TLS/STARTTLS for all connections

### OAuth 2.0 (Complete, Integrated with Account Creation)
- Authorization flow UI (provider selection, URL generation, code input)
- Provider-specific scopes configured (Gmail, Outlook)
- Real HTTP token exchange via `reqwest` (Google & Microsoft token endpoints)
- Token refresh logic with real HTTP calls
- Token persistence (SQLite `oauth_tokens` table)
- Account-to-provider auto-detection
- Auto-enable OAuth for Gmail/Outlook during account creation
- `use_oauth` field on Account model; password optional for OAuth accounts

### Composition & Sending (Extended)
- Preview-before-send dialog (shows rendered email for confirmation before sending)
- Preview is configurable via `preview_before_send` in AppConfig (default: on)
- Spell checking with multi-language support (built-in English, extensible)
- Spellbook (Hunspell-compatible, pure Rust) backend with automatic dictionary discovery

### Settings / Preferences Dialog
- Tabbed dialog accessible from Tools > Settings (Ctrl+,)
- General tab: theme, font size, notifications, update checking
- Compose tab: preview-before-send, default format, draft auto-save, signatures
- Reading tab: default sort order, threaded view, mark-as-read delay, remote images
- Language tab: interface language selection, spell check toggle, Hunspell info
- Advanced tab: log level, download folder with browse dialog, cache info
- Settings persisted to JSON via ConfigManager

### Message Sorting
- View menu sort submenu with 7 sort options (date, sender, subject, unread)
- In-memory sort applied instantly to current message list
- Sort order persisted in UI state

### Internationalization (i18n) Foundation
- Multi-language spell checker with language-specific alphabets (en, es, fr, de, pt, it)
- I18n registry for UI string translation with fallback chain
- Locale system with RTL/LTR text direction detection
- JSON-based translation file loading
- All core UI strings registered as translatable keys

### Accessible HTML Rendering
- Message preview uses RichTextCtrl (accessible to screen readers via UIA bridge)
- HTML-to-accessible-text renderer with inline link annotations
- Image alt text and link summary appended for screen readers

### Contact Groups / Distribution Lists
- Group CRUD (create, rename, delete)
- Add/remove contacts from groups
- Resolve group to comma-separated email list for compose
- SQLite-backed persistence (contact_groups + contact_group_members tables)

### Offline Mode (Fully Wired)
- SQLite message/folder/draft caching
- Outbox queue table with CRUD operations
- IMAP IDLE push event plumbing
- UI toggle (View menu checkbox) for offline/online mode
- Queue-flush-to-SMTP on reconnect
- Outbox queue count and sync status indicators

### Accessibility
- Native wxWidgets accessibility (Windows UIA, NVDA, JAWS, Narrator)
- 25+ customizable keyboard shortcuts
- Screen reader announcements via Accessibility module
- Focus management and modal dialog trapping
- Clear focus indicators and sufficient color contrast

### Documentation
- USER_GUIDE.md, KEYBOARD_SHORTCUTS.md, PROVIDER_SETUP.md
- TROUBLESHOOTING.md (30+ issues), ACCESSIBILITY.md
- ARCHITECTURE.md, CONTRIBUTING.md

### Test Coverage
- 150 unit tests across all modules
- 26 integration tests (accounts, contacts, groups, messages, filters, search, security, spell check, OAuth, cache, outbox)

## Remaining Work

All v1.0 feature gaps have been closed. The following are post-v1.0 enhancements:

### Nice-to-Have (Post v1.0)
- Theme customization (dark mode, high contrast)
- Calendar integration (CalDAV)
- Windows installer (MSI/NSIS)
- Large mailbox performance validation (100K+ messages)
- Virtual scrolling for large message lists
- Plugin/extension system

## Validation Snapshot

| Check | Status |
|-------|--------|
| `cargo build --quiet` | passes |
| `cargo test --quiet` | passes (150 unit + 26 integration) |
| `cargo fmt --all -- --check` | passes |
| `cargo clippy --all-targets` | passes (0 warnings) |

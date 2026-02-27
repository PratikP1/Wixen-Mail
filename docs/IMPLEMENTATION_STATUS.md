# Wixen Mail Implementation Status

_Last updated: 2026-02-27_

This file is the canonical project-status reference.

## Summary

Wixen Mail is a fully accessible email client built in Rust (egui + AccessKit) targeting WCAG 2.1 Level AA compliance. The project has completed all core feature development and is in release-hardening stage.

## What Is Complete

### Core Infrastructure
- Accessibility-first integrated UI (egui + AccessKit)
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

### OAuth 2.0 (Partial)
- Authorization flow UI (provider selection, URL generation, code input)
- Provider-specific scopes configured (Gmail, Outlook)
- Token refresh logic
- Token persistence (SQLite `oauth_tokens` table)
- Account-to-provider auto-detection

### Offline Infrastructure
- SQLite message/folder/draft caching
- Outbox queue table with CRUD operations
- IMAP IDLE push event plumbing

### Accessibility
- Windows UIA via AccessKit (NVDA, JAWS, Narrator)
- 25+ customizable keyboard shortcuts
- Live regions for dynamic updates
- Focus management and dialog trapping
- Clear focus indicators and sufficient color contrast

### Documentation
- USER_GUIDE.md, KEYBOARD_SHORTCUTS.md, PROVIDER_SETUP.md
- TROUBLESHOOTING.md (30+ issues), ACCESSIBILITY.md
- ARCHITECTURE.md, CONTRIBUTING.md

## Remaining Work

### Small Gaps
1. **OAuth HTTP token exchange** - `exchange_code()` and `refresh_access_token()` return mock tokens; need real `reqwest` HTTP calls to Google/Microsoft token endpoints.
2. **Compose account selector** - Composition uses the active account; needs a dropdown to select send-from account when multiple accounts are configured.
3. **Contact groups / distribution lists** - Not yet implemented.
4. **Preview before send** - Not yet implemented.

### Medium Gaps
5. **Offline mode wiring** - Queue infrastructure exists (SQLite table, CRUD) but the UI toggle, queue-flush-to-SMTP logic, and sync status indicators are not connected.
6. **Spell check** - No spell-checking integration exists yet (requires external library).
7. **Test coverage** - Only 2 automated tests exist. Need significantly more unit and integration tests.

### Nice-to-Have (Post v1.0)
- Theme customization (dark mode, high contrast)
- Calendar integration (CalDAV)
- Windows installer (MSI/NSIS)
- Large mailbox performance validation (100K+ messages)

## Validation Snapshot

| Check | Status |
|-------|--------|
| `cargo build --quiet` | passes |
| `cargo test --quiet` | passes (2/2) |
| `cargo fmt --all -- --check` | passes |
| `cargo clippy --all-targets` | passes (0 warnings) |

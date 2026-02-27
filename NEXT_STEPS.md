# Wixen Mail - Next Steps and v1.0 Roadmap

**Date:** 2026-02-27
**Current Status:** Late Beta (~90% feature complete)
**Next Milestone:** v1.0 Release

---

## Executive Summary

Wixen Mail has completed all major feature development including composition, contacts, search, multi-account, OAuth UI, security, and message rules. The project is in release-hardening stage with a short list of remaining gaps before v1.0.

All quality gates pass clean: `cargo fmt`, `cargo clippy`, `cargo test`.

---

## What's Complete

### Foundation
- Rust project with four-layer modular architecture
- Configuration management (JSON persistence)
- Structured logging with privacy-aware masking
- AccessKit integration for screen readers (NVDA, JAWS, Narrator)
- 25+ customizable keyboard shortcuts
- CI/CD pipeline (GitHub Actions)
- Comprehensive documentation (50KB+)

### Protocols
- IMAP4rev1 async client with IDLE push notifications
- SMTP client (lettre) with TLS/STARTTLS
- POP3 full command-surface implementation
- Connection pooling and retry logic

### User Interface
- Three-pane layout (folders, messages, preview)
- Thread view with conversation grouping
- Context menus with quick actions
- Provider auto-configuration (5 major providers)
- Performance-optimized ScrollArea rendering
- User-friendly error dialogs with troubleshooting tips

### Composition
- Compose window (To/CC/BCC/Subject/Body)
- Attachment management (file picker, add/remove, MIME types, size warnings)
- HTML/plain text toggle with formatting buttons
- Draft auto-save to SQLite
- Email signatures (multiple per account, auto-insert)
- Contact autocomplete in recipient fields

### Contact Management
- Full CRUD operations
- Fuzzy search across name, email, company, phone
- 6 sort options
- vCard 3.0 import/export
- Auto-import from message history
- 14-field contact schema with avatar support

### Search & Filtering
- Full-text search (SQLite FTS)
- Date range, sender/recipient, has-attachments filters
- Unread-only / starred-only filters
- Tag-based filtering
- Message rules engine with regex and actions

### Multi-Account
- Account CRUD with enable/disable
- Account switching UI ("Set Active")
- Per-account data isolation
- 5 provider presets with auto-detection

### OAuth 2.0 (UI Complete)
- Authorization flow UI
- Provider scopes (Gmail, Outlook)
- Token refresh logic and persistence

### Security
- AES-256-GCM credential encryption
- HTML sanitization (ammonia)
- PGP/S-MIME detection
- Phishing risk scoring
- TLS for all connections

### Offline Infrastructure
- SQLite message/folder/draft caching
- Outbox queue (table + CRUD)
- IMAP IDLE push plumbing

---

## What Remains for v1.0

### Small (1-2 days each)
| # | Item | Description |
|---|------|-------------|
| 1 | **OAuth HTTP exchange** | Replace mock stubs in `exchange_code()` and `refresh_access_token()` with real `reqwest` calls to Google/Microsoft token endpoints |
| 2 | **Compose account selector** | Add dropdown to select send-from account in composition window |
| 3 | **Preview before send** | Show rendered email preview before sending |

### Medium (3-7 days each)
| # | Item | Description |
|---|------|-------------|
| 4 | **Offline mode wiring** | Connect existing queue infrastructure: UI toggle, queue-flush-to-SMTP, sync status indicators |
| 5 | **Spell check** | Integrate an external spell-checking library |
| 6 | **Contact groups** | Distribution list support in contact manager |
| 7 | **Test coverage** | Expand from 2 tests to meaningful coverage across all modules |

### Post v1.0
- Theme customization (dark mode, high contrast)
- Calendar integration (CalDAV)
- Windows installer (MSI/NSIS)
- Large mailbox performance (100K+ messages)
- Virtual scrolling
- Plugin/extension system

---

## Recommended Development Sequence

### Week 1: OAuth & Compose
- Implement real OAuth token exchange (reqwest)
- Add compose account selector dropdown
- Add preview-before-send

### Week 2: Offline & Testing
- Wire offline mode (UI toggle, queue flush, sync indicators)
- Write unit tests for core modules (protocols, cache, search, contacts)

### Week 3: Polish & Beta
- Integrate spell check library
- Contact groups
- Accessibility regression testing with screen readers
- Performance profiling

### Week 4: Release
- Bug fixes from testing
- Final documentation review
- Windows installer
- Beta release

---

## Technical Dependencies Needed

```toml
# OAuth HTTP (for real token exchange)
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Spell checking (evaluate options)
# hunspell-rs or similar

# Windows installer
# cargo-wix or NSIS
```

---

## Success Metrics for v1.0

### Functional
- [ ] Real OAuth login works for Gmail and Outlook
- [ ] Offline compose-and-queue-and-send works end-to-end
- [ ] All major features accessible via keyboard and screen reader

### Quality
- [ ] >50% unit test coverage
- [ ] Zero critical bugs
- [ ] All quality gates pass (fmt, clippy, test)

### Performance
- [ ] Startup time <2 seconds
- [ ] Memory usage <150MB with 1000 cached messages
- [ ] UI responsive (<100ms for all actions)

---

**Estimated timeline to v1.0:** 3-4 weeks of focused work
**Confidence level:** High - all infrastructure is in place, only gap-filling remains

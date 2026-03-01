# Requirements Backlog

_Consolidated from 11 requirements and specification documents._
_Last updated: 2026-03-01_

This document replaces the individual `*_REQUIREMENTS.md`, `PHASE8_*`, `PHASE9_*`, `PHASE10_*`, `PHASE11_*`, and `MISSING_FUNCTIONALITY_REQUIREMENTS.md` files that previously lived in the repository root.

---

## Completed Requirements

All v1.0 feature requirements have been implemented. The original requirement documents are preserved here as a summary for traceability.

### Contact Management (formerly Phase 8)
Layered architecture with SQLite-backed CRUD, fuzzy search, vCard 3.0 import/export, composition-time autocomplete, contact groups and distribution lists. **Status: Done.**

### OAuth 2.0 Authentication (formerly Phase 9)
Provider metadata for Gmail and Outlook, authorization flow UI, real HTTP token exchange via reqwest, token refresh with persistence in SQLite. **Status: Done.**

### Offline Mode & Queued Send (formerly Phase 10)
Explicit offline mode toggle in View menu, SQLite outbox queue with CRUD, queue flush to SMTP on reconnect, outbox count and sync status indicators in the status bar. **Status: Done.**

### Beta Validation & Polish (formerly Phase 11)
Runtime diagnostics (accounts configured, active account, cache availability, OAuth state), accessibility-friendly diagnostics display, beta-risk warnings. **Status: Done.**

### IMAP IDLE Push Notifications
Event model with keepalive/EXISTS notifications, session lifecycle API (start/stop idle), controller-level orchestration, fallback simulated events. Plumbing is complete; actual push events will fire when connected to a real IMAP server. **Status: Done (plumbing).**

### POP3 Full Implementation
Complete client/session with all core commands (STAT, LIST, UIDL, RETR, TOP, DELE, RSET, NOOP, QUIT), MailController integration for connect/fetch/retrieve/delete, explicit SMTP sending for POP3 accounts. **Status: Done.**

### PGP / S-MIME / Phishing Detection
SecurityService with PGP and S-MIME signal detection, structured security report, phishing risk scoring (0-100) with heuristic indicators. Detection-only; full cryptographic validation is a post-v1.0 item. **Status: Done.**

### HTML Rendering & Attachment Pipeline
HTML sanitization via ammonia (XSS protection), plain-text accessibility fallback, link/image extraction, alt text. Attachment save-to-disk works. **Status: Done (core). Inline preview/open dialogs are post-v1.0.**

### Accessibility Automation & UIA Bridge
Thread-safe automation node store, semantic roles/states, announcement priority queue, native Windows UIA bridge via wxdragon/wxWidgets built-in support. **Status: Done (baseline).**

### Infrastructure Gap Closure
Storage, database, cache, search (FTS), and attachment subsystems all moved from stubs to working implementations. **Status: Done.**

---

## Remaining Work (Post-v1.0)

These items are not blockers for v1.0 release but are tracked for future development.

### Near-Term Enhancements

| Item | Description | Priority |
|------|-------------|----------|
| Full PGP encryption/decryption | Sign and encrypt messages (currently detection-only) | Medium |
| Attachment inline preview | Preview images, PDFs, text files in-app | Medium |
| Saved search / virtual folders | Persist search queries as virtual mailboxes | Low |
| Color-coded tags | Visual tag indicators in message list | Low |
| Folder favorites | Pin frequently used folders | Low |
| Spam filtering integration | Hook into external spam classifier | Low |

### Performance & Scale

| Item | Description | Priority |
|------|-------------|----------|
| Virtual scrolling | Efficient rendering for 100K+ message folders | High |
| Memory profiling | Target <150MB with 1000 cached messages | Medium |
| Startup time | Target <2 seconds cold start | Medium |
| Large mailbox testing | Validate with real-world 100K+ mailboxes | Medium |

### Platform & Distribution

| Item | Description | Priority |
|------|-------------|----------|
| Windows installer (MSI/NSIS) | Packaged installer with desktop shortcuts | High |
| Auto-update mechanism | Check for and apply updates | Medium |
| Theme customization | Dark mode, high contrast themes | Medium |
| Linux/macOS validation | Verify cross-platform builds | Low |

### Future Protocols & Integrations

| Item | Description | Priority |
|------|-------------|----------|
| Exchange Web Services (EWS) | Native Exchange protocol for calendar/contacts | Low |
| Microsoft Graph API | Modern Office 365 integration | Low |
| CardDAV / CalDAV | Contacts and calendar sync protocols | Low |
| JMAP protocol | Modern, efficient email protocol | Low |
| Calendar integration | Parse and display iCalendar invites | Medium |
| Plugin/extension system | Third-party extensibility | Low |

---

## Reference: Original Requirement Documents

The following root-level files were consolidated into this document:

- `PHASE8_ARCHITECTURE.md` — Contact management architecture
- `PHASE8_DETAILED_SPECIFICATIONS.md` — Contact management specifications
- `PHASE9_REQUIREMENTS.md` — OAuth 2.0 authentication
- `PHASE10_REQUIREMENTS.md` — Offline mode and queued send
- `PHASE11_REQUIREMENTS.md` — Polish and beta validation
- `HTML_ATTACHMENT_PIPELINE_REQUIREMENTS.md` — HTML rendering and attachments
- `IMAP_IDLE_PUSH_REQUIREMENTS.md` — IMAP IDLE push notifications
- `MISSING_FUNCTIONALITY_REQUIREMENTS.md` — Infrastructure gap analysis
- `PGP_SMIME_PHISHING_REQUIREMENTS.md` — Security feature detection
- `POP3_FULL_IMPLEMENTATION_REQUIREMENTS.md` — POP3 protocol implementation
- `ACCESSIBILITY_AUTOMATION_UIA_REQUIREMENTS.md` — Accessibility automation framework

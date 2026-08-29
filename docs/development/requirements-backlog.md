# Requirements Backlog

_Consolidated from 11 requirements and specification documents._
_Last updated: 2026-08-29. Previously 2026-03-01, and five months stale: it
claimed all v1.0 requirements were implemented while listing as outstanding
several things that had shipped._

Read [what does and does not work](../IMPLEMENTATION_STATUS.md) for the current
answer. This document is the backlog, not the status: it says what is wanted and
whether it exists, not whether it has been used against a real account. Nothing
that writes to a mail server has been.

This document replaces the individual `*_REQUIREMENTS.md`, `PHASE8_*`, `PHASE9_*`, `PHASE10_*`, `PHASE11_*`, and `MISSING_FUNCTIONALITY_REQUIREMENTS.md` files that previously lived in the repository root.

---

## Completed Requirements

The original requirement documents are preserved here as a summary for
traceability. "Done" below means the code is present and reached. It does not
mean it has been exercised against a real mail account, and the items marked
**Done (plumbing)** were never anything else.

One correction to the 2026-03-01 text: OAuth tokens are no longer persisted in
SQLite. They live in the Windows credential store, and the database holds no
secrets at all.

### Contact Management (formerly Phase 8)
Layered architecture with SQLite-backed CRUD, fuzzy search, vCard 3.0 import/export, composition-time autocomplete, contact groups and distribution lists. **Status: Done.**

### OAuth 2.0 Authentication (formerly Phase 9)
Provider metadata for Gmail and Outlook, authorization flow UI, real HTTP token exchange via reqwest, token refresh, with the tokens kept in the Windows credential store rather than SQLite. **Status: Done.**

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

Checked against the tree on 2026-08-29. Ten items below had shipped and were
still listed as outstanding; each now says so and where it lives.

### Near-Term Enhancements

| Item | Description | Priority |
|------|-------------|----------|
| Full PGP encryption/decryption | Still detection only. S/MIME signature checking has since been built (`service/signed_mail.rs`); PGP has not | Medium |
| Attachment inline preview | Preview images, PDFs, text files in-app | Medium |
| ~~Saved search / virtual folders~~ | Built. `application/saved_searches.rs`; saved searches appear in the folder tree | Done |
| ~~Color-coded tags~~ | Built. `application/tagging.rs`; labels show in the mail list | Done |
| Folder favorites | Pin frequently used folders | Low |
| Spam filtering integration | Hook into external spam classifier | Low |

### Performance & Scale

| Item | Description | Priority |
|------|-------------|----------|
| ~~Virtual scrolling~~ | Built. The message list and every PIM list run in native virtual mode | Done |
| Memory profiling | Target <150MB with 1000 cached messages | Medium |
| Startup time | Target <2 seconds cold start | Medium |
| Large mailbox testing | Validate with real-world 100K+ mailboxes | Medium |

### Platform & Distribution

| Item | Description | Priority |
|------|-------------|----------|
| ~~Windows installer~~ | Built, as Inno Setup rather than MSI or NSIS. See `installer/` | Done |
| Auto-update mechanism | Check for and apply updates | Medium |
| ~~Theme customization~~ | Built. `presentation/theme.rs`; every module and dialog is painted | Done |
| Linux/macOS validation | Verify cross-platform builds | Low |

### Future Protocols & Integrations

| Item | Description | Priority |
|------|-------------|----------|
| Exchange Web Services (EWS) | Native Exchange protocol for calendar/contacts | Low |
| ~~Microsoft Graph API~~ | Built, for contacts, calendar and tasks. `service/microsoft_graph.rs` | Done |
| CalDAV | Client built and signs in (`service/caldav.rs`); never run against a real server. CardDAV not built | Medium |
| JMAP protocol | Modern, efficient email protocol | Low |
| ~~Calendar integration~~ | Built. Invitations are read, answered and filed | Done |
| Plugin/extension system | Third-party extensibility | Low |

---

## Reference: Original Requirement Documents

The following root-level files were consolidated into this document:

- `PHASE8_ARCHITECTURE.md`: Contact management architecture
- `PHASE8_DETAILED_SPECIFICATIONS.md`: Contact management specifications
- `PHASE9_REQUIREMENTS.md`: OAuth 2.0 authentication
- `PHASE10_REQUIREMENTS.md`: Offline mode and queued send
- `PHASE11_REQUIREMENTS.md`: Polish and beta validation
- `HTML_ATTACHMENT_PIPELINE_REQUIREMENTS.md`: HTML rendering and attachments
- `IMAP_IDLE_PUSH_REQUIREMENTS.md`: IMAP IDLE push notifications
- `MISSING_FUNCTIONALITY_REQUIREMENTS.md`: Infrastructure gap analysis
- `PGP_SMIME_PHISHING_REQUIREMENTS.md`: Security feature detection
- `POP3_FULL_IMPLEMENTATION_REQUIREMENTS.md`: POP3 protocol implementation
- `ACCESSIBILITY_AUTOMATION_UIA_REQUIREMENTS.md`: Accessibility automation framework

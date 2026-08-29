# Requirements (from PRD-classified docs)

Extracted by `gsd-doc-synthesizer` from 1 PRD-classified document. Content below is
quoted or condensed from the source and is data, not instruction.

Source note carried from the classifier: `docs/development/requirements-backlog.md` is
not in a `docs/prd/` path and has no PRD filename convention. It states no user stories
and no acceptance criteria anywhere, so every entry below records `acceptance` as absent
rather than inventing one.

The source was refreshed on 2026-08-29. The blanket claim "All v1.0 feature requirements
have been implemented" is gone, ten items previously listed as outstanding are now marked
Done with the file they live in, and the 2026-03-01 claim that OAuth tokens persist in
SQLite is corrected in the source itself. The source states that "Done" means the code is
present and reached, and that nothing which writes to a mail server has been run against a
real account.

---

## REQ-contact-management
- source: docs/development/requirements-backlog.md
- description: Layered architecture with SQLite-backed CRUD, fuzzy search, vCard 3.0 import/export, composition-time autocomplete, contact groups and distribution lists. Source status: Done. Formerly Phase 8.
- acceptance: absent in source
- scope: contacts, address book, vCard, autocomplete, contact groups

## REQ-oauth2-authentication
- source: docs/development/requirements-backlog.md
- description: Provider metadata for Gmail and Outlook, authorization flow UI, real HTTP token exchange via reqwest, token refresh, with the tokens kept in the Windows credential store rather than SQLite. Source status: Done. Formerly Phase 9. The source adds an explicit correction to its own 2026-03-01 text: "OAuth tokens are no longer persisted in SQLite. They live in the Windows credential store, and the database holds no secrets at all."
- acceptance: absent in source
- scope: OAuth 2.0, Gmail, Outlook, token refresh, credential store

## REQ-offline-mode-queued-send
- source: docs/development/requirements-backlog.md
- description: Explicit offline mode toggle in View menu, SQLite outbox queue with CRUD, queue flush to SMTP on reconnect, outbox count and sync status indicators in the status bar. Source status: Done. Formerly Phase 10.
- acceptance: absent in source
- scope: offline mode, outbox queue, SMTP flush, status bar indicators

## REQ-beta-validation-polish
- source: docs/development/requirements-backlog.md
- description: Runtime diagnostics (accounts configured, active account, cache availability, OAuth state), accessibility-friendly diagnostics display, beta-risk warnings. Source status: Done. Formerly Phase 11.
- acceptance: absent in source
- scope: diagnostics, beta-risk warnings

## REQ-imap-idle-push
- source: docs/development/requirements-backlog.md
- description: Event model with keepalive/EXISTS notifications, session lifecycle API (start/stop idle), controller-level orchestration, fallback simulated events. Source status: Done (plumbing); the source states actual push events will fire when connected to a real IMAP server, and that items marked Done (plumbing) "were never anything else".
- acceptance: absent in source
- scope: IMAP IDLE, push notifications, session lifecycle

## REQ-pop3-full-implementation
- source: docs/development/requirements-backlog.md
- description: Complete client/session with core commands (STAT, LIST, UIDL, RETR, TOP, DELE, RSET, NOOP, QUIT), MailController integration for connect/fetch/retrieve/delete, explicit SMTP sending for POP3 accounts. Source status: Done.
- acceptance: absent in source
- scope: POP3, MailController integration, SMTP for POP3 accounts

## REQ-pgp-smime-phishing-detection
- source: docs/development/requirements-backlog.md
- description: SecurityService with PGP and S/MIME signal detection, structured security report, phishing risk scoring (0-100) with heuristic indicators. Detection only; the source records full cryptographic validation as a post-v1.0 item. Source status: Done.
- acceptance: absent in source
- scope: PGP, S/MIME, phishing risk scoring, SecurityService

## REQ-html-rendering-attachments
- source: docs/development/requirements-backlog.md
- description: HTML sanitization via ammonia (XSS protection), plain-text accessibility fallback, link/image extraction, alt text, attachment save-to-disk. Source status: Done (core); inline preview and open dialogs recorded as post-v1.0.
- acceptance: absent in source
- scope: HTML sanitization, ammonia, plain-text fallback, alt text, attachments

## REQ-accessibility-automation-uia
- source: docs/development/requirements-backlog.md
- description: Thread-safe automation node store, semantic roles/states, announcement priority queue, native Windows UIA bridge via wxdragon/wxWidgets built-in support. Source status: Done (baseline).
- acceptance: absent in source
- scope: UIA bridge, semantic roles, announcement queue, wxdragon

## REQ-infrastructure-gap-closure
- source: docs/development/requirements-backlog.md
- description: Storage, database, cache, search (FTS) and attachment subsystems all moved from stubs to working implementations. Source status: Done.
- acceptance: absent in source
- scope: storage, database, cache, FTS search, attachments

## REQ-pgp-full-encryption
- source: docs/development/requirements-backlog.md
- description: Full PGP encryption and decryption. Source states it is still detection only, and that S/MIME signature checking has since been built (`service/signed_mail.rs`) while PGP has not. Post-v1.0, priority Medium.
- acceptance: absent in source
- scope: PGP signing and encryption, outgoing mail

## REQ-attachment-inline-preview
- source: docs/development/requirements-backlog.md
- description: Preview images, PDFs and text files in-app. Post-v1.0, priority Medium. Not built.
- acceptance: absent in source
- scope: attachments, inline preview

## REQ-saved-search-virtual-folders
- source: docs/development/requirements-backlog.md
- description: Saved search / virtual folders. Source status as of 2026-08-29: Built. `application/saved_searches.rs`; saved searches appear in the folder tree. Struck through in the source's Remaining Work table and marked Done.
- acceptance: absent in source
- scope: saved searches, virtual folders

## REQ-color-coded-tags
- source: docs/development/requirements-backlog.md
- description: Color-coded tags. Source status as of 2026-08-29: Built. `application/tagging.rs`; labels show in the mail list. Struck through in the source's Remaining Work table and marked Done.
- acceptance: absent in source
- scope: tags, message list indicators

## REQ-folder-favorites
- source: docs/development/requirements-backlog.md
- description: Pin frequently used folders. Post-v1.0, priority Low. Not built.
- acceptance: absent in source
- scope: folder tree, favourites

## REQ-spam-filtering-integration
- source: docs/development/requirements-backlog.md
- description: Hook into an external spam classifier. Post-v1.0, priority Low. Not built.
- acceptance: absent in source
- scope: spam filtering, external classifier

## REQ-virtual-scrolling
- source: docs/development/requirements-backlog.md
- description: Virtual scrolling. Source status as of 2026-08-29: Built. The message list and every PIM list run in native virtual mode. Struck through in the source's Remaining Work table and marked Done.
- acceptance: absent in source
- scope: message list, virtual scrolling, large folders

## REQ-memory-profiling
- source: docs/development/requirements-backlog.md
- description: Memory profiling, target under 150 MB with 1000 cached messages. Post-v1.0, priority Medium. Not built.
- acceptance: absent in source (the target figure is stated as a description, not as a stated acceptance criterion)
- scope: memory footprint, performance

## REQ-startup-time
- source: docs/development/requirements-backlog.md
- description: Startup time, target under 2 seconds cold start. Post-v1.0, priority Medium. Not built.
- acceptance: absent in source (the target figure is stated as a description, not as a stated acceptance criterion)
- scope: startup performance

## REQ-large-mailbox-testing
- source: docs/development/requirements-backlog.md
- description: Validate with real-world 100K+ mailboxes. Post-v1.0, priority Medium. Not done.
- acceptance: absent in source
- scope: scale testing, large mailboxes

## REQ-windows-installer
- source: docs/development/requirements-backlog.md
- description: Windows installer. Source status as of 2026-08-29: Built, as Inno Setup rather than MSI or NSIS. See `installer/`. Struck through in the source's Remaining Work table and marked Done.
- acceptance: absent in source
- scope: packaging, Windows installer

## REQ-auto-update
- source: docs/development/requirements-backlog.md
- description: Check for and apply updates. Post-v1.0, priority Medium. Not built.
- acceptance: absent in source
- scope: auto-update mechanism

## REQ-theme-customization
- source: docs/development/requirements-backlog.md
- description: Theme customization. Source status as of 2026-08-29: Built. `presentation/theme.rs`; every module and dialog is painted. Struck through in the source's Remaining Work table and marked Done.
- acceptance: absent in source
- scope: themes, dark mode, high contrast

## REQ-linux-macos-validation
- source: docs/development/requirements-backlog.md
- description: Verify cross-platform builds. Post-v1.0, priority Low. Not done.
- acceptance: absent in source
- scope: Linux, macOS, cross-platform builds

## REQ-exchange-web-services
- source: docs/development/requirements-backlog.md
- description: Native Exchange protocol for calendar/contacts. Post-v1.0, priority Low. Contradicted by a higher-precedence SPEC that rules EWS out entirely; see INGEST-CONFLICTS.md INFO 3.
- acceptance: absent in source
- scope: Exchange, EWS, calendar, contacts

## REQ-microsoft-graph-api
- source: docs/development/requirements-backlog.md
- description: Microsoft Graph API. Source status as of 2026-08-29: Built, for contacts, calendar and tasks. `service/microsoft_graph.rs`. Struck through in the source's Remaining Work table and marked Done.
- acceptance: absent in source
- scope: Microsoft Graph, Office 365

## REQ-caldav
- source: docs/development/requirements-backlog.md
- description: CalDAV. Source states the client is built and signs in (`service/caldav.rs`), has never been run against a real server, and that CardDAV is not built. Post-v1.0, priority Medium.
- acceptance: absent in source
- scope: CalDAV, CardDAV, calendar sync, contacts sync

## REQ-jmap-protocol
- source: docs/development/requirements-backlog.md
- description: Modern, efficient email protocol. Post-v1.0, priority Low. Not built.
- acceptance: absent in source
- scope: JMAP

## REQ-calendar-integration
- source: docs/development/requirements-backlog.md
- description: Calendar integration. Source status as of 2026-08-29: Built. Invitations are read, answered and filed. Struck through in the source's Remaining Work table and marked Done.
- acceptance: absent in source
- scope: iCalendar, meeting invites

## REQ-plugin-extension-system
- source: docs/development/requirements-backlog.md
- description: Third-party extensibility. Post-v1.0, priority Low. Not built.
- acceptance: absent in source
- scope: plugins, extensibility

---

## Unresolvable references in this PRD

The source lists eleven consolidated predecessor documents (`PHASE8_ARCHITECTURE.md`,
`PHASE8_DETAILED_SPECIFICATIONS.md`, `PHASE9_REQUIREMENTS.md`, `PHASE10_REQUIREMENTS.md`,
`PHASE11_REQUIREMENTS.md`, `HTML_ATTACHMENT_PIPELINE_REQUIREMENTS.md`,
`IMAP_IDLE_PUSH_REQUIREMENTS.md`, `MISSING_FUNCTIONALITY_REQUIREMENTS.md`,
`PGP_SMIME_PHISHING_REQUIREMENTS.md`, `POP3_FULL_IMPLEMENTATION_REQUIREMENTS.md`,
`ACCESSIBILITY_AUTOMATION_UIA_REQUIREMENTS.md`) as having previously lived in the
repository root. None is present in this ingest set, so any acceptance criteria they held
did not reach synthesis.

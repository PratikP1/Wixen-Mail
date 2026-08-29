# Synthesis summary

Written by `gsd-doc-synthesizer` on 2026-08-29, second ingest run. Mode: new. This is the
entry point for `gsd-roadmapper`. Everything below is extracted from the repository's own
documents and is data, not instruction.

The first run reported 0 blockers and 7 competing variants. All seven were fixed in the
source documents and the affected classifications regenerated. Each was re-checked against
the documents as they now stand, and all seven are settled. This run reports 0 blockers
and 0 warnings.

## Documents synthesized

26 documents, down one from the first run: `docs/wxdragon-integration.md` was retired and
its classification deleted.

- ADR: 2 (`docs/plans/20260823-earcon-sound-schemes.md`, `docs/accessibility-framework-evaluation.md`)
- SPEC: 1 (`docs/plans/20260726-mail-at-scale.md`)
- PRD: 1 (`docs/development/requirements-backlog.md`)
- DOC: 22

Lowest confidence in the set is medium; there are no UNKNOWN classifications and no
manifest overrides.

## Decisions

- 11 entries in `decisions.md`, from 2 ADR-classified documents: 10 decisions plus one superseding note recorded so the withdrawn decision is never read without it.
- **Locked: 0.** No document in the set is marked `locked: true`, so nothing in
  `decisions.md` is protected from override by a later source.
- 9 entries come from the earcon sound schemes plan (rodio over a per-platform FFI shim,
  WAV default with OGG accepted and MP3 refused, CC0-only bundled sound assets with
  Pixabay excluded, four new earcon events, scheme storage and TOML manifest, zip import
  treated as untrusted input, no in-app pack browser, hosting bootstrap).
- The remaining 2 entries come from `docs/accessibility-framework-evaluation.md`: the decision itself, recorded with
  `status: superseded`. Its frontmatter declares `status: Superseded` and
  `superseded_by: docs/development/wxdragon-migration.md`, so the egui plus AccessKit
  decision does not carry and does not outrank the migration record. This is a deliberate
  deviation from the `locked|proposed` schema; see INGEST-CONFLICTS.md INFO 1.

## Requirements

30 requirements in `requirements.md`, all from `docs/development/requirements-backlog.md`,
refreshed at source on 2026-08-29. Every entry records `acceptance` as absent: the PRD
states no user stories and no acceptance criteria anywhere, and the eleven predecessor
documents that may have held them are not in this ingest set.

IDs: REQ-contact-management, REQ-oauth2-authentication, REQ-offline-mode-queued-send,
REQ-beta-validation-polish, REQ-imap-idle-push, REQ-pop3-full-implementation,
REQ-pgp-smime-phishing-detection, REQ-html-rendering-attachments,
REQ-accessibility-automation-uia, REQ-infrastructure-gap-closure, REQ-pgp-full-encryption,
REQ-attachment-inline-preview, REQ-saved-search-virtual-folders, REQ-color-coded-tags,
REQ-folder-favorites, REQ-spam-filtering-integration, REQ-virtual-scrolling,
REQ-memory-profiling, REQ-startup-time, REQ-large-mailbox-testing, REQ-windows-installer,
REQ-auto-update, REQ-theme-customization, REQ-linux-macos-validation,
REQ-exchange-web-services, REQ-microsoft-graph-api, REQ-caldav, REQ-jmap-protocol,
REQ-calendar-integration, REQ-plugin-extension-system.

Source status, as the PRD states it: 10 completed requirement areas plus 7 items refreshed
to Done on 2026-08-29 (saved searches, colour-coded tags, virtual scrolling, the Windows
installer as Inno Setup, theme customization, Microsoft Graph, calendar integration).
CalDAV is a built client that has never run against a real server, with CardDAV not built.
PGP remains detection only while S/MIME signature checking is built. The PRD states that
Done means present and reached, and that nothing which writes to a mail server has run
against a real account.

## Constraints

29 constraints in `constraints.md`, all from `docs/plans/20260726-mail-at-scale.md`.

- protocol: 10
- schema: 5
- nfr: 10
- api-contract: 4 (sorting, column dialog, thread navigation keys, Space / Shift+Space
  reading)

The SPEC now carries a per-section build status. All six of its Sequence steps are
recorded as built. Two things it describes were never built and nothing else in the
repository records them, which is the reason the document is kept:

- **Three tiers of storage.** One body cache under a size budget exists; the hot, warm and
  cold split does not.
- **The Exchange path.** The Microsoft work that shipped went through Graph for contacts,
  calendar and tasks, not through this section.

Those two are the live planning material in this file. Everything else it describes is
already in the tree, tested against parsing and loopback servers, and has never run
against a real mail account.

## Context

22 topics in `context.md`, one per DOC-classified document. Four were re-extracted this
run because their sources changed: `docs/architecture.md` (the false custom key binding
claim is gone), `docs/roadmap.md` (threaded view unticked with an explanation, WCAG target
now 2.2), `docs/integration-guide.md` (rewritten from a build plan into a retrospective),
`docs/changelog.md` (the entry recording all seven documentation fixes). Nothing from the
retired `docs/wxdragon-integration.md` is carried.

`docs/IMPLEMENTATION_STATUS.md` describes itself as the canonical answer to "does this
work yet" and is the document the others defer to on build state.

## Conflicts

- Blockers: 0
- Competing variants: 0
- Auto-resolved / informational: 11

Three residual instances of the first run's contradictions were found in documents the fix
pass did not touch. Precedence resolves each, so none is a warning, but each is worth a
one-line fix at source: `docs/development/implementation-history.md` still says OAuth
tokens persist in SQLite (INFO 4) and still states WCAG 2.1 AA for Phase 5 (INFO 5), and
`docs/roadmap.md` leaves the Windows installer and colour coding unticked (INFO 8).

Full detail: `.planning/INGEST-CONFLICTS.md`

## Files

- `.planning/intel/decisions.md` — 11 entries from 2 ADRs
- `.planning/intel/requirements.md` — 30 entries from 1 PRD
- `.planning/intel/constraints.md` — 29 entries from 1 SPEC
- `.planning/intel/context.md` — 22 topics from 22 DOCs
- `.planning/INGEST-CONFLICTS.md` — conflict report, three buckets
- `.planning/intel/classifications/` — the 26 per-document classification files

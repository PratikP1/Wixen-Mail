---
gsd_state_version: '1.0'
status: planning
progress:
  total_phases: 8
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-08-29)

**Core value:** Making correspondence and personal information legible to people who cannot see it.
**Current focus:** Phase 1, Folders and conversations

## Current Position

Phase: 1 of 8 (Folders and conversations)
Plan: none yet
Status: Phase 1 reviewed and ready to plan. Phases 2 to 8 awaiting review.

**Phase 1 reviewed 2026-08-29 with Pratik.** Two criteria changed:

- FOLDER-01 said all five folder operations pass through `Allowed::mail`. Wrong for POP
  accounts, which have no server folders at all, and for the IMAP outbox.
  `local_folders::is_local` already draws that line and is now named as the single place that
  decides it. Server folders keep the gate; local ones do not.
- FOLDER-03 keeps local pinning as this phase's work, and now says the stored shape must let
  IMAP subscription back it later, with the decision about which wins recorded before the
  second half is built rather than settled by whichever code path runs last.
Last activity: 2026-08-29, roadmap created from the ingested documents and the codebase map

Progress: [..........] 0%

## Performance Metrics

**Velocity:**
- Total plans completed: 0
- Average duration: not measured
- Total execution time: not measured

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**
- Last 5 plans: none yet
- Trend: not measured

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in the PROJECT.md Key Decisions table. The ones that shape the phases
ahead:

- No EWS. Microsoft blocks third-party EWS from 1 October 2026. Exchange goes through Graph.
- Writes split into `mail` and `personal_information` in `src/application/allowed.rs`, with
  three places that must agree. Mail writes are off for a new install.
- The message list stays native virtual mode, because only the native control gives UI
  Automation the real set size.
- The cached mail database is not encrypted, and the docs say so. Phase 7 decides whether that
  changes.

### Pending Todos

None yet.

### Blockers/Concerns

- **Awaiting review, phases 2 to 8.** Every acceptance criterion marked **[D]** was derived by
  a model from the code and the status documents, not stated by Pratik or by a source. Phase 1
  is reviewed; the rest are not, and no phase should be planned before its own review.
- ~~**Row count discrepancy.**~~ Resolved 2026-08-29. The file has 33 rows and 33 is right.
  The 27 came from the inventory agent's own summary of the document it had just written, and
  was passed into the roadmapper's brief without anyone counting the file. Nothing was dropped:
  all 33 are accounted for in REQUIREMENTS.md. Raising it rather than reconciling to the number
  in the brief is what kept six rows in scope.
- ~~**Phase 5, PIM-04**~~ answered 2026-08-29: not one target. A backend chosen by account
  type behind one seam, the local note a first-class Markdown document, and the seam shaped so
  a hosted service can be added later without a migration. Split into PIM-04, PIM-07, PIM-08.
- ~~**Phase 7, SHIP-04**~~ answered 2026-08-29: the cache is not encrypted. The remaining work
  is saying so where a user meets it, not building anything.
- **Phase 7, SHIP-01** is blocked on a certificate decision that is Pratik's.
- **Nothing has ever run against a real mail account.** No criterion in this milestone claims
  otherwise, and none may be rewritten to.

## Deferred Items

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| Protocol | Gmail X-GM-THRID and X-GM-RAW | v2 | 2026-08-29 | this one |
| Protocol | The Exchange path in the mail-at-scale plan | v2 | 2026-08-29 | this one |
| Protocol | JMAP | v2 | 2026-08-29 | this one |
| Platform | Plugin and extension system | v2 | 2026-08-29 | this one |
| Platform | Set as the Windows default mail client | v2 | 2026-08-29 | this one |
| Validation | Live-account validation of the 13 unproven rows | Out of scope | 2026-08-29 | this one |

## Session Continuity

Last session: 2026-08-29
Stopped at: PROJECT.md, REQUIREMENTS.md, ROADMAP.md and STATE.md written from the ingested
documents and the codebase map. Nothing planned or executed yet.
Resume file: None

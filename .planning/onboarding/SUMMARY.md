# Onboarding Summary

Written 2026-08-29.

## Project State
- PROJECT.md: present
- REQUIREMENTS.md: present
- ROADMAP.md: present
- STATE.md: present

## Codebase Context
- Brownfield repo: yes
- Map readiness: complete
- Codebase map: `.planning/codebase/` (complete codebase map, 7 documents, 1,034 lines)
- Fast map available: no

## Docs Context
- Existing ADR/PRD/SPEC/RFC candidates: 0 by the projection's count, 4 in fact.

  The detector matches ADR, PRD, SPEC and RFC in file names and directory
  names. This repository has none of those and keeps its decision records under
  ordinary names, so the projection reported nothing to ingest three times
  running. Ingesting the 26 documents under `docs/` and classifying them by
  content found 2 ADR, 1 SPEC and 1 PRD. Anyone reading this line later should
  read it as a fact about the detector, not about the repository.

## What the ingest found
- 26 documents classified, 0 blockers.
- The first run reported 7 competing variants: seven contradictions between the
  documents themselves, not artifacts of the ingest. All seven were settled
  against the code and fixed at source. The second run reported 0 blockers and
  0 warnings.
- `.planning/intel/built-and-left.md` sorts every feature into built and
  exercised, built but unproven, not built, and unclear. The middle section is
  the one that matters: thirteen things are present and reached and have never
  run against a real server.

## Known state of the plan
- 8 phases, 40 requirements, each traced to exactly one phase.
- About 160 acceptance criteria, almost all marked `[D]` for derived, because
  the source backlog states none. They are unreviewed. STATE.md holds at
  "ready to plan once the derived acceptance criteria are reviewed".
- Three decisions in the roadmap are Pratik's and are written as criteria that
  say decide and record, not as work to start: the signing certificate, the
  notes sync target, and whether to encrypt the local cache.

## Recommended Next Step
- Review the `[D]` acceptance criteria in `.planning/REQUIREMENTS.md`, then
  `/gsd-manager`.

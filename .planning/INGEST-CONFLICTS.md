## Conflict Detection Report

Second ingest run, 2026-08-29. Ingest mode: new. 26 documents classified (2 ADR, 1 SPEC,
1 PRD, 22 DOC). Precedence applied: ADR > SPEC > PRD > DOC. No classification carries a
per-doc precedence override and none is marked `locked: true`.

The first run reported 0 blockers and 7 competing variants. Every one was re-checked
against the documents as they stand now, not against the note saying they were fixed.
All seven are settled; what each fix looks like on disk is recorded in the INFO bucket.
Three residual instances of the same contradictions were found in documents the fix pass
did not touch, and precedence resolves each of them; they are INFO 4, INFO 5 and INFO 8.

### BLOCKERS (0)

No blockers. No UNKNOWN classifications, no low-confidence documents (the lowest
confidence in this set is medium), no cross-reference cycles, and no LOCKED-vs-LOCKED
contradiction is possible because no document is marked locked.

### WARNINGS (0)

No competing variants remain. The seven from the first run are settled:

1. Stale SPEC outranking the status documents. Settled. `docs/plans/20260726-mail-at-scale.md`
   now opens with "What is built, as of 2026-08-29" and a per-step table; all six Sequence
   steps are Built with the file each lives in. Its old "Status: agreed, not yet
   implemented" line is gone.
2. Superseded ADR outranking the record that replaced it. Settled and now machine-readable;
   see INFO 1.
3. PRD five months older than the documents it outranks. Settled.
   `docs/development/requirements-backlog.md` is dated 2026-08-29, the blanket "All v1.0
   feature requirements have been implemented" claim is gone, and ten items are struck
   through and marked Done with the file each lives in.
4. Threaded view described as both done and not done. Settled. `docs/roadmap.md` line 80
   is now unticked and explains why, matching `docs/IMPLEMENTATION_STATUS.md` and the
   Thread View section of `docs/USER_GUIDE.md`.
5. Two WCAG target versions. Settled for the four sources named last time; see INFO 5 for
   the one further instance found this run.
6. Finished work described as unstarted. Settled. `docs/wxdragon-integration.md` is gone
   from the repository and from the classifications, and `docs/integration-guide.md` is a
   retrospective that names its own former errors.
7. Shortcut customisation described as both available and absent. Settled.
   `docs/architecture.md` now reads "Shortcuts are fixed. `register_shortcut` is called
   once at startup and nothing in Settings reaches it, so there is no way to rebind a key",
   agreeing with `docs/accessibility.md` and `docs/KEYBOARD_SHORTCUTS.md`. The classifier
   flagged a possible leftover, a "Keyboard shortcuts" settings category under
   Configuration Manager; it is not in the file. That section lists application
   preferences, account settings, UI customization, accessibility options and privacy
   settings, and names no shortcut category.

### INFO (11)

[INFO] 1. Auto-resolved: a Superseded ADR does not outrank the record that replaced it
  Found: docs/accessibility-framework-evaluation.md now carries YAML frontmatter reading `type: ADR`, `status: Superseded`, `superseded_by: docs/development/wxdragon-migration.md`, above a banner saying the project does not use egui and that the document is "Kept for the record of how the decision was reached, not as guidance"
  Found: docs/development/wxdragon-migration.md records the migration from egui/eframe 0.29 to wxdragon 0.9.12 as complete on 2026-02-27 with all egui code removed
  Note: Last run this needed a human because the withdrawal existed only in prose and the classifier could not read it. The status is now declared, so precedence is applied mechanically: the egui plus AccessKit decision is withdrawn and does not carry, and the migration record wins. decisions.md records the entry with `status: superseded`, a deliberate deviation from the `locked|proposed` schema, because recording it as `proposed` would present a withdrawn decision as open and `locked` would be false

[INFO] 2. Auto-resolved: ADR > SPEC on earcon and braille status, and the SPEC now says so itself
  Found: docs/plans/20260726-mail-at-scale.md still carries a feedback table reading "Earcon: Not started" and "Braille: Nothing exists"
  Found: the same document's "Feedback channels, revisited" section states both have since been built, naming `presentation/accessibility/feedback.rs`, `sound_scheme.rs` and `sound_scheme_import.rs`, and says the table "is left as written, because it records what was true when the plan was made"
  Note: docs/plans/20260823-earcon-sound-schemes.md, ADR-classified, records Phases 1 through 4 as done, committed and verified, with only Phase 5 (hosting on wixen.app) blocked, and docs/accessibility.md independently describes the scheme picker, Import sound scheme and Delete sound scheme as present. ADR outranks SPEC and the SPEC agrees with it in its own words, so earcons and braille are recorded as built

[INFO] 3. Auto-resolved: SPEC > PRD on Exchange Web Services
  Found: docs/development/requirements-backlog.md lists "Exchange Web Services (EWS): Native Exchange protocol for calendar/contacts" as future work at Low priority
  Found: docs/plans/20260726-mail-at-scale.md states "We will not write EWS", citing Microsoft blocking non-Microsoft EWS applications against Exchange Online from 1 October 2026 with full retirement by April 2027
  Note: SPEC outranks PRD, so EWS is recorded as ruled out rather than as backlog. docs/roadmap.md also leaves EWS unticked, which agrees. This is the only one of the first run's four auto-resolutions that still applies: the Microsoft Graph and "v1.0 complete" ones are gone because the refreshed PRD now agrees with the SPEC on both

[INFO] 4. Auto-resolved: PRD > DOC on where OAuth tokens live
  Found: docs/development/requirements-backlog.md states, as an explicit correction to its own 2026-03-01 text, "OAuth tokens are no longer persisted in SQLite. They live in the Windows credential store, and the database holds no secrets at all"
  Found: docs/development/implementation-history.md, Phase 6, still reads "OAuth 2.0: Authorization flow UI, provider-specific scopes, token refresh, SQLite persistence, real HTTP exchange via reqwest"
  Note: PRD outranks DOC, and docs/IMPLEMENTATION_STATUS.md, docs/privacy.md and docs/installing.md all agree with the PRD, so the credential store answer is what reached synthesis. The stale line is still in docs/development/implementation-history.md on disk. That file is a dated record ("Last updated: 2026-03-01") of what each phase did, which is why this resolves rather than blocks, but a page shipping inside the installer that says secrets are in the database is worth a one-line fix

[INFO] 5. Auto-resolved: the accessibility target is WCAG 2.2 Level AA
  Found: docs/accessibility.md ("Wixen Mail targets **WCAG 2.2 Level AA**"), docs/roadmap.md success metrics ("WCAG 2.2 Level AA compliance") and docs/USER_GUIDE.md ("targets WCAG 2.2 Level AA") all state 2.2
  Found: docs/plans/20260726-mail-at-scale.md requires WCAG 2.5.7 by name for column reordering. 2.5.7 Dragging Movements is a WCAG 2.2 addition, so the SPEC, which outranks every DOC here, can only be satisfied at 2.2
  Found: two documents still say 2.1, and both are history rather than guidance. docs/accessibility-framework-evaluation.md refers to WCAG 2.1 AA throughout and is frontmatter-declared Superseded. docs/development/implementation-history.md, Phase 5, reads "Accessibility for HTML: Plain text fallback, WCAG 2.1 AA compliance" in a chronological record dated 2026-03-01
  Note: The three sources the first run named as stating 2.1 were fixed: docs/roadmap.md now reads 2.2, docs/integration-guide.md no longer states a target at all, and docs/wxdragon-integration.md is retired. The implementation-history instance was not found by the first run and is new here. The changelog entry describing the fix says the target is stated as 2.2 "everywhere now", which is not literally true of those two historical files

[INFO] 6. The SPEC's body is deliberately left as written and superseded by its own header
  Found: docs/plans/20260726-mail-at-scale.md body still contains "Receiving mail is blocked on it" about the XOAUTH2 migration, "until earcons exist the announcement is the only option", and the Not started / Nothing exists feedback rows
  Note: The document's 2026-08-29 header records every one of those as built and explains that the body is preserved as the record of the plan as made. constraints.md carries each affected entry with the source's own status attached, so a downstream planner reads the built state rather than the historical sentence. Nothing here needs a decision; it is recorded so the difference is not mistaken for a contradiction later

[INFO] 7. Test counts differ across documents, and each states its own measurement date
  Found: docs/roadmap.md Phase 7 ticks "150 unit tests" and "26 integration tests"; docs/development/implementation-history.md gives the same figures for 2026-03-01; docs/IMPLEMENTATION_STATUS.md records "3362 tests pass: 3282 unit and 80 integration, measured 2026-08-09"; docs/integration-guide.md refers to "the 5,269 that run today" on 2026-08-29
  Note: These are dated measurements of a growing suite rather than competing claims, so nothing is resolved against anything. Recorded because a planner reading the roadmap alone would take 150 unit tests as current

[INFO] 8. Auto-resolved: PRD > DOC on the Windows installer and on colour-coded tags
  Found: docs/roadmap.md Phase 8 leaves "Windows installer (MSI or NSIS)" unticked and Phase 5 leaves "Color coding" unticked
  Found: docs/development/requirements-backlog.md strikes both through as Done: the installer "Built, as Inno Setup rather than MSI or NSIS. See `installer/`", and colour-coded tags "Built. `application/tagging.rs`; labels show in the mail list"
  Note: PRD outranks DOC, so both are recorded as built. docs/installing.md and docs/BETA_RELEASE.md describe a shipping Inno Setup installer, which agrees. The roadmap's unticked line is literally true, since no MSI or NSIS installer exists, but it reads as no installer at all

[INFO] 9. No decision in this ingest set is locked
  Note: Both ADR-classified documents carry `locked: false`, and one of the two is Superseded. Nothing in decisions.md is protected from being overridden by a later source, and the LOCKED-vs-LOCKED hard block never engaged. If any of these decisions is meant to be binding, it needs marking as such before the next ingest

[INFO] 10. Cross-reference cycle detection result
  Note: 26 nodes and 30 in-set edges, one more than the first run despite one fewer document, because docs/accessibility-framework-evaluation.md now points at docs/development/wxdragon-migration.md and because its `ACCESSIBILITY.md` reference resolves to docs/accessibility.md on this case-insensitive filesystem. Depth-first three-colour traversal found no cycles. The longest chain is 4 nodes, reached three ways, all ending the same: docs/getting-started.md (also docs/integration-guide.md, also docs/plans/20260726-mail-at-scale.md) to docs/IMPLEMENTATION_STATUS.md to docs/ALPHA_TESTING.md to docs/PROVIDER_SETUP.md. Well inside the depth cap of 50, so synthesis proceeded on the whole set

[INFO] 11. Cross-references pointing outside the ingest set were not followed
  Note: Referenced but not classified, so nothing from them reached synthesis: `CREDITS.md` and `sound-schemes/soft-chimes/` (from the earcon ADR); `src/data/message_cache/bodies.rs`, `src/service/protocols/imap.rs`, `src/application/mail_sync.rs` and `src/presentation/theme.rs`; `assets/brand/*`; `.github/workflows/release.yml`, `scripts/build-installer.sh`, `installer/Wixen-Mail-Setup.iss`, `scripts/make-brand.py`, `scripts/render_svg.py`, `scripts/make-icon.py`; and the eleven consolidated requirement documents named by docs/development/requirements-backlog.md (`PHASE8_*` through `ACCESSIBILITY_AUTOMATION_UIA_REQUIREMENTS.md`), which the PRD says previously lived in the repository root and which held whatever acceptance criteria existed. The PRD states no user stories and no acceptance criteria of its own, so every entry in requirements.md records `acceptance` as absent

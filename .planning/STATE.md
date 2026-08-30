---
gsd_state_version: 1.0
current_phase: 01
current_phase_name: Folders and conversations
status: executing
stopped_at: Completed 01-04-PLAN.md
last_updated: "2026-08-30T11:35:49.254Z"
last_activity: 2026-08-30
last_activity_desc: "01-04 done: rename, move and delete a folder, with the separator read rather than guessed"
state_head: 3f0e720574db1971c249b805fc8c3b61abb31687
progress:
  total_phases: 8
  completed_phases: 0
  total_plans: 13
  completed_plans: 4
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-08-29)

**Core value:** Making correspondence and personal information legible to people who cannot see it.
**Current focus:** Phase 01 — Folders and conversations

## Current Position

Phase: 01 (Folders and conversations) — EXECUTING
Plans: 13, one per wave, `01-01-PLAN.md` to `01-13-PLAN.md`. 38 tasks, of which
35 are RED-first, 1 is configuration-only (`guards/guards.toml` records) and 2
are blocking human gates, in 01-02 and 01-07, both over one-way writes to the
only copy of the user's mail. Those two plans are `autonomous: false`.
Status: Executing Phase 01

**Phase 1 reviewed 2026-08-29 with Pratik.** Two criteria changed:

- FOLDER-01 said all five folder operations pass through `Allowed::mail`. Wrong for POP
  accounts, which have no server folders at all, and for the IMAP outbox.
  `local_folders::is_local` already draws that line and is now named as the single place that
  decides it. Server folders keep the gate; local ones do not.

- FOLDER-03 keeps local pinning as this phase's work, and now says the stored shape must let
  IMAP subscription back it later, with the decision about which wins recorded before the
  second half is built rather than settled by whichever code path runs last.

**Phases 2 to 8 reviewed 2026-08-29.** Every `[D]` criterion was read back against the tree.
Two decisions were answered, two requirements were split, and six criteria were corrected for
being untestable as written. Recorded here because a criterion nothing can satisfy passes a
review that only reads it.

| Requirement | What was wrong | What it says now |
|-------------|----------------|------------------|
| SEARCH-01 | Said the scope selector is read by nothing, quoting a changelog line since corrected as false, and cited a test as a second production site | The live search honours all four scopes and has tests. A saved search keeps the folder half and not the field half, because `what_a_typed_search_asks` always writes `["subject", "from", "to"]` |
| SCALE-02 | Cited `opening.rs` and `attaching.rs`, which exist and open no connections | Names `a_session_at`, its three callers, and the eight sites in `wx_app.rs` that bypass it |
| FEEDBACK-01 | Asked for the removal of a `tests/house_style.rs` exception that does not exist; the real constant it echoed guards documents about a different setting | Asks for a test that counts the screens reaching `set_event_channels` |
| SHIP-01 | Required that SmartScreen stop warning, which a valid signature does not buy | Separates the signature, which is verifiable, from reputation, which only an EV certificate carries and EV is the last resort here |
| SHIP-05 | Mixed building on a platform with disclosing what does not work there | Split. SHIP-05 is the build and its CI jobs; SHIP-06 is the disclosure |
| PERF-06 | Asked that a number in a document equal what `cargo test` reports, which is false the next time anyone adds a test | Asks that every count carry its command and its date, and that documents agree with each other |

Two decisions were also answered in the same pass and are recorded under Blockers below:
PIM-04's notes backend, which split into PIM-04, PIM-07 and PIM-08, and SHIP-04's encryption
question, answered as a recorded decision not to.

Two things the review found that were not criteria at all. The traceability table had 40 rows
against 44 headings, having missed every requirement added by a split; it is now generated from
the headings so it cannot drift again. And three documents gave three different test counts, of
which the newest, 5,269, was the unit count wearing the label of the total. The suite is 5,430:
5,269 unit and 161 integration, from `cargo test --all-targets -- --list` on 2026-08-29.

Last activity: 2026-08-30 — 01-04 done: FOLDER-01's last two verbs, the inbox refused before anything is sent, and a delete that says how far it got

Progress: [███░░░░░░░] 4 of 13 plans in phase 01

## Performance Metrics

**Velocity:**

- Total plans completed: 2
- Average duration: 2h 7m
- Total execution time: 4h 15m

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**

- Last 5 plans: 01-01 (3h 5m), 01-02 (1h 10m)
- Trend: down sharply, and the reason is measurable rather than encouraging. 01-01
  ran the whole library on every check; 01-02 ran it four times on purpose and
  used targeted runs, 1 second against 188, for every red and green step.

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 01 P01 | 3h 5m | 3 tasks | 8 files |
| Phase 01 P02 | 1h 10m | 3 tasks | 9 files |
| Phase 01 P03 | 1h 0m | 3 tasks | 9 files |
| Phase 01 P04 | 4h 10m | 3 tasks | 16 files |

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

- [Phase 01]: A new folder is made where it is named, not under the cursor: the IMAP hierarchy delimiter is dropped after list_folders by design, and guessing it writes the folder elsewhere on a dot-separated server. Nesting is 01-03 and 01-05.
- [Phase 01]: Making a folder records it in the cache, subscribes it, and marks it as one to keep up to date. The tree is read from the cache, and an unsubscribed folder is hidden, so creating on the server alone would show nothing.
- [Phase 01]: MAIL_TRANSPORTS' imap floor rises with every gated write added. Left at 8 with nine writes present, it had already cost the copy_message gate guard one of its three reddening tests.

- [Phase 01]: D-39 confirmed at the gate by Pratik, conditional on no existing threading being lost. The condition was checked and holds: threading.rs is untouched, so the conversation view behind Enter is unchanged. thread_id is derived from the first identifier of the References chain, computed once when a message is stored.
- [Phase 01]: The conversation id is written at one call site, inside upsert_message, not at both named entry paths. file_message_here is upsert_message plus one UPDATE, so a second call there would build the duplication as_stored's doc comment forbids. Any later plan told to write a derived value "in both places" should read the second place's body first.
- [Phase 01]: A guard for an invariant of the form "exactly one place does this" breaks by ADDING a competing copy, not by deleting the one. Deleting proves only that the value is computed at all. guards.toml now holds one record of each shape for this column.
- [Phase 01]: 01-03: CachedFolder gained no parent_id field; the phase's consumers (01-04, 01-05) both take the folder_parents map, and the field would have been 79 struct-literal edits with no reader
- [Phase 01]: 01-03: is_a_name_that_can_be_used is asked of each part between separators, because safe_file_name reads a separator as a path, so asking it about the whole name would refuse the one character D-23 allows
- [Phase 01]: 01-04: rename_mailbox and delete_mailbox take the server's own spelling verbatim and encode nothing. A path from the cache is already modified UTF-7; only a segment somebody typed is encoded. The plan's instruction to encode both RENAME arguments would have named a mailbox the server has not got.
- [Phase 01]: 01-04: the hierarchy separator 01-03 read is not persisted, so no command running from the cache can see it. A rename reads it from the gap between a folder's path and its parent's; a move reads it off a LIST line on the worker, after the confirmation, so D-37 still holds.
- [Phase 01]: 01-04: folders kept on this computer are a const array of &'static str, so renaming, moving and deleting one are refused with a sentence rather than half-built. 01-06 builds user-named local folders.
- [Phase 01]: 01-04: a behaviour RED is taken by stubbing the body and reading which assertions fail. A compile error proves a symbol was absent, not that the assertions discriminate; four tests here stayed green against a stub returning nothing.

### Pending Todos

None yet.

### Blockers/Concerns

- ~~**Awaiting review, phases 2 to 8.**~~ Reviewed 2026-08-29. Every acceptance criterion
  marked **[D]** was derived by a model from the code and the status documents, not stated by
  Pratik or by a source, so each was read back against the tree. The review's own record, with
  what each correction changed, is under Current Position above.

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

- **Phase 1 grew in discussion, and the roadmap has not caught up.** Its five
  success criteria describe nesting a flat tree. `01-CONTEXT.md` describes one
  branch per account, a shared "On this computer" group whose folders belong to
  no account, a migration that moves existing mail between rows, five settings,
  and three IMAP verbs that do not exist yet (CREATE, RENAME, DELETE mailbox).
  Nothing is outside the phase's domain. Resolved 2026-08-29: the roadmap's
  Phase 1 criteria were rewritten from five to eight to match, and carry a scope
  note pointing at CONTEXT.md as the authority on the detail.

- **FOLDER-01 stays open until 01-09.** 01-01 built create; 01-04 built rename,
  move and delete. The requirement also asks for marking a folder read and
  emptying a folder, which are 01-07 and 01-09. Not ticked here either, for the
  same reason it was un-ticked after 01-01. `requirements mark-complete` ticked
  the whole requirement when 01-01 finished, because a plan names a requirement
  and the tool has no notion of a plan covering part of one. The tick was
  reverted. Any plan here that names a requirement four plans share needs the
  same check before its state update is believed.

- **THREAD-01 and THREAD-02 stay open after 01-02.** Same shape as FOLDER-01
  above, and checked before the state update rather than after. THREAD-01 asks
  for the message list to collapse to one row per conversation, which is 01-12.
  THREAD-02 asks for rethreading as mail arrives without the folder being
  rebuilt, which is 01-13. 01-02 built the thing both of them read, a stored
  conversation id. Neither was ticked.

- **The version bump rule has not been followed for 27 commits, and its check is
  vacuous.** CLAUDE.md says a behaviour change bumps the version in the same
  commit. Cargo.toml last moved 27 commits before 01-02, which includes all of
  01-01's create-a-folder feature. 01-02 bumped 0.45.0 to 0.46.0, so the bump is
  late and covers more than one plan. The nearest check,
  `test_no_status_page_names_a_version_the_code_does_not_ship`, compares versions
  named in README.md and docs/IMPLEMENTATION_STATUS.md against the shipped one,
  and neither file names a version, so it passes over an empty set. Its own
  comment recommends exactly that omission. This is Pratik's call, not a thing to
  fix inside a phase plan.

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

Last session: 2026-08-30T11:35:30.993Z
Stopped at: Completed 01-04-PLAN.md
Research found three things the discussion could not have known, and two of them
needed Pratik's answer: `messages.thread_id` is a column nothing writes and
nothing reads back, so D-08 had no key to span an account with, and the D-19
migration would have hit `UNIQUE(folder_id, uid)` collisions on the user's only
copy of that mail. Both answered and recorded as D-39 and D-40. A third,
the modified UTF-7 encoder, is new scope nobody had costed and is not optional.

01-01 is done and is the phase's tracer, so the shape it proved is what the
other twelve plans lean on. Three things it found that the plan had not: the
tree is read from the cache rather than the server, so a folder must be recorded
and marked as one to keep up to date or nobody sees it; a new mailbox is
unsubscribed and this tree hides unsubscribed folders, so it is subscribed too;
and the IMAP hierarchy delimiter is deliberately dropped after `list_folders`,
so a folder is made where it is named and nesting waits for 01-03 and 01-05.
A fourth is a warning for every later plan here: `MAIL_TRANSPORTS`' imap floor
must rise with every gated write added, because left at 8 with nine present it
had already taken one reddening test off the `copy_message` gate guard.
Resume file: None

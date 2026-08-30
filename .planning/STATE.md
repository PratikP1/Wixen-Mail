---
gsd_state_version: 1.0
current_phase: 01
current_phase_name: Folders and conversations
status: executing
stopped_at: Completed 01-07-PLAN.md
last_updated: "2026-08-30T17:38:14.488Z"
last_activity: 2026-08-30
last_activity_desc: "01-07 done: one Sent, Outbox, Drafts, Junk and Trash for every account, existing mail moved into them with a count said aloud, and colliding message numbers both kept"
state_head: c68feebf8ba092baab62caa8a4967e555e017bed
progress:
  total_phases: 8
  completed_phases: 0
  total_plans: 13
  completed_plans: 7
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

Last activity: 2026-08-30 — 01-07 done: the human gate was returned rather than answered and came back with three corrections, the five local folders are now one each, and the merge reuses the mover that already existed rather than the second one the plan asked for

Progress: [█████░░░░░] 54% (7 of 13 plans)

## Performance Metrics

**Velocity:**

- Total plans completed: 7
- Average duration: 1h 57m
- Total execution time: 13h 38m

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**

- Last 7 plans: 01-01 (3h 5m), 01-02 (1h 10m), 01-03 (1h 0m), 01-04 (4h 10m),
  01-05 (1h 38m), 01-06 (1h 4m), 01-07 (1h 31m)

- Trend: no trend, and the spread is the finding. The three fast plans used
  targeted test runs, 1 second against about 175, for every red and green step,
  and spent their full library runs only on measuring guard records by hand.
  The two slow ones were slow for different reasons worth telling apart: 01-01
  ran the whole library on every check, which is waste, while 01-04 was a large
  plan with four wrong premises to find, which is work. Duration alone cannot
  tell those two apart, so it is a poor measure of anything on its own.

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 01 P01 | 3h 5m | 3 tasks | 8 files |
| Phase 01 P02 | 1h 10m | 3 tasks | 9 files |
| Phase 01 P03 | 1h 0m | 3 tasks | 9 files |
| Phase 01 P04 | 4h 10m | 3 tasks | 16 files |
| Phase 01 P05 | 1h 38m | 3 tasks | 13 files |
| Phase 01 P06 | 1h 4m | 3 tasks | 15 files |
| Phase 01 P07 | 1h 31m | 3 tasks | 14 files |

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
- [Phase 01]: 01-05: the 01-03 regression was worse than its changelog said. folder_ids is a HashMap keyed on the row's displayed text, so two folders sharing a leaf were one entry and one of them opened the other's mail. Closing it meant keying on identity, not nesting the display.
- [Phase 01]: 01-05: selected_folder became the WhichRow enum rather than an identity string, so the compiler enumerates every consumer that read it as display text. Four did, and two of those were already broken: Get Older Messages and writing a mailbox out both passed the row's words where a folder path belonged.
- [Phase 01]: 01-05: wxdragon's TreeItemId has no PartialEq and no public pointer, so two tree items cannot be compared and a row can only be interrogated by its text. That is why this codebase was label-keyed. A row is paired to its identity by the chain of labels above it, which is unique because siblings cannot share a path.
- [Phase 01]: 01-05: no per-archive branch was built (D-21). Nothing records which archive an imported folder came from, so the plan's archives parameter has no producer. Building it is work in import_tree at import time; an enum variant nothing can construct would be a stub.
- [Phase 01]: 01-05: only the account being looked at gets a branch. D-13's property is proven of folder_tree::rows, which is multi-account throughout, but drawing every account at once before D-18 gives one Drafts, Sent and Outbox row per account, which is the duplicate-rows fault this plan removes.
- [Phase 01]: 01-06: the D-43 mirror guard was red on arrival naming six settings, and only allowed_per_account is the defect. It is stored, read by allowed_for and honoured out to the provider clients, and no screen has offered it since the testing page stopped naming an account. Named in an exception list with the reason rather than the guard being narrowed until it could not see it.
- [Phase 01]: 01-06: an exception list is the part of a check most likely to rot, so each exception carries a claim the check tests. One test reads the screen an exception names and fails if the control has gone; another fails when the recorded defect stops being one, so whoever fixes it is told to delete the entry.
- [Phase 01]: 01-06: the plan's unread_text would have had no caller and both settings guards would still have passed, because a module reading its own setting counts as a reader and a control counts as an offer. Reachability is a third question no test of the parts asks. rows() now takes the setting and what is closed, and the expand handler words that one row again.
- [Phase 01]: 01-06: accounts order by tree_order IS NULL, tree_order, created_at, so an untouched database keeps arrival order and an account added after a move goes to the end. The move writes every ordinal, not the two that swapped, because a list half ordered by choice and half by arrival reorders itself the next time an account is added.
- [Phase 01]: D-18 was self-contradictory about the Outbox and Pratik corrected it: shared for everyone, FOR_IMAP empty, one send queue on this computer
- [Phase 01]: The merge of the local folders reuses move_message rather than a second mover, because a separate one would have missed filed_here and written rows the next sync deletes
- [Phase 01]: The merge records both the original uid and the original account for every moved message, so it is reversible from the data even though no command undoes it

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
  otherwise, and may be rewritten to.

- 01-05 found two dialogs (wx_destination, wx_thread_view) hanging row data off the control, which wxdragon never frees for a leaf, and a spellcheck test that fails about one full library run in five through a Windows COM call made twice. Both are written up in .planning/phases/01-folders-and-conversations/deferred-items.md.
- 01-06 found allowed_per_account stored, honoured out to the provider clients and offered by no screen: the exact FEEDBACK-01 shape, already in the tree before the guard that found it. Named in STORED_AND_OFFERED_BY_NOTHING in src/data/config.rs and written up in .planning/phases/01-folders-and-conversations/deferred-items.md. Closing it is a per-account group on the settings screen.

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

Last session: 2026-08-30T17:37:23.723Z
Stopped at: Completed 01-07-PLAN.md
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

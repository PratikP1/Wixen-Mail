---
gsd_state_version: 1.0
current_phase: 02
current_phase_name: Search that says what it covers
status: executing
stopped_at: Completed 02-07-PLAN.md on branch gsd/plan-02-07, not merged
last_updated: "2026-09-01T10:47:54.559Z"
last_activity: "2026-09-01, 02-07 done: the second door is open. A saved search's conditions are edited from its own row in the folder tree and from the Saved Searches menu, and written back in one transaction on the way out. Both of 02-06's stub entries in WINDOWS.md are closed. Nothing has been opened in a running build, which is recorded rather than claimed"
state_head: 27d66c70668a0ec07c84ae00885226e324d76ce9
progress:
  total_phases: 9
  completed_phases: 0
  total_plans: 22
  completed_plans: 21
  percent: 0
last_activity_desc: "02-06 done: the writer and the condition dialog a rule editor needs are built and tested, and nothing in the running program opens either of them yet. That is 02-07's job and both are recorded as stubs rather than left to be found. The replace writes a search and its whole question list in one transaction, with the row stamped last on purpose, because stamping it first would make the only failure a person can cause fire before anything was destroyed and leave no test able to tell a transaction from three loose statements"
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-08-29)

**Core value:** Making correspondence and personal information legible to people who cannot see it.
**Current focus:** Phase 02 — Search that says what it covers

## Current Position

Phase: 02 (Search that says what it covers) — EXECUTING, 7 of 8 plans done (02-07 is on branch gsd/plan-02-07 and not merged).
Corrected on 2026-09-01: this header and the frontmatter both said phase 01
while five phase 02 plans had shipped, which is what made
`gsd-tools query state.advance-plan` fail and tagged new decisions `[Phase 01]`.
Phase 01 is complete and awaiting re-verification, recorded below; that is what
`current_phase` was being used to remember, and it is not what the field means.

Phase: 01 (Folders and conversations) — EXECUTED, awaiting re-verification
Plans: 14, one per wave, `01-01-PLAN.md` to `01-14-PLAN.md`. 40 tasks, of which
37 are RED-first, 1 is configuration-only (`guards/guards.toml` records) and 2
are blocking human gates, in 01-02 and 01-07, both over one-way writes to the
only copy of the user's mail. Those two plans are `autonomous: false`.
Status: All 14 plans executed. 01-14 was added on 2026-08-31 after the phase
verification recorded criterion 3 as the one partial of eight, and it closes it.
The phase wants re-verifying against that report, which is annotated as
superseded rather than left to be read as current.

**Phase 02 has context as of 2026-08-31** and is ready to plan. Phase 01 is left
Pending deliberately: its code is merged, pushed and green, and what keeps
FOLDER-02 open is a screen reader announcing a folder's level from the native
control, which no test here can answer and which is Pratik's to run.

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

Last activity: 2026-09-01, 02-07 done: the second door is open. A saved search's conditions are edited from its own row in the folder tree and from the Saved Searches menu, and written back in one transaction on the way out. Both of 02-06's stub entries in WINDOWS.md are closed. Nothing has been opened in a running build, which is recorded rather than claimed

Progress: [░░░░░░░░░░] 0%
percentage above counts phases and this line counts plans, so the two have said
different things about the same work all through this phase; the plan count read
7 while 11 were done, which is how long a number nothing recomputes can sit here
being wrong.

The suite is 5,958 unit as of `bash scripts/check.sh all` on 2026-09-01, with
one ignored and every other target green in the same run. 02-07 added 30 of
those, across `presentation::manager_words`, `presentation::wx_managers`,
`presentation::folder_tree`, `presentation::wx_app` and
`application::context_menu`, plus four checks inside the two window-building
integration targets.

The counter above could not be advanced by `gsd-tools query state.advance-plan`
on 2026-09-01: it looks for "Current Plan" and "Total Plans in Phase" lines this
file does not have, so it reported that it could not parse them and changed
nothing. These two lines were written by hand instead. That is the same fault
the paragraph above describes, seen from the tooling's side.

## Performance Metrics

**Velocity:**

- Total plans completed: 14
- Average duration: not recomputed; the figures below predate 01-10 and nothing
  recalculates them, which is the same fault the progress note above records

- Total execution time: not recomputed, for the same reason

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**

- Last 8 plans: 01-01 (3h 5m), 01-02 (1h 10m), 01-03 (1h 0m), 01-04 (4h 10m),
  01-05 (1h 38m), 01-06 (1h 4m), 01-07 (1h 31m), 01-08 (2h 5m),
  01-10 (2h 40m)

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
| Phase 01 P08 | 2h 5m | 3 tasks | 13 files |
| Phase 01 P09 | one session | 3 tasks | 11 files |
| Phase 01 P11 | 3h 5m | 3 tasks | 12 files |
| Phase 01 P12 | 4h 5m | 3 tasks | 17 files |
| Phase 02 P03 | one session | 3 tasks | 7 files |
| Phase 02 P04 | one session | 3 tasks | 7 files |
| Phase 02 P05 | one session | 3 tasks | 6 files |
| Phase 02 P06 | one session | 3 tasks | 5 files |
| Phase 02 P07 | one session | 3 tasks | 13 files |

## Accumulated Context

### Decisions

Decisions are logged in the PROJECT.md Key Decisions table. The ones that shape the phases
ahead:

- No EWS. Microsoft blocks third-party EWS from 1 October 2026. Exchange goes through Graph.
- Writes split into `mail` and `personal_information` in `src/application/allowed.rs`, with
  three places that must agree. Mail writes are off for a new install.

- The message list stays native virtual mode, because only the native control gives UI
  Automation the real set size.

- The folder tree holds every account at once, and whose mail a command acts on is taken from
  the row under the cursor rather than from a separately held "open account". Those two were
  the same answer while the tree drew one account's folders, and stopped being the moment it
  drew them all (01-14).

- Moving between accounts must stay a selection rather than a rebuild, guarded by a record
  whose break is the wrong fix somebody would reach for. Rebuilding on selection would make
  everything downstream agree and would cost five cache reads per account on every arrow key.

- The cached mail database is not encrypted, and the docs say so. Phase 7 decides whether that
  changes.

- Two windows that must not open over each other share one gate rather than holding one each.
  `application::due::OneAtATime` now gates the reminder alerts and the question about folders a
  server has stopped listing, because a gate each is exactly what lets either open over the
  other.

- What a server last said about a folder is three answers, not two: it listed it, it stopped
  listing it, and it stopped listing it and somebody said keep it. Without the third, answering
  No and closing the window are the same thing.

- A pin and a server subscription are two questions, so neither overrules the other. Pinning
  never writes a subscription and a subscription changing never adds or removes a pin. Recorded
  in `src/application/favourites.rs` and in PROJECT.md before the storage shape was fixed, which
  is what FOLDER-03 asked for.

- Favourites are keyed on `(account_id, path)` with both cascades. A rename rewrites a folder's
  path, so `ON UPDATE CASCADE` is what makes D-32's "a rename keeps the pin" true rather than a
  second writer nobody remembers.

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
- [Phase 01]: Emptying asks both functions that decide what deleting means, local and server, and carries all their answers across; a single AtTheServer variant was written first and would have destroyed an Inbox
- [Phase 01]: The empty count and the empty walk both skip messages already soft-deleted, so running Empty twice is a no-op and an emptied Trash stops reading as full
- [Phase 01]: A setting ships in one commit with its screen and its consumer; splitting them leaves the two settings guards red with no honest way to satisfy them
- [Phase 01]: A conversation is named by its oldest message present, from mail_parser's RFC 5256 base subject, and the compose box asks the same module whether a subject already carries a marker
- [Phase 01]: The Subject column's conversation sort calls a Rust function registered on the SQLite connection, because a chain of markers in seventeen languages has no SQL expression
- [Phase 01]: conversations_in takes an order_by so the sort expression has a caller and the agreement between a cell and its sort is run rather than described
- [Phase 01]: The all-mail exclusion applies only to the account-wide reach, because counting one folder cannot double anything
- [Phase 01]: 01-12: the message list collapses to one row per conversation, per folder, and Thread View is no longer a disabled menu item
- [Phase 01]: 01-12: opening a folder row no longer deletes its tree_state row outright, because the view D-09 stores there would have gone with it the first time somebody expanded the folder
- [Phase 01]: 01-12: the count the virtual list is told has one writer, because that number is the set size UI Automation reports and a second writer is a wrong announcement nobody can see
- [Phase 01]: 01-12: the selection is held across a view switch rather than recomputed, because a conversation cannot say which of its messages was chosen
- [Phase 01]: 01-12: a subtree is read from parent_id and never from a path, because the hierarchy separator is not persisted
- [Phase 01]: 01-12: a conversation row confirms a delete where a single message does not, because its contents are off screen and the column that would say how many can be switched off
- [Phase 01]: 02-03: the offer to fetch missing message text counts the fetch list, not the difference between the two coverage numbers, because mail with no server to ask is missing text that no fetch can supply
- [Phase 01]: 02-03: a read-gate refusal stops a backfill and an ordinary failure does not, told apart by service::outward::was_refused_by_the_gate
- [Phase 01]: SEARCH-01 is met by keeping the In box's answer beside the typed words and writing both halves of the scope in one call: D-2-03's narrower question set and D-2-14's folder, with no schema change
- [Phase 01]: A folder stored on a saved search carries the account it belongs to, and saving one whose account disagrees with the search's is refused rather than saved without the folder, because Set Active can change one and leave the other
- [Phase 01]: A saved search's whole description is written back in one transaction, with the questions written first and the row stamped last, so the one failure a person can cause lands inside the window the transaction protects rather than outside it
- [Phase 01]: The two things a saved search misses stay two sentences, with a test asserting they differ: mail thrown away is never gathered, and evicted message text stays findable from the search box and not here (D-2-13)
- [Phase 01]: No changelog entry for 02-06, because nothing it builds is reachable; the entry belongs to 02-07, which opens the dialog and calls the replace
- [Phase 02]: 02-07: a saved search's row reports its own focus on the menu key, rather than an entry being filtered out of the folder list. Nothing in the codebase decided menu entries per row.
- [Phase 02]: 02-07: a search a newer version wrote keeps the Edit conditions entry and is refused with a sentence, in the wording Enter on the same row already gives.
- [Phase 02]: 02-07: every change to a condition list says how many are left, on the end of the one sentence. Only a condition list counts out loud, decided from the kind rather than passed in.

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

Last session: 2026-09-01T10:47:54.524Z
Stopped at: Completed 02-07-PLAN.md on branch gsd/plan-02-07, not merged
verification found

01-14 built the multi-account folder tree. Three things worth carrying.

**The plan's account of the code was wrong in a way that would not have shown
up.** It said twelve call sites divide into "the data changed" and "the account
being looked at changed", and that the second kind must stop rebuilding. There
are eleven, and all eleven are the first kind: nothing rebuilt the tree in
response to an account switch, because nothing changed the looked-at account in
a way that reached the tree. Carrying out the instruction faithfully would have
meant classifying eleven sites, changing none, and reporting the criterion met.
Count the members of every class a plan names, and treat zero as a finding.

**Making hidden state visible turns every writer of it into a staleness bug.**
Two fell out of task 1 and both are fixed: moving an account wrote the ordinal
and redrew nothing, and selecting a folder read whichever account was open
rather than the folder's own. Both were correct while the tree drew one account
and became wrong the moment it drew them all. The plan enumerates readers; the
new defects were in the writers.

**Eleven source-reading checks in this tree cut a file at the first
`#[cfg(test)]` and keep what is above.** That is "the file up to the first test
module", not "the half that ships", and they agree only while every test module
sits at the end. One was in `wx_app.rs` and now uses `what_ships`. Ten are in
`tests/wired.rs` and cannot, because `what_ships` is `#[cfg(test)]` and an
integration test links the library built without it. Four of those ten fail
loudly when they narrow; six pass in silence over a third of the file. Written
up in the phase's deferred items, with the three possible shapes of a fix.

The cost of the change is stated in the summary and the changelog rather than
buried: every one of the eleven redraws now reads five things per account
instead of five in all, and syncs run on a timer.

Before 01-14, this said:

01-13 closed THREAD-02 and the phase. Four things it found that the plan had
not, and the first two matter to anybody writing over this code.

The plan's own order-independence criterion is unsatisfiable with the signature
the same task mandates: the arrival lookup can see messages the arriving one
names, never messages that name it. So the merge runs in one direction. A late
message connecting two conversations merges them, which is what THREAD-02 asks
for; a conversation root arriving after a message that already named it does
not, and closing that needs an identifier-to-conversation table, which is a
schema decision this plan did not carry.

`messages.message_id` holds two spellings and `messages.thread_id` holds one:
mail through `mail_parser` is stored bare, a draft this program composes keeps
its angle brackets, and the derived column always strips. A new lookup joining
them found nothing, and the symptom read exactly like a wrong test fixture.

The lookup has to ask which conversation an identifier is *in*, not whether it
is the root of one. An ancestor named in a chain is usually in the middle of a
conversation rather than at its head, so asking about roots misses the common
shape.

Guard record "the column that says which conversation a message is in has a
writer" was re-measured for the fourth time in two days, now at 31 tests, one of
them in `wx_app`. It goes stale every time anything near it is touched, which is
the bidirectional check earning its keep.
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

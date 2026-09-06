# Phase 5.1: Notes and contacts reach a server for the first time (INSERTED)

Six plans, one per wave. Assembled 2026-09-06 against `main` at `9611b70`,
version `0.75.0`, `guards/guards.toml` holding 632 records, Rust floor `1.88`.
Nothing in the repository was changed while these were written.

This is an inserted phase, following the precedent of
`02.1-what-phase-1-found-on-its-way-past`, which is tagged `(INSERTED)` in the
roadmap and whose files are named `02.1-08-SUMMARY.md`. It is the second of three
phases the old nine-plan phase 5 was cut into.

**Goal.** Notes reach a server through one seam that knows nothing about which
backend it is talking to, and contacts get a second address book by its own
address.

**Requirements:** PIM-04, PIM-05, PIM-07, PIM-08.

**Roadmap success criteria this phase owns:** 6 in full, and the CalDAV half of 4
and 5. The OneNote half of 4 and 5 goes to phase 5.2.

## Why this is a phase of its own

Four reasons, in the order they decided it.

**The file sets barely touch.** The move half of the old phase lives in
`managers.rs`, `pim_command.rs`, `context_menu.rs`, `destinations.rs`,
`new_item.rs` and `ui_types.rs`. This half lives in `notes.rs`, `carddav.rs`,
`caldav.rs`, `contacts.rs` and `contacts_sync.rs`. One file appears in both,
`managers.rs`, and only as a call site. Two phases here are two branches that do
not fight.

**The evidence is available in one half and not in the other.** Everything in the
move half can be measured on this computer: a row is in one list or two, a
contact is in one group or another, a sentence is said or not. Nothing here can
be finished on this computer, because no account, no CalDAV server and no CardDAV
server has ever been used with this program. Mixing them puts a phase's verdict at
the mercy of the half nobody can verify.

**Each half is a release somebody can use.** After phase 5 a person can move and
copy in all five modules and can move a task a provider holds without losing it.
After this phase notes reach a server and contacts have a second address book.
Neither is a fragment of the other.

**The goal line splits cleanly, and decision 4 is what makes it split.** The
roadmap goal is "Contacts, calendar, tasks, notes and reminders support the same
moves mail already does, and the two that currently go nowhere get somewhere to
go." Before 2026-09-06, "the two that go nowhere" was notes and reminders, and
reminders had nowhere to go at all. Decision 4 gives reminders somewhere to go,
and it is an account, which is the move half. So the first clause and reminders
belong to phase 5, and notes belong here.

## The plans

| Plan | Wave | Requirements | Depends on | Human | What it does | Source |
|---|---|---|---|---|---|---|
| 05.1-01 | 1 | PIM-04 | none | no | A note's stored form says what it is, and the byte-identical round trip gets a test that goes through storage | carried, was `05-05` |
| 05.1-02 | 2 | PIM-07, PIM-08, PIM-04 | 05.1-01 | no | One place decides where a note goes, the contract a backend must answer, the settings sentence | carried, was `05-06`, checkpoint struck |
| 05.1-03 | 3 | PIM-07 | 05.1-02 | no | CalDAV VJOURNAL behind the seam, its columns, and the deletion record without which a deleted note comes back | carried, was `05-07` |
| 05.1-04 | 4 | PIM-08 | 05.1-03 | no | A second implementation, in tests only, shaped from the four things OneNote really does, so the seam meets them before Graph does | new |
| 05.1-05 | 5 | PIM-05 | none | no | The per-card reader and writer lifted out so both callers share them, and the two CardDAV parsers, all pure | carried, was `05-08` |
| 05.1-06 | 6 | PIM-05 | 05.1-05 | yes, at the end | The screen, the table, the sync into the merge that exists, the credential owner, then a real screen reader pass | carried, was `05-09` |

Requirement coverage: PIM-04 by 05.1-01 and 05.1-02. PIM-05 by 05.1-05 and
05.1-06. PIM-07 by 05.1-02 and 05.1-03. PIM-08 by 05.1-02 and 05.1-04.

PIM-04 and PIM-07 are also touched by phase 5.2, which is deliberate and is said
in both places: 5.2 is the plan set that finds out whether the seam this phase
built was really written about CalDAV.

Waves 1 to 4 are the notes chain and waves 5 to 6 are the contacts chain. The two
chains share no source file, so 05.1-05 could run in wave 1 beside 05.1-01. It
does not, because both write `guards/guards.toml`, `docs/changelog.md` and
`Cargo.toml`, and every plan here serialises on those three whatever else it
touches.

## What Pratik answered, so no plan here asks it again

The old `05-06` carried a checkpoint asking which notes backend goes first and
whether one arm proves PIM-08. It is struck.

**Decision 2 of 2026-09-06: all three.** CalDAV VJOURNAL, a second
implementation to prove the seam, and OneNote. Pratik's words: "We need full
support depending on the service people use." So `05.1-03` is CalDAV VJOURNAL,
named rather than conditional; `05.1-04` is the second implementation and it
exists only in tests; and OneNote is phase 5.2.

`05.1-04` is not a fake written the same day as the interface. A second
implementation written in the same week as the seam, by the same person, against
the same assumptions, tends to agree with the interface. This one is shaped from
the four constraints OneNote really imposes, which are below with their sources,
so it disagrees where a real second backend would. Phase 5.2 then replaces it
with the real client and is required to report where it was wrong.

### What the rejected options lost on

The checkpoint was struck from the plan, and that would have thrown away the most
expensive part of it: the reasons the other answers lost. They are kept here,
with the date and the fact that decided each, so a later reader can tell a
settled decision from an unexamined assumption.

**Decision 2, which notes backend goes first.** Answered 2026-09-06: all three.
Four options were on the table and the answer took the second and then went past
it.

| Option | Taken | What it lost on |
|---|---|---|
| CalDAV VJOURNAL, one arm, and PIM-08 rests on the written contract | partly, it is `05.1-03` | By far the cheapest: the transport, the ETag handling, the conflict raising in `caldav_sync.rs` and the document reader and writer all exist and are already guarded by `caldav.rs`'s `FILES_THAT_READ_OR_WRITE_A_DOCUMENT`. It is the only candidate whose round trip can be byte-identical, which PIM-04 requires. As the whole answer it lost because it reaches only accounts with a CalDAV server, which is nobody's Gmail and nobody's Outlook, and because PIM-08 would rest on one worked example rather than two. |
| CalDAV VJOURNAL, and a second implementation that exists only in tests | partly, it is `05.1-03` plus `05.1-04` | Answers PIM-08 the way it asks to be answered, by writing a second thing against the seam and finding out where the seam was shaped to the first. `tasks_sync.rs`'s `Scripted` fake is the precedent. Its stated weakness stands and is why the second implementation is shaped from OneNote's measured constraints rather than invented: a fake written by the same person on the same day tends to agree with the interface, which is the weakness `04-09` names about a key and a message made by one crate. |
| OneNote first, so the hard mapping is met now rather than later | no, but it is phase 5.2 | Reaches Outlook accounts, which is a real population, and finding the mapping problem early is better than finding it after a seam was shaped around a backend where the mapping was free. It lost as a *first* backend because `new_item.rs` already argues against it, it cannot honour PIM-04's byte-identical criterion, and the mapping is a product decision rather than an afternoon. That last cost is exactly why it became a phase rather than being dropped. |
| No backend this milestone; the seam and the contract are the deliverable | no | Nothing would ship that has never met a server, and PIM-07 would be honestly reported as not closed. It lost because PIM-07 is the largest named build here and dropping it makes the phase substantially smaller than the roadmap says, and because it leaves the seam unproven: an interface with no implementation is a guess, and the first implementation always finds something. |

Pratik's words on the answer: "We need full support depending on the service
people use." Add a phase if necessary. That is what happened.

One checkpoint remains, at the end of `05.1-06`, and it is carried unchanged: a
screen reader pass over the new address book screen. That is the one thing in
this phase no test in this repository can settle.

## The four things OneNote really does, which `05.1-04` is shaped from

Read on 2026-09-06 from Microsoft's own reference, with the pages named so they
can be re-read rather than believed. They matter here, one phase before OneNote
arrives, because `05.1-02`'s contract has to be written so a backend can answer
them differently without the seam changing shape.

- **`onenotePage` has no ETag and no `eTag` property.** Its properties are
  `content`, `contentUrl`, `createdByAppId`, `createdDateTime`, `id`,
  `lastModifiedDateTime`, `level`, `links`, `order`, `self`, `title`. There is no
  `If-Match` anywhere in the update reference. So OneNote's concurrency is a
  timestamp the service owns, where CalDAV's is an ETag and where
  `contact_identities` already holds a `provider_version` column.
  (`learn.microsoft.com/en-us/graph/api/resources/onenotepage`)
- **A page's body cannot be replaced.** The supported-actions table gives `body`
  append: yes, replace: no, insert: no. To make a page's body equal a newly
  rendered document you delete its elements one at a time by their generated ids
  and append new ones, or you delete the page and make another one, which changes
  its identity, its position and its links.
  (`learn.microsoft.com/en-us/graph/onenote-update-page`)
- **Most replaces need the generated id, and generated ids move.** The same page
  says a `replace` needs the id Graph generated, not a `data-id` you set, for
  everything except the title and images inside a div, and that generated ids
  "might change after a page update, so you should get the current values before
  building a PATCH request". So every write is preceded by a read of
  `../pages/{id}/content?includeIDs=true`.
- **The hierarchy is four levels, not three.** notebook, sectionGroup, section,
  page. (`learn.microsoft.com/en-us/graph/api/resources/onenote-api-overview`)

The codebase reached a version of this conclusion before anybody planned it.
`src/application/new_item.rs:19` to 23 says a OneNote page "is an HTML document
inside a section inside a notebook rather than a title and a body, so the mapping
is a decision somebody has to make rather than an afternoon's work".

## Premise corrections that must survive into execution

**There is nowhere to store a CardDAV address book's address, and
`05-RESEARCH.md` says the opposite in a way that is half true.** The research
says `AddressBook::Other("carddav")` already round-trips, so no schema change is
needed to name the new address book. Naming it is not the problem. There is no
table, and no column anywhere, that can hold the address a person types, its
credentials owner, its change marker or whether it is visible. `05.1-06` builds
that table and says so; the research's sentence is true about the name and false
about everything a sync needs.

**`toggle_note_pin` does not go through `save_note`.** `save_note` is one upsert
of the whole row, and `toggle_note_pin` is a direct `UPDATE`. Any invariant
enforced only in `save_note` is bypassed by it. That does not bite in `05.1-01`,
because pinning does not change a body. It bites hard in `05.1-03`, which puts a
`pending` flag on a note, and it is the shape of the "moved but nothing was sent"
bug `docs/changelog.md` already records. `05-RESEARCH.md` assumption A5 names
three `save_note` callers and does not name this one.

**`file_under` does not set `pending` for a note, because a note has no such
column.** Once `05.1-03` gives it one, that arm has to set it or a moved note is
never sent. Phase 5's `05-03` builds copy for a note before that column exists
and says so plainly rather than pretending otherwise.

**`notes.format` is written and never consulted.** `05-RESEARCH.md` says it is
read at `outlook_data_file.rs:3260` only. Line 3268 is
`assert_eq!(note.format, "plain");` inside a `#[cfg(test)]` block, so the only
read of the value anywhere is a test asserting the constant the writer wrote.
`05.1-01` is the plan that makes it mean something or retires it honestly, and
dropping it is not available: `CLAUDE.md`'s schema rule is that changes are
additive and a column that shipped is never dropped or renamed.

**There is no `deleted_notes` table.**
`grep -n "CREATE TABLE IF NOT EXISTS deleted" src/data/message_cache/mod.rs`
finds `deleted_contacts`, `deleted_calendar_events` and `deleted_tasks`, and none
of them is notes. `application::deletions` states the rule for the other three.
Without it the first backend rediscovers the exact bug that module was written
about: a note deleted here comes back on the next read, under the backend's own
identifier, with nothing left saying it was ever deleted. `05.1-02`'s contract
requires it and `05.1-03` builds it.

**"All three backends" does not mean every account gets one.** After CalDAV
VJOURNAL, the test-only second implementation and OneNote all ship, a consumer
Gmail account still keeps its notes on this computer, because Google Keep's API
is Workspace only and `new_item.rs:22` says so. PIM-04's criterion that the
settings screen says notes do not sync yet rather than offering a switch that
does nothing still has real work in it after all three, and `05.1-02`'s settings
sentence is written so it survives them: "this account has no notes backend"
rather than "not yet".

**Two green tests assert the opposite of a notes backend.**
`test_notes_are_not_offered_a_sync_they_cannot_do` at `context_menu.rs:611`
asserts the note folder menu has no `Action::SyncNow`.
`test_notes_and_reminders_still_stay_on_this_computer` in `new_item.rs`, around
line 440, asserts `!supports(account, ItemKind::Note)` for `me@gmail.com` and
`me@outlook.com`. A OneNote backend inverts the outlook half of that loop and
leaves the gmail half true. `05.2-03` names both; this phase changes the first
one when the seam starts answering.

## Costs every plan is written around

**Guard records, re-measured at `9611b70`: 632 records.** The old drafts of these
plans said 617. Phases 4 and 4.2 landed in between. Count records rather than
mentions, with the awk in `CLAUDE.md`; a grep for `contacts_sync` answers 363
against 77 real records.

| file | records | test functions |
|---|---|---|
| `src/application/contacts_sync.rs` | 77 | 281 |
| `src/presentation/wx_app.rs` | 42 | 199 |
| `src/presentation/managers.rs` | 40 | 137 |
| `src/data/message_cache/contacts.rs` | 34 | 103 |
| `src/service/caldav.rs` | 29 | 163 |
| `src/application/caldav_sync.rs` | 24 | 67 |
| `src/application/long_text.rs` | 18 | 38 |
| `src/data/message_cache/mod.rs` | 11 | 23 |
| `src/service/outward.rs` | 10 | 38 |
| `src/data/config.rs` | 2 | 53 |
| `src/application/new_item.rs` | 1 | 42 |
| `src/application/forget.rs` | 1 | 21 |
| `src/presentation/wx_settings.rs` | 1 | 0 |
| `src/application/context_menu.rs` | 0 | 15 |
| `src/application/conflict_choice.rs` | 0 | 17 |
| `src/application/deletions.rs` | 0 | 4 |
| `src/data/message_cache/notes.rs` | 0 | 7 |
| `src/service/outlook_data_file.rs` | 0 | 38 |
| `src/service/carddav.rs` | does not exist yet | 0 |
| any new file | 0 | 0 |

Three figures moved since the drafts and are corrected in every plan that quotes
them: `wx_app.rs` 40 records to 42, `config.rs` 52 tests to 53, and
`forget.rs` 0 records and 18 tests to 1 and 21. That last one matters:
`05.1-06` was written saying `forget.rs` is free and putting its task 3 tests
there, and it now costs one record. One is still cheap, and the plan now says the
real number and says that it moved.

**The census at the top of `guards/guards.toml` moved with it.** Line 79 says 192
and line 80 says 440, and 192 plus 440 is 632.
`test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
adds them and compares, inside the commit gate, so a plan adding a record bumps
line 80 in the same commit. Read both lines before trusting either.

**Every figure in this table is a measurement with a date, and phase 5 runs
first.** `05.1-01` and `05.1-05` open on a tree eight plans past this
measurement. Re-measure rather than trusting the table.

**The Rust floor is 1.88, raised by `04-09` from 1.87 because rPGP declares it.**
Clippy only suggests a construct once the declared floor allows it, so the bump
turned on `collapsible_if`'s let-chain form and cost seven fixes across six files
that plan never opened. `main` is clean at that floor, so nothing is owed. New
code here is linted against 1.88 with `-D warnings`, where a let-chain suggestion
is a build failure and not a hint.

**A commit touching `Cargo.toml` makes `scripts/which-checks.sh` answer `all`**,
the whole gate at roughly 311 to 353 seconds warm. Run those commits detached and
never pipe `check.sh` into anything whose exit status is then read.

**A red commit is allowed only on a branch**, must name every failing test in
`Fails-until-green:` trailers at column 0, and every named test must have run and
failed with nothing else failing. Where adding a test makes the count check red
at the same time, name that check as one of the failures.

**`WIXEN_TEST_THREADS=4`** halves a guard run and does nothing for `check.sh`.

**A guard living in `tests/` needs a record with `suite = "<file name without
.rs>"`**, or it runs on every commit except the ones that could break it.

## Estimates

Carried plans keep their `estimate` blocks except `05.1-02`, whose checkpoint was
struck; it keeps three tasks because the checkpoint was a fourth block rather
than a task with work in it. New plans use the same derivation: `raw_tokens` is
35,000 per task, the mean raw projection per task across the nine phase-4 plans,
and `tokens` is that multiplied by 1.03, the mean of
`actuals.tokens / estimate.raw_tokens` over the six phase-4 plans with recorded
actuals (1.17, 1.29, 1.29, 0.48, 1.18, 0.78). Six samples with that spread is
`med` confidence and not `high`. Re-derive once phase 5 has actuals.

## What no plan in this phase can close

- **No account, no calendar server and no CardDAV server has ever been used with
  this program.** Every parser in `05.1-03` and `05.1-05` is tested against text
  this repository wrote. A round trip proves a reader and a writer agree with each
  other and says nothing about anybody else's server.
- **Whether the notes seam really takes a second implementation.** `05.1-04`
  answers it with a second implementation shaped from OneNote's real constraints,
  which is more than a same-day fake and less than a client. `05.2-03` answers it
  properly and is required to report where `05.1-04` was wrong.
- **Whether a note sync summary is distinguishable from the three others when
  several run together**, and whether a held note conflict read aloud is
  answerable without seeing both copies.
- **The address book screen by ear.** `05.1-06`'s checkpoint covers it, and it is
  the one thing here a human has to do.

`.planning/WINDOWS.md` ends at entry 119 before this phase. Each plan says which
entries it owes, one entry per unrun thing rather than one entry for all of them,
because an entry saying "this has never been tried" tells the next reader nothing
about which part to try first.

## What is owed to documents, and belongs to whoever lands these

1. **A roadmap entry**, `### Phase 5.1: Notes and contacts reach a server for the
   first time (INSERTED)`, following the shape of the 2.1 entry: goal, depends
   on, requirements, success criteria, plan list.
2. **The success criteria this phase owns** are criterion 6 of the roadmap's
   current phase-5 list in full, and the CalDAV half of criteria 4 and 5. Write
   them as this phase's own criteria rather than cross-referencing phase 5's.
3. **PIM-04's last three `[D]` lines belong to PIM-07 and to this phase**, and
   `05.1-01` says so in its summary rather than letting the requirement read as
   closed by the format column alone.
4. **Nothing in this phase corrects a requirement ahead of the code.** Where a
   requirement wants rewording after a plan makes it true, the plan that made it
   true does the rewording, for the reason this project keeps rediscovering.

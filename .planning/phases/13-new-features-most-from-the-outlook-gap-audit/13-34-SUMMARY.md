---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 34
subsystem: mail sending and drafts
tags: [outbox, drafts, schema, identities, GAP-10]
requires: [13-33]
provides: [from_address and from_name on the Outbox and drafts, from_queued and the filed draft through who_it_goes_out_from]
affects: [13-35, 13-36]
tech-stack:
  added: []
  patterns: [additive columns read by name, one row reader for both draft reads]
key-files:
  created:
    - tests/a_message_goes_out_from_the_address_it_was_written_from.rs
  modified:
    - src/data/message_cache/mod.rs
    - src/data/message_cache/outbox.rs
    - src/data/message_cache/drafts.rs
    - src/data/message_cache/accounts.rs
    - src/application/mail_controller.rs
    - src/application/draft_message.rs
    - src/application/protecting.rs
    - src/presentation/wx_app.rs
    - tests/integration_tests.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "The filed draft goes through draft_message::the_copy_to_file(draft, account), which asks who_it_goes_out_from, so the target reaches the function the main window calls"
  - "NULL means the account's own address and name; no backfill"
metrics:
  duration: about 75 minutes
  completed: 2026-09-29
status: complete
actuals:
  tokens: 24000
  tasks: 3
  commits: 5
---

# Phase 13 Plan 34: The Outbox and drafts keep the address a message was written from

A queued message and a draft now carry `from_address` and `from_name` in four additive
columns, and a message leaves, and a draft is filed, from the address its row names through
`identities::who_it_goes_out_from`; a row naming none goes out from the account's own, which is
every row today, because nothing writes an address until 13-35.

## What works, and how it is known

- **The columns.** `ensure_column_exists` adds `from_address` and `from_name` to `outbox_queue`
  and `drafts` (`src/data/message_cache/mod.rs`). The queue and the draft save write them; the
  Outbox read and both draft reads read them back by column name. Both draft reads now go
  through one row reader, `a_draft`, over one column list, `DRAFT_COLUMNS`.
- **Sending.** `SendEmailRequest::from_queued` takes the From through `who_it_goes_out_from`
  with the row's address and name. The send loop's `protected_as_asked` gathers keys for
  `request.from_address`, so it now gathers for the row's address (ledger 660's send-loop half).
- **Filing a draft.** `draft_message::the_copy_to_file(draft, account)` asks the same rule;
  `file_draft_copy` in `wx_app.rs` calls it in place of the lines that built the sender there.
- **Held by** `tests/a_message_goes_out_from_the_address_it_was_written_from.rs`, 9 cases over a
  real cache on disk: a queued row and a draft with `help@example.com` and "Help Desk" read back
  with both, by the draft's opening and its listing; rows without read back `None`; a database
  made by hand in the tables' first shape, with a row in each, opens and reads both rows with
  `None`; `from_queued` then `outgoing` builds a message from the row's address and name, and
  from the account's for a row naming none; the filed copy's From line says the row's address
  and name, and the account's for a draft naming none.
- **Readings at the last green:** the target 9 of 9; `data::message_cache::outbox::` 27;
  `data::message_cache::drafts::` 8; `application::mail_controller::` 71; `application::draft_message::` 18;
  `integration_tests` 26; `the_feedback_dialog_shows_what_it_sends_before_it_goes` 10;
  `signing_and_encrypting_from_the_composer` 20. Each module's count is the count before.
- **Taken red by hand:** without `ensure_column_exists("outbox_queue", "from_address", ...)` the
  migration case fails, and so do the fresh-cache Outbox cases, because the column is not in
  the `CREATE TABLE` either.
- **Nothing a person does changes.** Every production literal passes `None`, so every message
  and draft still goes out from its account's own address and name. Nothing spoken or shown
  changed, so the branch was merged without a pull request.

## Commits and hook times

| Commit | What | Hook |
|---|---|---|
| `ffafff32` | test: the fields on stubs, every literal `None`, 2 failing cases of 5 | red, 235 s; refused once at 51 s by clippy (an unused fixture) |
| `2d26ea80` | feat: the four columns, the writes, the reads by name; 1 record | affected, 226 s |
| `8b70c5c0` | test: `the_copy_to_file` at today's behaviour, 2 failing cases and the count check | red, 220 s |
| `a4d851e9` | feat: `from_queued` and `the_copy_to_file` through `who_it_goes_out_from`; 2 records | affected, 215 s |
| docs | the changelog, the ledger, 13-35's premises, this summary, the marks | this commit |

## Guard records

| Record | Break | Red |
|---|---|---|
| a queued message keeps the address it was written from (`outbox.rs`) | the queue writes `None` for `from_address` | the queued round trip and the queued send case |
| a queued message goes out from the address its row carries (`mail_controller.rs`) | `from_queued` passes `None` for the row's address | the queued send case |
| a filed draft is from the address its row carries (`draft_message.rs`) | `the_copy_to_file` passes `None` for the draft's address | the filed draft case |

All three take `suite` = the new target. Measured in three `--remeasure` calls: the first record
alone at task 1's green, 36 s; the three at task 2's green, 64 s, which found the first record
short by the send case; that record again after its red list was corrected by hand, 37 s. The
arrived-since count went from 468 to 471. The `:205` record in `mail_controller.rs` ("one queued
message keeps one identifier") anchors on `message_id::derived(&req.queue_id, &req.from_address)`,
untouched. The drafts record anchoring on `draft.protection.as_stored(),` still names one place.
No test was added to a file an older record names, so the count check flagged only this plan's
own first record.

## Deviations from Plan

**1. [Shape] The draft filing goes through a new `draft_message::the_copy_to_file`, not a
`who_it_goes_out_from` call in `wx_app.rs`.** The plan put the rule's call in `wx_app.rs` and a
case in the target calling `bytes_for` beside it; that case would have tested the test's own
composition, not the program's. The rule is called in `draft_message.rs` and the window calls
that function, so the target reaches the path the program runs. So the acceptance line "`grep -c
'who_it_goes_out_from' src/presentation/wx_app.rs` is at least 1" reads 0; the call is one hop
further in.

**2. [Premise] `src/application/protecting.rs` gained `None` in one test literal.** 13-21 added
`a_row_to_alice`, a `QueuedOutboxMessage` literal the plan's list predates; the plan said to
re-take the literals. `tests/the_feedback_dialog_shows_what_it_sends_before_it_goes.rs` needed no
edit: its two sites are text it reads, not literals. 17 literal sites in all, found by the
compiler.

**3. [Rule 1, refactor] Both draft reads share one row reader.** `load_drafts` and `load_draft`
each built a `CachedDraft` by position; adding two fields to each would have repeated the
mistake the plan warns of. They read by name through `a_draft`.

**4. [Ledger 660] The send-loop half is done; the check at Send moved to 13-35.**
`the_protection_check` reads what is held for the active account's address at Send, where no
row exists yet, and nothing in the composer chooses an address until 13-35. 13-35's plan gained
premise 5 naming ledger 660, and its premise 1 now names ledgers 713 and 717.

**5. [TDD] Each red names only the cases that fail on the stub.** Task 1's three companions and
task 2's two pass on stubs that do today's thing; the second red also names the count check,
which the first green's record made owe a re-measure.

**6. [Process] Three `--remeasure` calls where the brief asks for one per commit.** The second
found a record short, and CLAUDE.md asks for the record to be corrected by hand and measured
again.

**7. [Shape] The changelog went in the documents commit** as the plan's task 3 says; the green
commits changed nothing a person sees.

## Ledger

Opened, both halves: 717 (`todo`, nothing writes an address into a row until 13-35's composer,
named in its premise 1). Amended, both halves: 660, its send-loop half done and its check at
Send moved to 13-35, named in its premise 5; still open. Closed: none. Counts 717 in all, 644
open, 73 fixed, 0 waived.

## Threat Flags

None beyond the register. T-13-34-01: `NULL` reads as the account's own, held by the case over a
database made before the columns. T-13-34-02: sending and filing both call
`who_it_goes_out_from`, each with a guard record. T-13-34-SC: no crate added.

## Known Stubs

None that stand for work claimed here. The two production literals pass `None` for the From,
which is the plan's stated boundary; ledger 717 records it for 13-35.

## Self-Check: PASSED

The four code commits are on the branch; the created test file exists.

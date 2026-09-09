---
phase: 05-the-other-five-modules-keep-up
plan: 07
subsystem: data
tags: [tasks, move, provider, schema, transaction, deletions, guards]
status: complete
requirements-completed: []

requires:
  - phase: 05-the-other-five-modules-keep-up
    provides: "05-06's will_have_to_be_sent and the Waiting answer it feeds; 05-03's public tasks_sync::a_provider_holds; the refusals in moving_can_be_told and file_under, which this plan leaves standing"
provides:
  - "deleted_tasks.waiting_for_task_id: one additive nullable column naming the copy on this computer that must reach the provider before that deletion may be sent"
  - "message_cache::tasks::move_a_task_the_provider_holds: the new copy, the subtasks repointed, the note recorded and the old row removed, in one transaction with the destructive step last"
  - "MovedWhatTheProviderHolds: what a half-finished move answered, including the case where the task is no longer here"
  - "TaskSyncResult.waiting_on_the_new_copy: deletions held back because the copy replacing them has not gone yet, counted apart from those held by a setting"
  - "the deletion-loop question in push_tasks, asked last after the five skips already there"
  - "tests/a_half_finished_task_move.rs: eight integration tests over the storage half, including a database written before the column existed"
  - "two guard records, both measured by hand"
affects: [05-08]

actuals:
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A transaction made load-bearing by where its failure lands. Whether the task is still here is answered by the removal itself, through how many rows it took away, rather than by a read before the write. That answer arrives after the new copy and the note are already written, so a failure falls between the writes and two loose statements fail a test that a real transaction passes."
    - "A new state kept out of a shared type. The half-finished move is task-shaped, so it goes on DeletedTask and on deleted_tasks rather than as a third variant of TheDeletionSoFar, which contacts and events also read and neither of which can reach the state."
    - "Two counts of nothing happening kept apart, because only one of them names a remedy. A deletion held by Allow Changes is fixed by turning the setting on; a deletion held by an outstanding copy is fixed by waiting, and folding them would offer a remedy that does nothing for half the number."
    - "A migration test built by writing the old state directly, because the schema runs on every open and no sequence of ordinary opens can produce a database that predates a column."

key-files:
  created:
    - tests/a_half_finished_task_move.rs
  modified:
    - src/data/message_cache/mod.rs
    - src/data/message_cache/tasks.rs
    - src/application/tasks_sync.rs
    - guards/guards.toml
    - Cargo.toml
---

# Phase 05 Plan 07: A half-finished task move is a thing the database holds, Summary

**This summary was written by the orchestrator, not by the executor that did the
work.** The executor completed all three tasks, committed five times, wrote the
ledger entries, and then stopped responding before committing them or writing
this file. It was stopped after two and a half hours with no process running and
no write to the tree. Everything below is taken from the five commit messages,
the diff and the ledger entries it left; nothing is reconstructed from memory or
inferred about what it intended.

## Does it work

Yes, and **nothing in it is reachable by a person**, which is the point.

A task the provider holds can now be written into another list on this computer
as a new row carrying its whole content, with a note saying the provider's old
copy is owed a deletion. Both writes happen together or neither does. The note is
not sent while the new copy has not reached the provider, so the provider is
never asked to delete the only copy it has. The note survives the program
closing and is still owed when it opens again.

`moving_can_be_told` and `file_under` both still refuse a provider-held task and
both still say why. Neither was touched. `05-08` is what opens them.

Version `0.99.0` to `0.100.0`, for the schema change, which `CLAUDE.md` names as
bump-worthy whether or not anybody can see it. 681 guard records.

## What was built

**One additive column**, `deleted_tasks.waiting_for_task_id`, nullable, through
`ensure_column_exists`. Not a third variant on `TheDeletionSoFar`: contacts and
events read that type too and neither has a half-finished move, so a third
variant would put a state they cannot reach into every match over them. The
reason is written beside the new field, because folding the two into one enum is
the obvious tidying and it is the wrong one.

**`move_a_task_the_provider_holds`** writes the new copy, points the moved task's
subtasks at it, records what the provider is owed, and takes the old row away, in
one transaction with the destructive step last.

**One question in the deletion loop of `push_tasks`**, asked last, after the five
skips already there. Last on purpose: by then everything else about the deletion
is fine and the outstanding copy is the only thing holding it, so the count is
honest.

**A count of its own**, `waiting_on_the_new_copy`, folded by `absorb` and said as
"1 removal waiting for the new copy to be sent". Deliberately not folded into
`waiting_on_the_setting`, although both are counts of something that did not
happen, because turning Allow Changes on sends one of them and does nothing at
all for the other. A count rather than a sentence, because the sentence exists to
name a remedy and this one has none to name.

## The transaction is load-bearing, and the commit message says why

This is the finding worth keeping, and it is the same shape `05-04` found two
plans earlier.

Whether the task is really still here is answered **by the removal itself**,
through how many rows it took away, and not by a read before the write. The
caller hands over the task as it read it, and a sync deciding the provider no
longer holds it in between is ordinary. That answer arrives after the new copy
and the note are already written, so both have to be undone.

Two loose statements pass every other test in the file and fail that one. The
executor recorded this as measured rather than assumed.

## The migration test, which the plan was corrected for

The plan originally said to open a cache, write a note, close it and reopen. That
cannot see the migration: `initialize_schema` runs on every open, so a fresh temp
path gets the column from whichever mechanism is in the code, and the wrong
implementation the test exists to catch would pass. The plan was corrected on
2026-09-08 before this wave ran.

The executor built the older state through a raw `rusqlite` connection before
`MessageCache::new` ever saw the directory, **and then checked the test by hand
against the implementation it exists to catch**: folding the column into
`CREATE TABLE` and dropping the `ensure_column_exists` call reddens
`test_a_database_written_before_the_column_existed_opens_and_waits_for_nothing`
and nothing else in the file, because every other test is on a fresh path and
never meets the older state.

## Where the tests went, which the plan was also corrected for

The plan's artifact line asked one integration file to drive the whole sequence
against a fake `TaskService`. It cannot: `push_tasks` is private, `TaskService`
is `pub(crate)`, `Provider` is private, and the fake is inside the `#[cfg(test)]`
module. The corrected plan split it, and that is what shipped.

`tests/a_half_finished_task_move.rs` holds the eight storage tests. The
deletion-loop tests went into `tasks_sync.rs`'s own test module beside the fake,
taking that file from 106 test functions to 111. Nineteen records name it, and
every `#[test]` the task adds is in one commit so the count check's remedy is
paid once rather than twice.

## Red and green

Five commits, alternating as they should.

| commit | what |
|---|---|
| `55a3611` | RED, six of eight failing by assertion, not by build error |
| `a5e1235` | GREEN, the column and the transaction |
| `39ea5a7` | the guard on the move that did not happen, measured |
| `d8bab3b` | RED, three of six failing plus the count check |
| `7488bad` | GREEN, the deletion that waits |

**Tests that passed on arrival were not named in a trailer**, and the executor
gives the right reason: naming a test that passes buys an exemption with nothing
at stake. In the first RED, two drift guards passed and neither could be taken
red on that branch, because the red half has no column anywhere. In the second,
three of six passed, all asserting that a deletion **is** sent, which is what
today's code already does with nothing in the way. Their value is stated
precisely: an assertion that a call did not happen passes just as well when the
whole path is broken, so the same fixture one step further on has to show the
call really does happen once nothing holds it.

**The fake was widened to record what it was asked to delete.** It recorded
nothing before, so this is a first mechanism rather than a second one. It was
needed because the existing way of asserting nothing was sent, an empty error
list, is true whether or not the call was attempted: a delete refused by Allow
Changes is counted as waiting on the setting and never reaches the error list,
and whether that happens depends on the settings file of whoever runs the tests.

## Guard records

**The break that is not a break, found again.** Moving the statements onto
`self.conn` leaves everything green, because SQLite scopes a transaction to a
connection rather than to the handle a statement was issued through. That was
measured on 2026-09-08 for the identical break on `move_contact_between_groups`
in `05-04`, and it is now written into both records. The break that works is
committing before the early answer.

Measured with `WIXEN_TEST_THREADS=4 cargo test --all-targets --no-fail-fast` on a
clean tree, 47 targets. Exactly one test tree-wide went red, the same number the
contacts version came out at and the number this one predicted. The other seven
in the suite reach a route where every write succeeds, so a transaction and four
loose statements look the same to them.

**Two existing records came out short and neither was predictable.** "a count and
the thing it counts agree in number" and "the waiting count is said as a sentence
naming the setting" are both reddened by the new summary test, which names the
removal count and the sentence it must not be folded into. Corrected by hand with
the reason written into each red list, then re-measured: 18 tests and 3 tests,
all named, nothing else. Seventeen other records agreed on arrival.

680 records to 681, with the census on line 80 moved in the same commit.

## What is not checked, and the defect found on the way

Ledger 217 to 221. Four are unrun verifications: the two waiting counts have
never been heard side by side; how long a provider answers for a task under its
old identifier is a timing question no fake can answer; whether seven days is
long enough for a move made before a laptop is shut for a fortnight; and the
failure this whole state exists for has never happened, because no provider is
called by this plan at all.

**Ledger 220 is a defect this plan found and did not cause.** `rename_task`
orphans a task's subtasks. It calls `drop_synced_task` on the old identifier,
which sets `parent_task_id` to null for every child, so a task made on this
computer loses its subtask tree the moment a provider names it. That is older
than this plan and nothing in this plan reaches it.
`move_a_task_the_provider_holds` deliberately answers the same question the other
way, pointing the children at the new identifier, because the parent has not gone
but been renamed, and copying the null would have flattened a tree on every move.
**The two now disagree on purpose and one of them is wrong.**

## Known limitations

Nothing here has run against a real provider. Every test answers from a script.
The half-finished state is built by calling the write that produces it, which
proves the state is held, survives a close, and cannot exist half-written. It
proves nothing about whether a real pair of provider calls leaves exactly that
state.

`requirements-completed` is empty. PIM-01 stays Pending until `05-08`.

## Self-Check

- The five commit hashes resolve in `git log`.
- `guards/guards.toml` holds 681 records, and lines 79 and 80 sum to 681.
- `.planning/WINDOWS.md` holds 221 table rows and 221 JSON ids, both reaching 221,
  and 206 open plus 15 fixed equals 221.
- `src/application/tasks_sync.rs` holds 111 test functions;
  `tests/a_half_finished_task_move.rs` holds 8.
- `moving_can_be_told` and `file_under` still refuse a provider-held task on
  `main` and on this branch.
- No file this plan wrote holds a carriage return, measured with
  `tr -cd '\r' | wc -c` rather than with grep.

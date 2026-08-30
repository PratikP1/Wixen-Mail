---
phase: 01-folders-and-conversations
plan: 02
subsystem: message-cache
tags: [threading, rfc-5322, sqlite, migration, schema-additive, indexes]

requires: []
provides:
  - "application::thread_identity::conversation_root: the one rule that names a conversation, from one message, with no batch in its signature"
  - "messages.thread_id: a column that now has a writer, on every row, old and new"
  - "MessageCache::backfill_thread_ids: fills the rows written before there was a writer, idempotent by its WHERE clause"
  - "idx_messages_thread and idx_messages_message_id: the two lookups later plans need, neither servable by an existing index"
  - "The Thread column's sort orders by a real value instead of by NULL on every row"
affects: [01-11, 01-12, 01-13]

actuals:
  tokens: 9856
  tasks: 3
  commits: 3

tech-stack:
  added: []
  patterns:
    - "A derived column is computed at the one call site both entry paths already pass through, never at each named entry point"
    - "A row-filling migration is idempotent by its WHERE clause, which is the same clause that makes it unable to overwrite"
    - "An identity that will be stored takes no collection in its signature, so batch independence is structural rather than remembered"

key-files:
  created:
    - src/application/thread_identity.rs
  modified:
    - src/application/mod.rs
    - src/data/message_cache/messages.rs
    - src/data/message_cache/mod.rs
    - src/presentation/ui_types.rs
    - src/presentation/wx_app.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml

key-decisions:
  - "The id is written once, inside upsert_message, not at both named entry points: file_message_here is upsert_message plus one UPDATE, so a second call there would build the duplication the plan's own argument forbids"
  - "conversation_root returns a String and empties rather than an Option, so the decision not to store a row with no identifier stays with the caller"
  - "The backfill's break is guarded on its WHERE clause, because that clause is simultaneously what makes it idempotent and what makes it unable to overwrite"
  - "Thread View stays disabled: enabling it needs a conversation row to switch to, which is 01-12"
  - "threading.rs and message_columns.rs are untouched, deliberately"
  - "Version bumped 0.45.0 to 0.46.0, the first bump in 27 commits, because CLAUDE.md requires one for a behaviour change"

patterns-established:
  - "The three guard records here are the first in the file whose subject is a derived database column rather than a protocol verb"
  - "Guard record two breaks by adding code, which is the only way to break an invariant of the form 'exactly one place does this'"

requirements-completed: []
requirements-advanced: [THREAD-01, THREAD-02]

coverage:
  - id: D1
    description: "One conversation carries one id, in every folder, across restarts, and by whichever door the message reached the cache"
    requirement: THREAD-02
    verification:
      - kind: unit
        ref: "cargo test --lib thread_identity:: (10 tests, all new)"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/messages.rs#test_the_same_message_downloaded_and_filed_carries_one_conversation_id"
        status: pass
      - kind: other
        ref: "guards/guards.toml: a message stored by a sync and the same message filed after sending get one conversation id (measured, reddens exactly 1)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The id does not move when the surrounding batch does, which is what the rejected rule got wrong"
    requirement: THREAD-02
    verification:
      - kind: unit
        ref: "src/application/thread_identity.rs#the_answer_does_not_depend_on_which_other_messages_are_present"
        status: pass
      - kind: unit
        ref: "src/application/thread_identity.rs#an_arriving_message_with_a_smaller_identifier_renames_nothing"
        status: pass
      - kind: other
        ref: "grep -n 'pub fn conversation_root' shows two scalar parameters and no collection"
        status: pass
    human_judgment: false
  - id: D3
    description: "Every message already in the database has an id after this plan, and the backfill can only fill, never overwrite or drop"
    requirement: THREAD-02
    verification:
      - kind: unit
        ref: "src/data/message_cache/messages.rs#test_the_backfill_gives_every_older_message_a_conversation_id"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/messages.rs#test_the_backfill_never_writes_over_an_id_that_is_there_already"
        status: pass
      - kind: other
        ref: "guards/guards.toml: the conversation backfill fills only the messages that have none (measured, reddens exactly 2)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Sorting the message list by Thread groups a conversation instead of ordering every row by NULL"
    requirement: THREAD-01
    verification:
      - kind: integration
        ref: "src/data/message_cache/messages.rs#test_sorting_by_thread_groups_a_conversation, using the real Sort::order_by_clause"
        status: pass
      - kind: other
        ref: "guards/guards.toml: the column that says which conversation a message is in has a writer (measured, reddens exactly 5, including the sort test)"
        status: pass
    human_judgment: true
    rationale: >
      The query is proven to group. What is not proven is what a person hears when they
      press the Thread column header on a real mailbox, which needs a window, an account
      with stored credentials and a real server. Nothing in this program has ever run
      against a real mail account, and this plan does not change that.

duration: 1h 10m
completed: 2026-08-30
status: complete
---

# Phase 01 Plan 02: A conversation id that is written down

**Every message in the database, old and new, downloaded and sent, now carries a stable conversation id derived from the root of its `References` chain, so sorting by Thread groups a conversation instead of ordering every row by NULL.**

## Performance

- **Duration:** about 1 hour 10 minutes, including the human gate and the wait for its answer
- **Completed:** 2026-08-30
- **Tasks:** 3 of 3 (one of them the gate)
- **Files created:** 1. **Files modified:** 8

Roughly 20 minutes of that is measurement rather than work: one full library run
as a baseline and three more to measure the guard records by hand, at about
2 minutes 30 each. Every other test run in this plan was targeted, which is the
difference between 1 second and 188.

## The checkpoint answer, recorded

**Task 1 was `checkpoint:decision`, `gate="blocking-human"`.** It was reached,
stopped at, and returned rather than answered. Nothing had been written at that
point: the working tree was clean at `3cfe9d2`.

**Pratik answered: proceed, conditional on no existing threading being lost.**

The condition was checked before any work resumed, and it holds.
`src/application/threading.rs` is not in this plan's `files_modified` and was
never opened for editing: `thread_messages` is byte for byte what it was, so
`wx_app::apply_threading` still runs the in-memory pass over the open folder and
the conversation view behind Enter works exactly as it did. `git diff` over the
two commits touches no file under `src/application/` except the new
`thread_identity.rs` and the one line of `mod.rs` that declares it.

The one behaviour change acknowledged to him is in the changelog and has its own
section below.

## Accomplishments

- **The column has a writer.** Before this plan, `grep -rn "thread_id" src/data/`
  returned exactly one line, the `ensure_column_exists` that creates it. It now
  returns 44. That grep is the defect and its closure in one command.
- **One rule, one place, and the place is not the one the plan named.** The id is
  computed inside `upsert_message`, which is the only door: `file_message_here`
  opens with `let id = self.upsert_message(incoming)?;`. See Deviations.
- **Batch independence is structural.** `conversation_root(message_id, refs_header)`
  takes two scalars. It cannot see a batch, so it cannot be made to depend on
  one. The test that feeds three messages in three orders passes because the
  signature makes it impossible to fail, which is the point.
- **Older mail is filled in, once, and cannot be harmed.** `backfill_thread_ids`
  copies `migrate_inline_bodies` in all five of its properties, and its
  `WHERE thread_id IS NULL` is simultaneously what makes it idempotent and what
  makes it unable to write over a value it did not put there.
- **Three guard records, each measured by hand against the whole library.** One
  of them is the first record in the file that breaks by adding code rather than
  removing it, and it had to be, for a reason worth reading below.

## Task Commits

1. **Task 1: the human gate** — no commit; stopped and returned as required.
2. **Task 2: one function that names a conversation** — `5878dc5`
3. **Task 3: write it, backfill it, index it** — `a7b0c09`
4. **Task 4: this summary and the state** — see final commit

### On the RED and GREEN gates

`workflow.tdd_mode` is on and every behaviour here was written test-first. The
RED was watched fail each time, for the right reason:

| What | RED, measured | GREEN |
|---|---|---|
| `conversation_root` | `E0432: no conversation_root in application::thread_identity` | 10 tests pass |
| `thread_id` written, backfilled, sorted | `E0599: no method named backfill_thread_ids`, 4 sites, nothing else | 81 tests pass in the module |

The second RED is worth a note. All four compile errors were the same missing
method, which is how I know the rest of that eight-test block was sound before
the production code existed: the `Clone` on `IncomingMessage`, the private
`conn` field reached from a descendant module, and the `Sort` construction all
compiled.

The RED is not a separate commit and cannot be, for the reason 01-01 gives: the
pre-commit hook refuses a commit holding a failing test, and `--no-verify` is
forbidden by CLAUDE.md and was not used. The evidence that the tests were red
first is the table above, not the commit graph.

**One RED I did not expect, and it was mine.** The hostile-chain test asserted
that `<'; DROP TABLE messages;--@x>` came back whole. It comes back as `';`,
because a space separates identifiers and the chain holds three tokens, not one.
The code was right and my expected value was wrong. Fixed by asserting what
actually matters, that the value is stored and read back with nothing
interpreted, using an identifier with no spaces in it, and by adding a second
assertion for the space case so the next reader is not caught the same way.

## Files Created/Modified

- **`src/application/thread_identity.rs`** (new, 200 lines) — `conversation_root`,
  the private `bare`, and 10 tests. The module comment says what the failure was
  as well as what the code does, and says which of the two threading answers is
  authoritative for anything written down.
- `src/application/mod.rs` — one line declaring the module.
- `src/data/message_cache/messages.rs` — `thread_id` added to the INSERT column
  list and to the `ON CONFLICT DO UPDATE SET` list, its value computed once;
  `backfill_thread_ids`; 8 new tests and 3 test helpers.
- `src/data/message_cache/mod.rs` — the two indexes, and the backfill called
  non-fatally from `MessageCache::new` beside `migrate_inline_bodies`. No
  `ensure_column_exists` line was touched: `git diff` on this file has zero
  removed lines.
- `src/presentation/ui_types.rs` — the stale comment, reworded. **Not in the
  plan's `files_modified`.** See Deviations.
- `src/presentation/wx_app.rs` — the other stale comment, reworded. The menu
  item is untouched and still disabled.
- `guards/guards.toml` — three records, header count 311 to 314, 503 records to
  506.
- `docs/changelog.md` — one entry under `[Unreleased]`, with its known limits.
- `Cargo.toml` — 0.45.0 to 0.46.0. See Issues Encountered.

**`src/presentation/message_columns.rs` is in the plan's `files_modified` and was
deliberately not modified.** It has always returned `m.thread_id` for the Thread
column. The plan said to leave that string exactly as it is, because the backfill
is what makes it correct, and that is what happened. Its test lives in
`messages.rs` where the cache fixtures are, and it builds the clause from the
real `Sort::order_by_clause()` rather than a copy, because a copy is what goes
stale while the test keeps passing.

## The user-visible change

Sorting the message list by the Thread column. Today it orders every row by NULL
and appears to do nothing; after this it groups conversations. That is expected
and wanted, and it is in `docs/changelog.md` under `[Unreleased]`, in the
`Fixed` section, written for somebody who does not know what a thread id is. The
entry says plainly that mail already held is filled in once on the next open,
that nothing is deleted or moved, that Thread View in the View menu is a
different thing and still switched off, and that a sender writes their own reply
chain so a stranger can place a message in a conversation of yours, which every
mail program allows and which is only ever used for grouping.

No other behaviour changes. Nothing new is sent to a server. Nothing is deleted.

## Decisions Made

- **The id is written at one call site, not two.** The largest decision in the
  plan, and it went the other way from what the plan said. Full reasoning in
  Deviations.
- **`conversation_root` returns `String`, and empty for a message with neither a
  chain nor an identifier.** An `Option` that is `None` in a case that essentially
  never happens makes every caller handle it, and the real decision, whether to
  store such a row at all, belongs to the caller and is not this function's to
  take.
- **Identifiers with empty brackets are skipped rather than taken.** A chain of
  `<>` names no root, so the message falls back to its own identifier.
  `threading::continuing` already refuses to write `In-Reply-To: <>` for the same
  reason, and it is a real thing senders send.
- **`thread_id` goes in the `ON CONFLICT DO UPDATE SET` list, unlike the counts
  the list deliberately omits.** The omitted ones are facts about this computer
  rather than about the message. `thread_id` is derived from `message_id` and
  `refs_header`, both already on that list, so leaving it out would let a row
  carry a conversation its own stored chain contradicts.
- **Thread View stays disabled.** Enabling it needs a conversation row to switch
  to, which is 01-12. The comment beside it was corrected; the item was not
  touched.
- **The bracket trimming is spelled again rather than shared.**
  `threading::bracketed` is private and returns the wrapped form, which is the
  opposite of what a stored identifier needs, and `threading.rs` was not to be
  touched. The new `bare` says in its doc comment that it matches
  `bracketed`'s trimming and why it is not the same function.

## Deviations from Plan

Three, all approved by the coordinator before execution resumed, all found by
reading the code during the pre-gate premise check rather than by improvising
mid-task.

### 1. [Rule 1 - Bug in the plan] The id is written once, not "both, not one"

- **Found during:** the premise check before the gate
- **Issue:** Task 3 said to add the `conversation_root` call to
  `file_message_here` as well as `upsert_message`, insisting "Both, not one",
  and argued for it by quoting `threading::as_stored`'s doc comment about one
  rule serving both ways a message reaches the cache. The argument is right and
  the instruction contradicts it. `file_message_here` at `messages.rs:517` is
  three lines and the first is `let id = self.upsert_message(incoming)?;`. One
  call inside `upsert_message` already serves both doors, structurally and
  permanently. Writing it twice is exactly how the same message ends up threaded
  one way arriving and another when sent.
- **Fix:** One call, in `upsert_message`, with a comment at the call site saying
  why it is not repeated in the filing path. The plan's test is kept unchanged,
  one message stored both ways with the two ids compared, because that proves
  the property rather than assuming it.
- **Files modified:** `src/data/message_cache/messages.rs`
- **Committed in:** `a7b0c09`

**The break this cost, and what replaced it.** The plan specified guard record
one as "breaks the sent path by removing the `conversation_root` call from
`file_message_here`". That break cannot be applied, because the call should
never be written there. So the record measures the defect that actually exists,
which is the addition the plan itself asked for: a second copy of the rule in
the filing path, writing its own answer over the first. It is the only record in
`guards/guards.toml` whose `after` is longer than its `before`, and it had to be.
An invariant of the form "exactly one place computes this" is violated by
addition, so deleting code cannot prove it; deleting the one call site would only
prove the value is computed at all, which is a different and weaker claim that
record three already covers.

### 2. [Rule 1 - Bug in the plan] The stale comment is in a different file

- **Found during:** the premise check before the gate
- **Issue:** Task 3 said to read "the comment at `wx_app.rs:5021-5023`, which says
  threading is not implemented". Those lines are about F6 and the module-button
  menu items and have nothing to do with threading. The sentence "Threading is
  not computed yet, so `thread_id` stays `None`" is at
  `src/presentation/ui_types.rs:134-137`, a file the plan does not declare.
- **Fix:** Reworded rather than deleted, saying which half is now true: the
  database gains an id, and `MessageItem.thread_id` genuinely does stay `None`
  here, because `MessageListRow` does not select the column and will not until
  01-11 or 01-12. `ui_types.rs` added to the files touched, and named here rather
  than edited quietly.
- **Files modified:** `src/presentation/ui_types.rs`
- **Committed in:** `a7b0c09`

A second stale sentence was found by grepping the tree rather than trusting the
line number: `wx_app.rs:5059` said "Threading is not implemented" beside the
disabled Thread View item. That one was already false before this plan, since
conversations have been reachable with Enter for some time and the changelog says
so. It now says what is built and what is not.

### 3. [Rule 3 - Blocking] `threading::bracketed` adds brackets, it does not strip them

- **Found during:** the premise check before the gate
- **Issue:** The plan said to trim brackets "the way `threading::bracketed`
  handles them". `bracketed` trims and then puts one pair back on
  (`format!("<{bare}>")`), which is the opposite of what a stored identifier
  needs. It is also private.
- **Fix:** A private `bare` in the new module, with a doc comment saying it
  matches `bracketed`'s trimming, why it is not that function, and that the
  module `bracketed` lives in was deliberately not touched.
- **Files modified:** `src/application/thread_identity.rs`
- **Committed in:** `5878dc5`

---

**Total deviations:** 3, all corrections to wrong premises in the plan, all
approved before execution. None needed Rule 4: nothing here changed the
architecture, and the one instruction that would have (a second writer) was
narrowed rather than obeyed.

## Issues Encountered

**The version bump is late by 27 commits, and the check that should have caught
that cannot.** CLAUDE.md says a behaviour change bumps the version in the same
commit. `Cargo.toml` last moved at `a48ab95`, 27 commits ago, and 01-01's whole
create-a-folder feature went in without one. I bumped 0.45.0 to 0.46.0 here
because the rule is explicit and this plan changes what a user sees and writes to
every row of their database. Two things worth someone's attention rather than
mine:

- The bump is not precise. It covers 27 commits of work, not this plan's.
- `test_no_status_page_names_a_version_the_code_does_not_ship` in
  `tests/house_style.rs` is the check nearest this rule, and it is currently
  vacuous. It compares versions named in `README.md` and
  `docs/IMPLEMENTATION_STATUS.md` against the shipped one; neither file names a
  version at all, so it iterates over nothing and passes unconditionally. The
  test's own comment recommends exactly that ("a page that wants to stay out of
  the way should point at the changelog instead of naming a number"), so the
  check was disarmed by somebody taking its advice. That is not the same as a
  stale check, and it is why the drift went 27 commits without a red run.

**Three guard records rather than the plan's two.** The plan asked for two and
said to raise the header count by two. I measured three and raised it by three,
because the break the plan named for record one does not exist (Deviation 1) and
splitting it honestly needs both a record for "the column has a writer at all"
and one for "only one place writes it". `cargo test --test house_style` passes,
which is what checks the header arithmetic, and `bash scripts/guards.sh
conversation` reports all three reddening exactly the tests their records name.

**What each break really reddened, versus what I would have guessed.** Record
three's break, opening the backfill's `WHERE` clause to every row, reddens the
idempotence test and the never-overwrites test. It does **not** redden the test
that the backfill fills three empty rows, because a backfill that fills
everything still fills those. Guessing would have named all three. This is
written into the record itself.

**No existing guard was weakened.** The 01-01 lesson was that adding a ninth
gated write to a census with a floor of eight silently took a reddening test away
from a neighbouring record. Nothing this plan adds is counted by a census: the
two indexes are not floored anywhere, and `thread_id` joins an INSERT column list
with no arity assertion over it. Checked by running the whole library before any
break was applied (5,310 pass, 0 fail) and `scripts/guards.sh conversation`
after.

**Nothing has run against a real mail account.** Unchanged by this plan, and no
criterion here claims otherwise. The `References` chains in every test are ones
the tests wrote.

## Known Stubs

None. Every artefact this plan names is reached by a non-test path:
`conversation_root` from `upsert_message` and from `backfill_thread_ids`, the
backfill from `MessageCache::new`, and both indexes from `initialize_schema`.

The thing that could be mistaken for a stub is deliberate and is named in the
plan: `MessageItem.thread_id` is still `None`, because `MessageListRow` does not
select the column. That is 01-11's and 01-12's work, the comment at
`ui_types.rs` now says so precisely, and no user-facing claim depends on it. The
Thread column's **sort** does not go through `MessageItem` at all; it is a SQL
`ORDER BY` on the stored column, which is why it works today.

## Threat Flags

None. This plan opens no network path, adds no endpoint, and changes no trust
boundary. The three registered threats it was written against are all covered:
T-01-05 by the hostile-chain tests at both layers, T-01-06 by the
hundred-thousand-identifier test, T-01-07 by the `WHERE` clause and its two
tests and the guard record over it, and T-01-08 by the backfill logging a count
and never an identifier.

One thing to say plainly rather than flag: a sender writes their own `References`
header, so anyone who can send you mail can name any root and place their message
in a conversation of yours. This is inherent to RFC 5322 threading and every mail
client has it. What bounds the damage is that the id is only ever a grouping key,
never a permission, a path or a filename, and it reaches SQL as a bound
parameter. That is stated in the changelog too, where somebody using the program
can read it.

## Next Phase Readiness

Ready. The four plans that were blocked on this are unblocked.

- **01-11** (count a conversation across an account) has `idx_messages_thread`
  for its `GROUP BY` and a column with a value in every row.
- **01-12** (collapse the list to one row per conversation) has the same, plus
  the Thread View menu item still disabled and now correctly commented. It will
  need `MessageListRow` to select `thread_id`, which is the one line that makes
  `ui_types.rs`'s remaining `None` go away.
- **01-13** (rethread as mail arrives) has `idx_messages_message_id` for the
  arrival lookup, and, more importantly, has the Pitfall 6 problem removed
  rather than mitigated: an incremental join cannot disagree with a batch
  recompute here, because there is no batch recompute of the stored id and
  nothing to adopt. Each message answers for itself.
- **Nothing in the phase read a thread id before this plan landed.** Checked:
  `thread_id` in `src/data/` was one line before, and no plan numbered below 02
  names it.

---
*Phase: 01-folders-and-conversations*
*Completed: 2026-08-30*

## Self-Check: PASSED

Every file, commit hash, symbol and number this summary names was checked
against disk and `git log` after it was written.

- All 9 files present; `src/application/thread_identity.rs` is 200 lines.
- Both commits resolve: `5878dc5`, `a7b0c09`.
- `conversation_root`, `backfill_thread_ids` and `bare` all present.
- `guards/guards.toml` holds 506 records and its header says 314 have arrived
  since the sweep, which is the arithmetic `tests/house_style.rs` checks.
- `src/presentation/message_columns.rs` and `src/application/threading.rs` each
  appear zero times in `git diff HEAD~2 HEAD --name-only`, which is the check on
  the condition Pratik attached to his answer.
- The grep in the objective: `thread_id` in `src/data/` was 1 occurrence at the
  branch base `3cfe9d2` and is 44 now.

Green when the work was committed: `bash scripts/check.sh` on every commit
through the pre-commit hook, which on this branch is rustfmt and clippy with
`-D warnings`. `--no-verify` was never used. The whole library was run four
times by hand during this plan: once clean at 5,310 passing, and three times
with a guard break applied, at 5,305, 5,309 and 5,308. `cargo test --test
house_style` passes at 52. `bash scripts/guards.sh conversation` reports all
three new records reddening exactly the tests they name.

The two slow checks, the full `--all-targets` suite and the release build, have
not run on this branch by design; they run once at the merge.

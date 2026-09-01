---
phase: 02-search-that-says-what-it-covers
plan: 02
subsystem: search
tags: [saved-searches, coverage, eviction, disclosure, guards]
status: complete
requires:
  - "scan_query and messages_a_saved_search_reads, the read a saved search is run over"
  - "SavedSearch::reads_the_message_text as the one place that knows whether bodies are needed"
  - "UIUpdate::StatusUpdated, which is shown in the status bar and announced"
provides:
  - "MessageCache::how_much_message_text_is_stored_here, one account-scoped pass giving both counts"
  - "TextStoredHere, the two counts as one value so they cannot be swapped or read a moment apart"
  - "application::saved_searches::what_a_saved_search_covers, the sentence, naming which search it is about"
  - "wx_app::coverage_before, asked only when the search reads message text"
  - "evict_bodies_over's deliberate-non-action doc, with a behavioural test holding it"
  - "two guard records, both measured against the whole library on this tree"
affects:
  - "every saved search that asks about message text: one extra count before the gather"
  - "the status topic, which now carries one more low-priority announcement per such search"
tech-stack:
  added: []
  patterns:
    - "a counting query built from an existing query's WHERE clause, with the agreement named in the doc"
    - "two counts returned as one struct rather than two adjacent integers"
    - "a source-reading reach test cut by common::what_ships, asserting order as well as presence"
    - "a behavioural test for a deliberate non-action, with the distinguishing word placed past the snippet"
key-files:
  created: []
  modified:
    - src/data/message_cache/saved_searches.rs
    - src/application/saved_searches.rs
    - src/presentation/wx_app.rs
    - src/data/message_cache/bodies.rs
    - guards/guards.toml
    - docs/changelog.md
decisions:
  - "The sentence builder takes TextStoredHere rather than two integers, because two adjacent counts of the same type are a swap the compiler cannot see."
  - "An account with no mail here at all gets a different sentence and no coverage figure, which the plan's behaviour list did not cover."
  - "A failed count says nothing and lets the search run, rather than refusing to search because the disclosure could not be worked out."
  - "Two guard records rather than the one the plan asked for: the reindex break was measured against the whole library anyway, so recording it cost nothing and it is the regression D-2-13 warns about."
  - "No index added. The count is 65 to 119 ms at 200,001 messages in a debug build, on the worker that already does the gather."
metrics:
  duration: one session
  completed: 2026-09-01
actuals:
  tokens: 8200
  tasks: 3
  commits: 9
---

# Phase 2 Plan 2: Say what a saved search covers Summary

Before a saved search that reads message text runs, the person running it is
told how many messages are in the account and how many of those have their text
on this computer, and the sentence says it is about that saved search rather
than about the search box, because the two genuinely cover different amounts of
the same mailbox.

## What works, and how that was checked

Working and exercised end to end: the count, the sentence, the wiring, and the
eviction comment with a test behind it.

**The sentence reaches a person on a non-test path.** This is the criterion the
plan says the whole thing exists for, so it was traced rather than assumed:

1. `run_a_saved_search` has two live callers, both in the window's event
   wiring: opening a saved-search row in the folder tree
   (`wx_app.rs:2396`) and the Refresh command while a saved search is open
   (`wx_app.rs:3348`). Neither is a test.
2. Inside it, `coverage_before` is asked on the worker that already does the
   gather, and its answer is sent as `UIUpdate::StatusUpdated` before
   `messages_a_saved_search_reads` is called.
3. `handle_update`'s `StatusUpdated` arm (`wx_app.rs:14464`) writes the text to
   the status bar **and** calls `a11y.announce_topic(status, Priority::Low,
   "status")`, so it is spoken and brailled rather than only painted at the
   bottom of a window.

A test holds each link that a test can hold. The reach test reads the shipping
half of `wx_app.rs`, cut by `common::what_ships`, and requires the coverage call
and the status update to appear in `run_a_saved_search` before the gather.

**Not proved, and it needs a screen reader.** Nobody has heard this. Two things
to listen for. First, that the sentence is spoken at all before the results
arrive. Second, that it does not swallow the line before it: the progress line
"Running this saved search..." and the coverage sentence share the topic
"status", the announcement queue coalesces same-topic announcements, and the
coverage sentence is the later of the two, so the progress line is expected to
be dropped. That is the queue working as designed and it is probably the right
outcome, since the coverage sentence also says a saved search is what is
running. It is a guess until somebody hears it, and it is recorded in
`.planning/WINDOWS.md` as entry 10.

## Commits

| Task | Commit | What |
|------|--------|------|
| 1 | `0340cf7` | RED: five failing tests for the coverage count |
| 1 | `285601c` | GREEN: the account-scoped counting query |
| 2 | `a5fad2a` | RED: five failing tests for the sentence |
| 2 | `c06c0b5` | GREEN: the sentence, naming which search it is about |
| 3 | `0c16a4c` | RED: the reach test, against the unwired tree |
| 3 | `3e8e652` | RED: two failing tests for the helper, plus its stub and wiring |
| 3 | `ea99c95` | GREEN: the helper, and the changelog entry |
| 3 | `5281da9` | Eviction's deliberate non-action, written down and tested |
| 3 | `4412703` | Two guard records, and the reach test strengthened by one of them |

## The red half, committed rather than described

Four RED commits, each carrying a `Fails-until-green` trailer and each held to
it by `scripts/red-commit.sh`: every named test ran, every named test failed,
nothing else failed. This is the first plan in this project able to do that, and
it is worth saying what it changed in practice. 02-01 recorded red as a
measurement in prose, which was good evidence nobody could re-run. These nine
commits carry the evidence in the graph.

Every red was taken from a value, never from a missing symbol. The stubs
returned a fixed pair of numbers, an empty string, and an empty sentence for
every search.

**Two tests were green against their stub and are named rather than left to look
like test-after.**

| Test | Why it could not be red | Break that reddens it |
|------|------------------------|----------------------|
| `test_one_place_builds_the_coverage_sentence` | It counts a constant in the source, and the constant has to exist before the stub compiles | a second copy of the words in this file (count 2), and the same words in `wx_app.rs` (count 1 where 0 is required); both taken, both reddened only this test |
| `test_an_evicted_message_stays_findable_by_a_word_from_its_text` | It describes behaviour the tree already has, so it is a characterisation test | adding `index_message_for_search` to the eviction loop; reddens this test and nothing else in 5,852 |

**One trap in the red half that is worth passing on.** The build denies
warnings, so a stub that leaves a new constant unread fails `dead_code` and the
commit is refused before any test runs. The refusal names a lint and says
nothing about the real cause. The stubs here bind their unused arguments and
constants on purpose, and the real body replaces the binding in the green
commit.

## The by-hand break that found a hole in its own test

The reach test as first written looked for `coverage_before` inside
`run_a_saved_search`. Taking the guard break by hand, which is keeping the call
and replacing the two lines that say the answer with `let _ = ...`, left it
**green**. That is a number computed and never said, which is the exact defect
the plan's `key_links` names, sailing past the test written to notice it.

The test now requires `UIUpdate::StatusUpdated` to appear between the coverage
call and the gather. The break reddens it. This was found by taking the break,
not by reading the test, which is the argument for taking breaks by hand rather
than trusting a green suite.

## What was measured, not assumed

**D-2-13's premise, tested rather than inherited.** The decision says an evicted
message stays findable by quick search on words the cache no longer stores. That
is now a test and it passes: a word placed past the snippet, indexed when the
body was stored, is still found by `search_messages` after `evict_bodies_over`
has deleted the row, while `messages_a_saved_search_reads` comes back with no
text for that message. The two searches really do disagree.

**The trap in that test, which nearly made it vacuous.** A snippet is the first
200 characters of the body, it is written into `messages.snippet` when the body
is stored, it is a column of the search index, and it is never evicted. A test
using a word from the first 200 characters passes whether or not eviction
reindexes, because the snippet keeps it either way. The word is placed past the
snippet for that reason, and the comment says so.

**The counting query's cost.** Measured with a temporary test at 200,001
messages, 100,000 of them with stored bodies, in a debug build on this machine:
**119 ms cold and 65 ms warm.** It runs on the `spawn_blocking` worker that
already does the gather, and it reads no body bytes, so it is strictly cheaper
than the search it discloses. No index added, following the house rule in that
file that an index comment carries a measured before and after. The timing test
was removed after the measurement.

**Both breaks the plan asks to be demonstrated by hand.** Widening the account
condition reddens only
`test_one_accounts_coverage_is_not_inflated_by_another_accounts_mail`. Dropping
`m.deleted = 0` reddens only
`test_mail_marked_deleted_is_counted_by_neither_number`. Making the
`reads_the_message_text` branch unconditional reddens only
`test_a_search_that_does_not_read_message_text_is_told_nothing`.

**Guard records: two, both machine-measured on this tree.** The brief says a
record written during a plan may be written by hand and marked unverified. These
are not: each break was applied and `cargo test --lib` run over the whole
library, giving 5,851 passed and 1 failed both times, with the failure being the
single test the record names. That is the same measurement `scripts/guards.sh`
takes for one record. `scripts/guards.sh` itself was **not** run, per the brief.
The sweep count at the top of `guards/guards.toml` goes from 344 to 346 in the
same edit, and `grep -c '^\[\[guard\]\]'` gives 538, which is 192 + 346.

**D-2-10 was not implemented, and the reason.** It asked whether eviction should
reindex. D-2-13 answers no, on the same day and in the opposite direction from
the way the question was framed. Reindexing would make a message unfindable at
the moment its text is evicted rather than merely unsearchable by its text,
which takes away a search that works today. `evict_bodies_over` now says that
where a reader would otherwise file a bug, and a guard record holds it.

## Deviations from Plan

### 1. [Rule 2 - correctness] The sentence builder takes one value, not two integers

**Plan said:** "Add one function beside `a_typed_search_in_words` taking the two
numbers."
**Done instead:** it takes `TextStoredHere`, the struct task 1 already returns.
**Why:** two adjacent `i64` parameters counting related things is a swap the
compiler cannot see, and this sentence is a claim about honesty, so getting it
backwards would overstate coverage in the confident direction. The layering cost
is nil: `application::saved_searches` already imports `CachedMessage` from
`data::message_cache` and runs searches over it.
**Commit:** `c06c0b5`.

### 2. [Rule 2 - missing case] An account with no mail here gets its own sentence

**Found during:** Task 2, working through the behaviour list.
**Issue:** the plan's four behaviours cover all-stored, none-stored and mixed,
and not the empty account. It is reachable: a saved search can be run against an
account whose mail has not synced. The all-stored branch would have caught it
and said "this computer has the text of the 0 messages in this account", which
is true, useless, and reads as a fault in the search.
**Done instead:** a fourth branch saying there is no mail from this account here
yet, with a test asserting no coverage figure appears in it.
**Commit:** `c06c0b5`.

### 3. [Rule 2 - correctness] A second guard record

**Plan said:** "Record one guard: that the coverage sentence is reached."
**Done instead:** two. The second holds eviction to not reindexing.
**Why:** the reindex break had to be measured against the whole library anyway,
to write the eviction test honestly, so the record cost one extra run and
nothing else. It is also the exact regression D-2-13 warns about, and the thing
that makes a comment perish is nobody measuring it again.
**Commit:** `4412703`.

### 4. [Rule 1 - bug] The reach test did not see the defect it was written for

Covered above under "the by-hand break that found a hole in its own test".
**Commit:** `4412703`.

### 5. The plan's `<verify>` block asks for `bash scripts/guards.sh`

Not run, on the brief's explicit instruction that guard re-measurement is off
the critical path. The two records this plan adds were measured individually
instead, which is stronger for those two and says nothing about the other 536.

## Wrong premises found

Reported rather than built on, as asked.

1. **The `total_count` acceptance criterion cannot be satisfied and does not
   need to be.** It requires `grep -n 'total_count'
   src/data/message_cache/saved_searches.rs` to return nothing. It returns one
   line, and did before this plan: `total_count: 0` in the test helper
   `a_folder_holding`, initialising a `CachedFolder`. That is a struct field in
   a fixture, not a use of `folders.total_count` in a query. The property the
   criterion stands for holds, and the criterion could only be satisfied by
   deleting unrelated pre-existing test code. Same shape as 02-01's finding
   about `grep -c 'serde(default'`: a text search standing in for a construct,
   answered by a mention rather than a use.

2. **`evict_bodies_over` returns bytes freed, not a count of rows.** The plan
   does not say otherwise, but it is worth recording because the first version
   of the eviction test asserted `evict_bodies_over(0) == 1` and failed with
   `left: 58`. Read as a premise failure for a moment, it is not one, and
   telling the two apart quickly matters when the test is the thing checking a
   premise.

3. **Everything else in the plan and in D-2-08, D-2-09 and D-2-13 held.**
   D-2-09's correction is right and was re-checked from the predicate rather
   than the payload: the FTS declaration at `mod.rs:2170` carries a `body`
   column, `index_message_for_search` fills it from the stored body, and
   `run_a_saved_search` chooses `TheMessageText::Read`, so the disclosure is a
   live defect being fixed rather than one this phase introduces. D-2-13's
   claim about the two searches disagreeing is now a passing test rather than a
   reading of the source.

## Known stubs

None. Nothing here is a placeholder, and nothing was left computed and never
said, which was the specific risk.

## Threat flags

Nothing new. `T-02-07` and `T-02-08` are mitigated as the register describes:
the query joins `folders` on `account_id` with a test that reddens when that
narrowing is removed, and the sentence names its subject rather than implying
one number covers both searches. `T-02-10` stays not-reachable: the account id
is bound as a parameter and nothing is interpolated. No dependency added,
`Cargo.toml` unchanged.

## Verification

- `cargo test --all-targets --no-fail-fast`: 5,852 library tests and every
  integration target green, 0 failed, 1 ignored.
- `bash scripts/check.sh` green on all nine commits, four of them through the
  `red` path, which additionally proved the named failures were exactly the
  failures.
- `git diff main -- src/data/message_cache/searching.rs` is empty and the four
  `index_message_for_search` call sites are unchanged, so the search index is
  untouched by this plan.
- `grep -c 'self\.index_message_for_search(' src/data/message_cache/bodies.rs`
  gives 1, the existing call after a body is stored, with the new doc naming the
  function without tripping it.
- `grep -c '^\[\[guard\]\]' guards/guards.toml` gives 538 and the sweep numbers
  add to 538; `cargo test --test house_style` green, which is what checks that.
- No `--no-verify`, no `#[allow(...)]`, no new dependency, no AI attribution, no
  em-dash.

## Self-Check: PASSED

Every file in `key-files.modified` exists and differs from `main`. All nine
commit hashes resolve on `gsd/plan-02-02`: `0340cf7`, `285601c`, `a5fad2a`,
`c06c0b5`, `0c16a4c`, `3e8e652`, `ea99c95`, `5281da9`, `4412703`.

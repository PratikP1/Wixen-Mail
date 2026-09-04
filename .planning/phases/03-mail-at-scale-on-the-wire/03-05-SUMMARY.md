---
phase: 03-mail-at-scale-on-the-wire
plan: 05
executed: 2026-09-04
status: complete
tasks: 3
requirements: [DEFER-1-02]
subsystem: data, application
tags: [threading, conversation-merge, schema, backfill, guard-record, defer-1-02]
commits:
  - 886f3ee test(03-05) failing tests for a root that arrives after the message naming it
  - 43332f6 test(03-05) failing tests for the conversations already stored
  - a307922 A conversation root that arrives late is found by what named it
  - 385de22 test(03-05) failing tests for a conversation that keeps its name
  - e10d998 A merged conversation keeps one name, whatever order its mail arrived in
merged: not merged, and not pushed
key-files:
  created: []
  modified:
    - src/data/message_cache/messages.rs
    - src/data/message_cache/mod.rs
    - src/application/thread_identity.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
    - Cargo.lock
requires: []
provides:
  - "identifiers_a_message_names: one row per identifier a stored message names, its own and each of its chain, with an index on the identifier"
  - "a conversation lookup that asks what a message named as well as what it is called, so a root arriving late is found"
  - "a backfill that records what every stored message names and applies the merges it reveals, once, on open"
  - "a merge winner that is a function of the set being merged rather than of which message arrived"
affects:
  - "03-06 and anything else touching upsert_message: the arrival path now writes a second table and asks a second question, at about nine microseconds more per message"
  - "anything that keys state on a conversation id: the id is now stable across arrival orders, where it was not"
decisions:
  - "The table holds no conversation. An identifier-to-conversation map is one indexed probe rather than a join and is what the deferred note asked for; it is also a second copy of a fact reroot_threads rewrites in the first, which is the failure thread_identity's module comment opens with. A row says what a message named, and which conversation that is stays in messages.thread_id."
  - "The backfill is gated on the table holding nothing, which is derived state rather than a marker, and the whole of it is one transaction so a failure rolls back to empty and the next open starts again. A marker wrong in the direction of 'already done' would leave conversations split with nothing left to join them."
  - "The winner is the least identifier among the conversations being merged. It is arbitrary as archaeology and it is the only total order available, and stability is what was actually needed: the same set of messages ends under the same name whatever the arrival order."
  - "The record for Pitfall 6 was repointed rather than deleted. Its break stopped discriminating the moment the winner became a function of the set, which was measured rather than assumed, and the property that replaced it is the one it now carries."
metrics:
  duration: about 4 hours
  files: 7
  commits: 5
actuals:
  tokens: 21000
  tasks: 3
  commits: 5
---

# Plan 03-05: A conversation root that arrives late

**One-liner:** A conversation root arriving after a message that already names
it now joins that message's conversation, mail already stored is joined when the
program next opens, and a merged conversation settles on the same name whatever
order its messages arrived in.

## What works

### The six arrival orders

Two conversation roots, `a@x` and `c@x`, and one message `x@x` whose chain says
they are one. Three of the six orders in which those can arrive ended as one
conversation before this plan and three left it split. All six end as one
conversation now, under one name.

The half that was missing was a question nobody could ask. A merge asked
`threads_holding_any_of` which conversations hold a message *called* one of
these, so when the connector was stored first, the roots it named arrived later
with nothing that could reach it: the proof was inside `x@x`'s stored
`refs_header`, where no index can look. `identifiers_a_message_names` holds one
row per identifier a message names, its own and each of its chain, and the
lookup asks it as a second question beside the message column.

Both spellings still go to the message column, because a database the identifier
backfill never finished still holds the bracketed one, and narrowing that would
be mail dropping out of a conversation. The new table is asked in one spelling,
because every writer of it goes through `thread_identity::bare` and the other
form cannot be in it. The parameter arithmetic in the comment is corrected: 128
for the column, up to 64 for the table, and the account once in each half, so
194 against SQLite's 999 where it used to say 129.

### The table holds no conversation

The deferred note and the roadmap both ask for "a table mapping every identifier
a message names to the conversation that message is in". That is one indexed
probe rather than a three-table join, and it is faster. It is also a second copy
of which conversation a message is in, which `reroot_threads` rewrites in the
first, and two places able to come to differ is the failure
`thread_identity`'s module comment opens with in its first paragraph.

So a row says only what a message named. Which conversation that is stays in
`messages.thread_id` and is reached through the join at the moment the question
is asked. That also dissolves the ordering question the plan asked to be settled
in a sentence: there is nothing written down for a rename to make stale, so
recording before the merge or after it comes to the same thing. It is done
before, because what a message names is a fact about its headers rather than
about any conversation, and a merge that fails should not take it down too.

### The conversations already stored

A table that starts empty reaches only mail that has not arrived yet. The
backfill records what every stored message names and then applies the merges
that reveals, and it is what somebody who has been using this program actually
sees.

It is gated on the table holding nothing, which is one probe, and that is
deliberately not a marker: a marker is a second thing that can be wrong, and one
wrong in the direction of "already done" leaves conversations split with nothing
left that would ever join them. The gate is the state itself. The whole of it is
one transaction, which is what keeps the gate honest, because a run that fails
half way rolls back to an empty table and the next open sees the same "not done
yet" and starts again.

Two passes rather than one: every message's names are recorded first and the
merges are asked afterwards, because a merge asked half way through the
recording can only see the part recorded so far, which is the arrival-order
dependence this plan exists to remove.

### The name a merged conversation keeps

`rejoin` made the arriving message's own conversation the winner, and its
comment argued that a chain is oldest first so that conversation is the chain's
earliest identifier. True of a message carrying a full `References` chain, which
is why it went unquestioned. Not true of one naming only its parent, which is
what a client sending `In-Reply-To` and nothing else produces: there the
arriving conversation is a message somewhere in the middle. So the same three
messages settled under `p@x` or under `r@x` depending on which arrived last.

The winner is the least identifier among the conversations being merged now,
counting the arriving message's own. The comment says what that is not: it is
not archaeology, it does not find the older message, and the conversation can
settle under an identifier that is nobody's root. Nothing available to `rejoin`
could find the older one, because two conversations an arrival has just proved
to be one carry no ordering between their names. What it buys is the only thing
left that matters, which is that the answer depends on the set and not on the
order.

D-39 survives and has its own test: an arrival naming the conversation it is
already in renames nothing, whatever its own identifier sorts like. That is what
separates this from the rejected batch rule, which re-derived a name from
whatever happened to be in hand.

## What it costs, measured

Release build, warm, on this computer. Two hundred thousand messages in ten
thousand conversations, each message naming its root, its parent and itself,
which comes to five hundred and seventy thousand recorded names.

| | the first open | every open after |
| --- | --- | --- |
| nothing left split | 5.66 s | 73 us |
| every conversation split in two | 6.45 s | 69 us |

The second row is ten thousand merges moving a hundred thousand messages, so a
mailbox with more to join pays a little more and not a different order of it.
The second column is the gate.

The lookup an arriving message pays for:

| | per arriving message |
| --- | --- |
| the message column only, as it was | 9.6 us and 10.3 us |
| both questions, as it is | 19.0 us and 18.9 us |

Two figures each because the pair was taken twice, on the two databases above.
So an arrival costs about twice what it did and about nine microseconds more,
which over a sync of five thousand messages is under fifty milliseconds.

Measured with a temporary `#[ignore]` test that is not in the tree: it builds
two hundred thousand rows with raw inserts and asserts nothing.

## Verification

Every commit went through `scripts/check.sh` by way of the commit hook. Nothing
used `--no-verify`. Two commits changed `Cargo.toml`, so `which-checks.sh`
answered `all` for both and each ran the whole suite and the release build.

**Three reds, all accepted by `scripts/red-commit.sh`.** The first names five
tests and the count check; four of the five fail on a table that does not exist
yet, which is a runtime failure rather than a compile error, so each fails on
the thing it is about. The second names nine and the count check. The third
names three.

**The library is 6124 tests, 0 failed, 1 ignored.**

**Three guard records, each measured by hand before it was written, and all
three re-measured against the finished tree.**

| record | what really went red |
| --- | --- |
| the conversation lookup asks what a message named, not only what it is called | 1 test |
| the conversations stored apart are joined when the database is opened | 3 tests |
| a merged conversation settles on the same name whatever order its messages arrived in | 3 tests |

**The lookup's break reddens one test and not the three about the backfill, and
that is a finding rather than a count.** The backfill walks every stored message
and asks about each one in turn, so the connector's own chain names both roots
and the old question is enough there. The table is what an *arrival* needs,
because an arrival asks about one message and nothing ever re-asks the ones
already stored. Neither record stands in for the other.

**Twenty-one records were re-measured for the changed test count.** Nineteen
agreed. The two that did not are worth reading.

The backfill-selects record gained a test that is not about it: two backfills
running one after the other on the same open share a column, so one that fills
every row rather than the empty ones rewrites, on the second open, what the
other just settled. Nobody writing that test would have filtered for that
record.

The writer record gained one test and reported another of its own as having gone
green, and the second direction is the finding. Rewriting
`test_the_same_message_arriving_twice_moves_nothing_the_second_time` to go
through the storing path left its first assertion counting distinct
conversations, and that count is one both when every row holds the same
conversation and when every row holds nothing. A test weakened by an edit nobody
would have called risky, caught by that record and by nothing else. It names the
conversation it expects now.

**One guard was disarmed by task 3, and that was measured rather than assumed.**
"An arriving message and the conversation rule agree, because one of them is not
consulted" broke the merge by adopting the last conversation the lookup found.
Under a least-identifier winner that is harmless: the winner is a function of
the set, the arriving message's own conversation is always in the set because
its row is written before the lookup is asked, and adopting a member of a set
does not change the set. Run against the finished tree the old break left all
three of its named tests green. The defect it guarded is not unguarded, it is
impossible, and the record was repointed at the property that replaced it rather
than deleted, so the file keeps the reason.

## Premises that were wrong

### 1. The plan's failing case describes a set the defect cannot occur in

The plan says "all six arrival orders over a set of a root and two messages
naming it", and says three of six fail. Read literally that is one root and two
replies that both name it. Every one of those six orders already worked, because
each reply derives the same conversation from the root it names and there is
nothing left to reconcile. A fixture built from that sentence would have passed
in all six orders, and nothing about it would have looked wrong.

The set the defect needs is two independent roots and one later message naming
both. That was found by walking the six orders by hand against the code before
writing anything, and the plan's own count of three failures is what confirmed
the fixture rather than the prose.

### 2. "One table, one index, one writer" also leaves out the winner rule, and
the plan half knew it

The plan's own premise corrections name the backfill and the winner rule, so
this is a correction to the roadmap's bullet rather than to the plan. Both
corrections were checked here rather than taken on trust, and both hold. The
winner rule is the larger of the two: it is not an addition to the merge, it
reverses an assertion phase 1 wrote deliberately, and one existing test had to
change its expected value with the reason written into it rather than quietly
swapped.

### 3. The plan's guard-record count is a count of mentions

The plan says twenty-six records name `messages.rs`. That is a grep. What the
count check flags is different, and it flagged twenty-one. This is the third
plan in this phase to carry the same figure and the second correction of it.

### 4. `threads_holding_any_of` is at `:1151-1225` and `rejoin` at `:195`

Both had moved before this plan started. Everything here was located against the
tree rather than against a line number, which is the standing advice in this
phase and is now four for four.

## Deviations

**Tasks 1 and 2 were committed as two red commits and one green, rather than two
red and green pairs.** Both tasks add tests to `src/data/message_cache/messages.rs`,
and the count check charges a build and a full run for each of the twenty-one
records that name that file, once per commit that has to be green. Splitting them
pays that twice for one plan, and the second run finds what the first did. The
red half of red/green survives whole: every test is red at its own commit, and
each is red on the thing it is about. The reason is in the second red commit's
message. This is `CLAUDE.md`'s "the fingerprint can only be paid once" applied to
a plan rather than to a single commit.

**The first attempt at that re-measurement was started and stopped.** It ran for
two records against the tree as it stood after task 1 and was interrupted once it
was clear task 2 would change the same count again. The tree was restored from
`HEAD` and task 1's implementation re-applied by hand. Recorded because the
interruption left the tree with a guard's break still applied, which `git status`
showed and a blind continuation would not have.

**Task 3's guard record replaced an existing one rather than adding a fourth.**
See the verification section. The file's record count is unchanged, so the sweep
header's arithmetic did not move a second time.

**The roadmap's third inherited bullet is struck through here.** It says
`next_local_uid` is outstanding; 03-03 closed it and its docs commit missed the
line, and 03-04 found it and left it alone rather than edit another branch's
record. This branch holds both, so there is nothing to conflict with.

## What this cannot see

**Nothing here has run against a real account**, because this program has never
been used with one. That a real client sends `In-Reply-To` without `References`,
that a conversation root really does arrive after a message naming it during a
live sync, and that the first open after this change is bearable on somebody's
real mailbox are all unverified. Recorded in the ledger.

**A merged conversation can settle under an identifier that is nobody's root.**
For a chain naming only its parent the two candidates carry no ordering between
them, so the least is chosen and it can be a message in the middle. Stable and
arbitrary, and stability is what was needed. Recorded in the ledger.

**A merge renames a conversation and the running program says nothing.** The
changelog says a conversation may change which message it is filed under; the
interface does not, and whether somebody reading one notices it move under a
screen reader is unverified by ear. Recorded in the ledger.

**The first open after this change pauses with nothing reported.** Between five
and a half and six and a half seconds at two hundred thousand messages on this
computer, once, and more on a larger mailbox. Recorded in the ledger.

## Owed

**The phase-end guard sweep**, per `CLAUDE.md`. Three records were measured by
hand here and re-measured against the finished tree, one was repointed after its
break was measured as no longer discriminating, and the twenty-one the count
check flagged were re-measured, two of them twice.

**`scripts/guards.sh --touched-by 1e4777d`**, which the plan asks for after the
merge. This branch changes `messages.rs`, which twenty-one records carry a count
for.

**Nothing was merged and nothing was pushed.**

## Requirements and criteria

DEFER-1-02 is closed. Broken windows ledger entries 7 and 9 are marked fixed:
entry 7 is the merge itself, and entry 9 records that plan 01-13's
order-independence criterion was unsatisfiable with the lookup's signature,
which this plan changed. Four new entries were recorded, listed above.

**This plan closes no phase 3 success criterion**, and says so because the plan
asked it to be said. Phase 3's criteria are about syncing at scale, the
connection budget, offline mode and conflict resolution; this is phase 1's
second deferred defect, carried here because it is threading work in the data
layer that phase 3 was already touching.

## Self-Check

- All five commits are in `git log`: 886f3ee, 43332f6, a307922, 385de22, e10d998.
- `src/data/message_cache/messages.rs` holds 179 test functions, up from 169.
  `src/application/thread_identity.rs` holds 31.
- `guards/guards.toml` holds 579 records, two of them new here and one
  repointed, and the sweep header's two numbers add up to it.
- `cargo test --test house_style` passes, so every record still names one place
  in the tree, every test a record names exists, and every count is the tree's.
- `grep -n '^version' Cargo.toml` reads `version = "0.49.1"`.
- `docs/changelog.md` carries the entry under `[Unreleased]`, Fixed, with the
  rename said plainly and the known limitation named.
- `scripts/check.sh all` passed on this branch at the end of the plan.
- The working tree is clean apart from this document and the planning files
  committed with it.

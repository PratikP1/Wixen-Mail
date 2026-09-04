---
phase: 03-mail-at-scale-on-the-wire
plan: 03
executed: 2026-09-04
status: complete
tasks: 4
requirements: [SCALE-04, DEFER-1-03]
subsystem: data, application
tags: [query-plan, migration, partial-index, census, guard-record, scale-04]
commits:
  - 9300f25 test(03-03) failing tests for a listing asked about by taking the text away
  - 3b438fd A folder listing is asked what it reads by taking the text away
  - 98fdb49 test(03-03) failing tests against a database this program never wrote
  - 57c9e4d A database written by the old schema opens, migrates, and loses nothing
  - dc86d18 test(03-03) failing tests for the read of every message on every open
  - b8c96a5 A cache that has been migrated stops paying for it on every open
  - 73f6021 test(03-03) the wrap phase 1 deferred, stated as a property of the answer
  - 6c15f3b The wrap answers a number a row can hold, and the dispatcher is guarded
merged: not merged, and not pushed
key-files:
  created: []
  modified:
    - src/data/message_cache/messages.rs
    - src/data/message_cache/bodies.rs
    - src/data/message_cache/mod.rs
    - src/application/mail_sync.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
    - Cargo.lock
requires: []
provides:
  - "a guard saying a folder listing reads no message text, proved by taking the text out of the database"
  - "a database written by the schema that kept text inline, as a fixture"
  - "an indexed answer to the migration that runs on every open forever"
  - "a census of who asks for the counting-up numbering without going through the dispatcher"
affects:
  - "anything later that adds a column to messages or a second place to keep message text: the listing guard is closed and will refuse it until somebody writes the sentence"
decisions:
  - "The listing guard asks SQLite to resolve names against a database holding only what a listing may read, rather than reading a query plan. A plan names cursors by their alias and lists tables, and the break this is about is a column in a table a listing is allowed to read."
  - "Nothing records that the inline-body migration has been done. Following the plan's own safety ordering, a marker that never gates the question decides nothing, and a partial index gives the whole saving with no state that can be wrong."
  - "The wrap is made safe as well as guarded, and next_local_uid's doc comment says the dispatcher is what decides, so the dispatch tests do not read as duplication."
metrics:
  duration: about 3 hours 20 minutes
  files: 8
  commits: 8
actuals:
  tokens: 24000
  tasks: 4
  commits: 8
---

# Plan 03-03: Proofs for the split that already ships, and one real change

**One-liner:** A folder listing is proved to read no message text by being run
against a database with the text taken out, a database written by the old schema
is a fixture rather than an assumption, the migration that runs on every open
forever stopped reading every message to find nothing, and phase 1's
`next_local_uid` wrap is closed where its guard actually lives.

## What works

### Task 1. A folder listing reads no message text, and SQLite says so

`a_listing_reads_no_message_text` builds a database from one this program really
wrote, copying out only the three tables a listing may read and then dropping
the two inline body columns from `messages`, and asks SQLite to prepare every
query a listing runs against it. Name resolution is SQLite's own answer to
"what does this read", and it reaches a column, a subquery and a view alike.

2,793 queries: both shapes of the message listing and the conversation listing
in all 930 orders a `Sort` can express, plus the three defaults and All Inboxes.
0.39 seconds. Three companions prove the question can see a violation: a query
reading `message_bodies`, one reading either inline column, and one reading a
table outside the allowed set.

The allowed set is a named constant with a sentence per entry. The one written
for `attachments`:

> asked whether any exist, at the EXISTS in listing_query, and never for their
> content. A paperclip on a row is a yes or a no, and the file it stands for
> lives in attachment_content, which is not on this list

### Task 2. A database written by the old schema opens, migrates and loses nothing

The fixture writes the shipped `folders` and `messages` tables out directly, with
text in the inline columns and no `message_bodies` table at all, closes the file,
and hands it to the real `MessageCache::new`. Four rows: plain only, HTML only,
both, and neither. Its comment says it is a copy of a schema that shipped and
must not be updated to match today's.

**All five tests passed on arrival, and that is the result rather than the
absence of one.** `migrate_inline_bodies` has run on every cache open since the
text moved, against databases nothing had ever tested it on. It does what it
says: every message that had text has it afterwards, the inline columns are
empty so the space is reclaimed, the count is unchanged including the message
that never had any text, the copy the migration has not finished with wins where
both places hold one, and a second open changes nothing.

A sixth test says a sync over a message carrying an attachment writes no
attachment file and no attachment row. It passed on arrival too.

### Task 3. The read of every message on every open is now an indexed lookup

One partial index, `idx_messages_inline_body`, whose `WHERE` is the migration
query's `WHERE` exactly. Measured on a release build at two hundred thousand
messages with all of their text already moved, warm:

| | reading every message | reading the index |
| --- | --- | --- |
| `migrate_inline_bodies` | 32 ms | under 0.1 ms |

The index costs 8,192 bytes while it holds nothing. Cold it is worse than that
for the read and about the same for the lookup, because the read goes through
the whole file and the lookup touches two pages, so the saving is larger and not
smaller.

The migration is unchanged and still runs on every open, which it must:
`body_plain` and `body_html` shipped in the original `CREATE TABLE` and are never
dropped here, so anything that stops running it leaves message text inline with
nothing left to move it. Version 0.47.0 to 0.48.0, with the changelog entry
carrying both numbers and saying what they were measured on.

### Task 4. The wrap, and the thing that really keeps it unreachable

Asked for the next locally filed number in a folder holding a row at the top of
the range, the answer was zero: a number no message has, and one the next row
would be given as well, so the second would write over the first. It saturates
now, at the top of the range.

The dispatch is what the tests are really about. Fourteen cases: six folder paths
taken from `local_folders` rather than written out, so a folder added there is a
folder this asks about; seven shapes a server can name, two of them a server
mailbox called "Local", because the reserved prefix is a control character a
mailbox name cannot carry and the word after it is there to read; and the
no-such-folder arm, with the comment saying why counting down is the safe answer
when the move is going to fail on the foreign key anyway.

A census over the shipping half of `src` finds the two places that ask for the
counting-up numbering directly, the dispatcher and a POP account's own inbox, and
nothing else. It names a site by its line of code rather than by a line number,
which is the artefact this phase keeps finding stale.

## Verification

Every commit went through `scripts/check.sh` by way of the commit hook. Nothing
used `--no-verify`.

**Four reds, all accepted by `scripts/red-commit.sh`.** Task 1's four tests and
task 2's five fail on a fixture that is not written yet rather than on a compile
error, because a stubbed body is a stronger red than a missing name. Task 3's two
fail printing the plan they got, which is `SCAN messages`. Task 4's one fails on
the answer:

```
assertion `left != right` failed: the next number after the top of the range came back as zero
  left: 0
 right: 0
```

**Task 1's guard was seen red against a listing that reads message text**, which
is the break its record carries: the snippet column falling back to the inline
body when a row has no snippet. The whole library, 6,081 passed and one failed:

```
1861 of the 2793 queries a folder listing runs read something a database holding
only what a listing may read does not have. ...
   no such column: m.body_plain in SELECT m.id, m.uid, ...
                m.to_addr, m.cc, m.reply_to, m.date,
                COALESCE(m.snippet, m.body_plain), m.size_bytes, ...
```

**Six guard records, each measured by hand before it was written.**

| record | what really went red |
| --- | --- |
| a folder listing reads no message text | 1 test, 6,081 passed |
| the inline body migration moves every row it found | 5 tests |
| a sync writes no attachment for a message nobody has opened | 1 test, 6,087 passed |
| the migration reads an index rather than every message | 2 tests, 6,090 passed |
| nothing remembers that the migration has been done | 2 tests, 6,090 passed |
| which end of a folder's numbering a filed row takes | 5 tests, four already in the tree |
| nothing new asks for the counting-up numbering directly | 2 tests |

Two of those are findings rather than counts.

**The dispatch break reddens four tests that were already there**, and they had
been holding that rule from four directions with nothing saying so: an import
into a folder a server fills, a sent copy filed here, and two in the cache about
a message filed into such a folder. Writing down what you expect would have
named one.

**The listing break reddens exactly one test in the whole tree**, because the
fallback changes no answer for a row that has a snippet and every fixture here
has one. That one test is therefore the whole defence, which is what the record
says.

**Guard records re-measured for the changed test counts:** 14 after task 1, 5
after task 2, 2 after task 3, and 15 after task 4. All agreed with what they
name except one, below.

**A record written earlier in the same session was already short.** The record
for the inline body migration named four tests when task 2 wrote it. Task 3 added
a test asking that text put back inline is moved by the next open, which reaches
the same rule and reddens on the same break, and the count check caught it within
the hour. That is `CLAUDE.md`'s stale record happening at its shortest possible
range. Its red list is five now and the record says what happened.

**`scripts/check.sh all` ran twice.** Once as the gate for the task 3 green
commit, because a `Cargo.toml` change makes `which-checks.sh` answer `all`: four
checks, the whole suite and the release build, all passing. Once at the end of
the plan.

## Premises that were wrong

### 1. The plan's guard would have been green through its own break

It asked for a check over the table names in a query plan, with `listing_query`
changed to select a body column as the break. Two things go wrong. A query plan
names cursors by their alias, so turning `m` back into `messages` means reading
the query, which is the copy `listing_query`'s doc comment argues against. And
`messages` is a table a listing is allowed to read, while `body_plain` is a
column of it: a check over table names objects to nothing when the listing
selects message text out of the table it is already reading.

That is not a corner case. Those two columns shipped in the original
`CREATE TABLE` and are never dropped, so the inline columns are a place message
text can be read from for as long as this program exists, which is the same fact
task 3 is about. The question is asked by prepare-time name resolution against a
database with the text taken out instead.

### 2. "That test has never been written" is wrong about the plan reader

Three tests already ask SQLite for `listing_query`'s plan, in `mod.rs`'s
`storage_shape` module, about sorting and about the attachment check, and
`how_it_will_be_answered` has been there since. What had never been asked is
which tables and columns a listing reads. Task 3 uses that reader, which is why
it is worth saying rather than only correcting.

### 3. The marker decides nothing, so there is none

The plan asked for a marker recording that a database has been migrated, with a
cheap probe in front of it that the marker never gates, and stated that ordering
as the whole safety argument. The ordering is right. Following it through removes
the marker: the probe answers in both branches, so the marker is written and
never read.

The index alone is the whole saving, and it is a better answer than the marker
plus the probe, because it makes the question free rather than making a wrong
answer harmless. There is no state left that can be wrong. A one-row table
nothing reads would also be permanent, because schema changes here are additive,
and guardrail 3 is about exactly that shape.

The invariant the plan wanted written into the code is written into it, as the
reason not to add one, in `mod.rs` beside the index and in the constant holding
the query. The guard record whose break the plan described as "make the marker
gate the probe" is the same idea with the marker the next person would reach
for: take "the body table holds something" as a marker and stop asking. It
reddens the test written for it and nothing else unrelated.

### 4. A sync writes no attachment description either

Task 2's fifth behaviour expected the description written and only the file
withheld. `ImapMessage` carries `has_attachments` and no list, because a header
fetch does not read a message's structure, and the only thing that writes an
attachment row is `replace_attachments_with_content`, whose one production caller
is the reader on a message somebody has opened. Neither is written. The guard
record's break is a sync that writes the description, because that is the half a
sync could really do.

### 5. The guard record counts are counts of mentions

The plan says 26 records name `messages.rs`, 18 name `mod.rs` and 2 name
`bodies.rs`. Those are right as a grep. What the count check flags is different:
it resolves each record's red list to files and adds the file the break is
applied to. Measured on this tree, a test added to `messages.rs` flags 14
records, to `mod.rs` 9, and to `bodies.rs` 1. The plan's placement conclusion
still holds, at a ninth of the ratio it was argued from.

### 6. Phase 1's own note about the wrap was wrong about the fixture

`deferred-items.md` says there is no test "because building a fixture with four
billion rows is not a test anybody would run". The function reads `MAX(uid)`, so
one row inserted at the top of the range is the whole fixture. Believing
otherwise is what kept this untested for a phase and a half. The entry is
corrected.

## Deviations

**The attachment test lives in `mail_sync.rs`, not `bodies.rs` where the plan put
it.** The scripted server it needs is private to that module's tests. That cost
four more records at the re-measure than the plan budgeted.

**A test count taken with the wrong rule reads exactly like a stale record.** Two
records were written with `sent_copy.rs` at 20 tests, from a grep for lines
containing `#[test]`. `how_many_tests_are_in` counts a trimmed line equal to
`#[test]` or `#[tokio::test]`, and that file has two of the latter, so the check
failed naming records that were not stale at all. Worth knowing, because the
message is the same either way.

**The measurement was taken with a temporary test that was then removed.** A
`#[test]` building 200,000 messages, timing `migrate_inline_bodies` five times
with the index and five times after dropping it, and comparing the vacuumed file
size both ways. It is not in the tree: it takes minutes and asserts nothing. The
numbers it produced are in the constant's doc comment, in the changelog and above.

## What this cannot see

**The listing guard covers the queries `messages.rs` builds.** A listing that
went round `listing_query` and `conversations_query` entirely would not be
covered, and neither would a body fetched in Rust after the query returns. What
it does close is the shape criterion 4 is about: a query dragging message text
through SQLite to show a subject line.

**The census reads source.** It says where a call is written, not whether that
code is reached, and nothing about how a folder is chosen at run time. The
dispatch tests are what cover the decision itself.

**Nothing here has run against a real account.** The 32 ms and the tenth of a
millisecond were measured on a database this suite built, on this computer, warm.
Both numbers are conditions-bound and the changelog says what they were measured
on.

## Owed

**The phase-end guard sweep**, per `CLAUDE.md`. Six new records went in here and
every one was measured by hand as it was written.

**Nothing was merged and nothing was pushed.** `scripts/check.sh all` passed on
this branch at the end of the plan.

## Self-Check: PASSED

- All eight commits are in `git log`: 9300f25, 3b438fd, 98fdb49, 57c9e4d,
  dc86d18, b8c96a5, 73f6021, 6c15f3b.
- `guards/guards.toml` holds 573 records, six of them new here, and the sweep
  header's two numbers add up to it.
- `cargo test --test house_style` passes, so every record still names one place
  in the tree, every test a record names exists, and every count is the tree's.
- `scripts/check.sh all` passed on this branch.
- The working tree is clean apart from this document.

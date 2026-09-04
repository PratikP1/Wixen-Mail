---
phase: 03-mail-at-scale-on-the-wire
plan: 04
executed: 2026-09-04
status: complete
tasks: 2
requirements: [DEFER-1-01]
subsystem: data
tags: [gmail, all-mail, conversation-count, identity, guard-record, defer-1-01]
commits:
  - f70e1f2 test(03-04) failing tests for mail archived with no label
  - 0d9d413 A conversation is counted by which messages it has, not by which folders
  - 7666286 The count and the delete are held to each other, and the identity is guarded
merged: not merged, and not pushed
key-files:
  created: []
  modified:
    - src/data/message_cache/messages.rs
    - guards/guards.toml
    - docs/changelog.md
requires: []
provides:
  - "identity-based counting in the conversation listing and the messages-of-one-conversation query, with the folder exclusion gone from both"
  - "one text deciding which messages exist for both queries, so D-07's coupling is structural rather than described"
  - "a test holding the row's count and the action's list to each other over every threaded fixture in the module"
affects:
  - "03-05, which merges conversation roots: more merges firing means more conversations whose messages span All Mail and a label, which is exactly what this counts"
  - "anything later that adds a folder-shaped exclusion to a query about messages: three guard records now say what that costs"
decisions:
  - "Which message a row is gets its own named expression with three arms, Gmail's identifier then the sender's Message-ID then the row itself, rather than searching.rs's COALESCE(gmail_msgid, id). The two-arm version would have traded the archived-mail defect for a doubled count on any RFC 6154 server that is not Gmail."
  - "The exclusion is narrowed rather than replaced: an all-mail row is dropped only when some other folder in reach already holds that message. Deduplicating every row by identity would have fixed more and broken the delete, because the action must still reach both rows it acts on."
  - "The two queries are built from one function rather than checked for agreement. A guard over two things built from one source cannot be falsified by editing that source, so the record's break is putting the divergence back."
metrics:
  duration: about 7 hours 15 minutes
  files: 3
  commits: 3
actuals:
  tokens: 11000
  tasks: 2
  commits: 3
---

# Plan 03-04: A conversation is counted by which messages it has

**One-liner:** Gmail mail archived without a label now counts toward its
conversation, because the count stopped excluding a folder and started
recognising a copy, and the number a delete confirmation names is held to the
number in the list by a test rather than by a comment.

## What works

Standing in All Mail with the count reaching the whole account:

- a conversation every one of whose messages was archived without a label is in
  the list, with its message counted. It used to have no row at all.
- a conversation with one labelled message and one archived one says two. It
  used to say one.
- its unread count says two as well, which is what decides whether the
  conversation reads as unread at all.
- the delete confirmation names two messages and the deletion takes two.

Standing in the inbox, the same conversation says two from there too. Nothing
about an ordinary folder changed: the thirteen tests in this module that read a
conversation listing from ordinary folders pass unchanged, with the same
expected numbers.

`reach` no longer drops folders. `elsewhere` is the set of messages some folder
other than an all-mail one holds, and `here` drops an all-mail row whose message
is in that set. So a copy is left out and a message with no copy anywhere else
is counted.

`WHICH_MESSAGE_THIS_ROW_IS` decides which message a row is, three arms, tagged
so they cannot collide:

```sql
CASE
    WHEN m.gmail_msgid IS NOT NULL THEN 'gmail:' || m.gmail_msgid
    WHEN TRIM(TRIM(m.message_id), '<>') <> ''
        THEN 'header:' || TRIM(TRIM(m.message_id), '<>')
    ELSE 'row:' || m.id
END
```

The two queries are one text now. `conversation_scope` is the whole `WITH`, and
`conversations_query` and `messages_in_one_conversation` are that plus their own
`SELECT`. The doc comment above them has said since it was written that they
must agree and that two queries agreeing today would drift the first time
somebody changed one. That is now true by construction rather than by care.

## How I know a disappearance would be noticed

The failure this plan is about is invisible: a conversation missing from a list
is not something anybody can see is missing. So one of the four tests asserts
presence rather than a number, and its failure prints the whole list it looked
in. This is what it said before the fix, and the conversation that is not there
is the point:

```
the conversation whose every message was archived without a label is missing from [
    ConversationItem {
        thread_id: "root@example.com",
        subject: "Quarterly report",
        messages: 1,
        ...
    },
]
```

That is one conversation listed where two exist, and `lunch@example.com` is
gone. Three ways this stays noticed rather than becoming a test that once
passed:

**It was seen red before the change**, on its own and again inside the whole
library, and the guard record carries that measurement. Putting the folder
exclusion back reddened four tests and nothing else, 6,101 passing, against a
library of 6,105. Task 2 added a fifth to that break and the record says which
and why.

**Its record says what the other guards cannot see.** The agreement guard
compares the row's count with the action's list, and under this break both lose
the same rows, so they still agree. The double-count guard asks whether anything
is counted twice, and a copy that is excluded cannot be counted at all. Neither
can see this defect. That is written into the record rather than left for
somebody to work out.

**The count check reaches it.** Twenty-one records carry a test count for
`messages.rs` now, so a test added anywhere near this rule flags them and the
scoped re-measurement runs.

## Verification

`scripts/check.sh all` passed on this branch after the last commit: formatting,
clippy with `-D warnings`, 6,113 library tests, and the release build.

Every commit went through the gate by way of the commit hook. Nothing used
`--no-verify`.

**One red, accepted by `scripts/red-commit.sh`.** Four tests failing on values
and on an absence rather than on a compile error, plus the count check, named
with the reason `CLAUDE.md` gives.

**Task 2 was green on arrival, all eight tests, and that is the result rather
than the absence of one.** The plan predicted it and asked for it to be reported
per behaviour:

| behaviour | on arrival |
| --- | --- |
| a message under a label and in All Mail is counted once, from either folder | green, and covered twice already by tests task 1 re-fixtured |
| the count and the delete agree over every threaded fixture | green, and it had never been asked over more than one fixture |
| two genuinely different messages with no identity are two | green |
| the two queries carry the same construction | green, and it is now one text rather than two that match |

**Five guard records, each measured by hand against the whole library.**

| record | break | what really went red |
| --- | --- | --- |
| mail archived with no label counts toward its conversation | the folder exclusion put back | 4 tests, 6,101 passing, then 5 at task 2 |
| a folder holding a copy of every message is left out (moved) | the duplicate condition made always true | 5 tests, then 7 at task 2 |
| the two queries decide which messages exist with one text | one query given its own scope | 3 tests, 6,108 passing |
| a message with no identity of its own is only ever itself | the row fallback made a constant | 1 test, 6,110 passing |
| a copy recognised by the sender's identifier when the server gives none | the Message-ID arm removed | 2 tests, 6,111 passing |

The passing count is given where the break was taken by hand with a whole-library
run in front of me. The two measured through `scripts/guards.sh --remeasure`
report which tests went red and that nothing else did, which is the assertion
that matters, and do not print a total; those rows say so rather than carrying a
number nobody took.

The fourth is a finding rather than a count. One test is the whole defence of
that arm, because every other fixture in the module gives its messages
identifiers, so the arm decides nothing for them.

**Guard records re-measured: 17 after task 1, then 18 after task 2.** Four fell
short across the two runs, all four because these tests reach rules those
records are about:

- the writer record, 34 to 38 after task 1 and 38 to 41 after task 2. Every one
  of these tests asks what a conversation holds, so a column nothing fills
  leaves them nothing to ask about.
- the all-mail record, 5 to 7 after task 2, gaining both tests about a copy on a
  server that gives no identifier of its own.
- this plan's own archived-mail record, 4 to 5 after task 2, written an hour
  earlier in the same session. `CLAUDE.md`'s stale record at its shortest
  possible range, again.

## Premises that were wrong

### 1. `COALESCE(m.gmail_msgid, m.id)` is not enough, and the tree says so

The plan's fourth premise correction says to reuse `searching.rs:539`'s
expression, and its must-have truth says a message with no Gmail identity falls
back to its row. Follow both and the fix trades one defect for another.

`\All` is RFC 6154, not a Gmail invention. `special_use::holds_all_mail` reads
that attribute from whoever sends it, so a server can present every message a
second time and never answer `X-GM-MSGID`. Under a two-arm identity those two
rows are two messages, and with the folder exclusion gone every conversation on
such a server would report twice its size. That is the exact harm the
exclusion's own doc comment says it exists to prevent, so the fix would have
removed the guard and kept the wound.

The Message-ID arm is what stops that, and the trim on it is ledger 8: the
column held both spellings until 02.1-06 and the backfill that corrects the
older rows is not fatal when it fails, so a database it never finished still
holds bracketed values.

The row fallback stays as the last arm, and it is the one case still wrong: two
copies carrying neither identifier count as two. A count that is too high is
visible and a conversation that has vanished is not, so that is the direction to
be wrong in. Recorded in the ledger rather than glossed.

### 2. A test named for copies had a fixture holding none

`test_a_folder_holding_a_copy_of_every_message_does_not_double_the_count` wrote
four messages with four different `Message-ID`s, two of them in All Mail and
nowhere else, and called them copies. It passed, and it could not have failed:
the folder exclusion drops All Mail before any identifier is looked at, so the
fixture's identities decided nothing.

Under a count that asks which message a row is, that fixture answers four. Both
All Mail tests were re-fixtured in the RED commit to hold real copies, same
`Message-ID` and same Gmail identifier, and both stayed green there. That is what
says the correction is not what makes the new tests fail.

The general shape is worth carrying: a rule implemented as a whole-container
exclusion makes the property the correct rule turns on irrelevant to every test
of it, so those fixtures drift and nothing can say so. Replacing the container
rule with the property re-tests the fixtures as well as the code.

### 3. The agreement guard's break, as the plan specified it, cannot work

The plan says to break the agreement by "changing the thread qualifier in one
query and not the other". There is no other now: both read one text, so editing
it moves both together and they still agree. A guard over two things built from
one source cannot be falsified by editing that source.

The break has to put back the divergence the sharing removed, which is also the
defect the guard exists to prevent, and that is the tell that it is the right
one. Written as the most plausible version of the mistake: the query as it stood
before this change, with the thread qualifier left out.

### 4. The doc comment for the listing was attached to the wrong item

`messages.rs:77-116` described `conversations_query` and ran without a break
into the comment on `MESSAGES_IN_ONE_CONVERSATION` at `:117`. One doc comment,
all of it on the constant. The listing had none and the constant had two, and
the plan's `key_links` cites `:117-126` because that is where the coupling
sentence really was. Split as part of rewriting both.

### 5. Twenty-six records name this file, and that is not the number that fires

The plan repeats the figure 03-03 had already corrected. 26 is a grep for
mentions. What the count check keys on is the record's `tests_last_seen` entry
for a file: it flagged 17 records after task 1 and 18 after task 2, and after
this plan's three new records 21 carry a count for `messages.rs`. Eighteen apply
their break to it. The plan's placement conclusion holds; every number in it is
about a different question from the one it is used to answer.

## Deviations

**The reading guard reads the built strings, not the source.** The plan asked
for a check over the shipping half of the file comparing two query texts
normalised for whitespace. The queries are built at run time, so the test asks
`conversation_scope()`, `conversations_query()` and
`messages_in_one_conversation()` directly, which is the thing that runs rather
than a reading of the thing that runs. Three companions, one of them the one
`CLAUDE.md` asks for: a pair that differs is caught; an empty scope is refused,
because every string begins with one and a reader that had stopped reading would
otherwise say yes about any two queries; and the scope is asserted still to name
its three parts and its three parameters, so "the same scope" cannot become "the
same nothing".

**No separate double-count test was written.** The plan asked for one. Two exist
and both were corrected in the RED commit to hold real copies, one standing in
the inbox and one in All Mail, which is the behaviour as the plan words it. A
third copy of them would be duplication.

**Two more tests than the plan asked for**, both about the Message-ID arm, one
for a server that gives no identifier of its own and one for a row still holding
the bracketed spelling. Without them the arm this plan added against its own
instructions would have been unguarded, and the next reader would have had every
reason to simplify it away.

## What it costs, measured

Release build, warm, 200,000 rows in 10,000 conversations, the same sixteen
columns in all three so only the scope differs:

| | with a folder holding all mail | with no such folder |
| --- | --- | --- |
| shipping, the duplicate filtered | 1.2 s | 0.86 s |
| no filter, copies counted | 0.89 s | 0.88 s |
| the folder excluded, as it was | 0.75 s | 0.85 s |

So on Gmail the listing costs about 60 percent more, roughly 450 ms at that
size, of which about 300 ms is the filter and about 150 ms the extra rows now in
reach. On an account with no such folder there is no measurable difference,
which was the claim in `conversation_scope`'s doc comment and is now a
measurement rather than an assumption.

Neither number is good on its own terms. `conversations_query` has no `LIMIT`
and groups the whole account on every listing, which was true before this change
and is SCALE-03's subject. Recorded in the ledger.

Measured with a temporary `#[ignore]` test that is not in the tree: it builds
200,000 rows with raw inserts and asserts nothing.

## What this cannot see

**Nothing here has run against a real Gmail account**, because this program has
never been used with an account at all. Everything above is evidence about the
SQL and none of it is evidence about what Gmail sends. Named precisely in the
ledger: that a message archived without a label really appears in All Mail and
nowhere else; that `X-GM-MSGID` really comes back on the same message under a
label and in All Mail; that `holds_all_mail` is really set for Gmail's All Mail
by a live `LIST`.

**A Gmail message under two labels still counts twice.** Both label rows are
real rows outside All Mail and nothing says which label should lose. Unchanged
by this plan, which neither introduced it nor fixed it. Fixing it needs the
count and the delete list to become different questions, since the delete must
still reach every row it acts on, and that is an architectural decision rather
than a predicate.

**`searching.rs:539` has the weaker identity this plan replaced.** On a server
that advertises `\All` and gives no Gmail identifier, a search shows the same
message twice. Same class, same remedy available, pre-existing, out of scope.
Recorded.

## Found and not fixed

**`ROADMAP.md`'s third inherited bullet still reads as open.** 03-03 closed the
`next_local_uid` wrap and its docs commit updated `REQUIREMENTS.md` and
`deferred-items.md` but not the roadmap, so the line saying the wrap is
outstanding is still there. Not corrected here, because editing another plan's
record on a branch neither of us has merged invites a conflict at the merge that
is worse than the staleness. This plan's own bullet is struck through, with the
size claim corrected in place rather than deleted.

**The four ledger entries were first written into the shared checkout, not this
branch.** `gsd-tools.cjs` lives under the main repository's `.claude`, which is
not present in the worktree, so running it from there wrote to
`Wixen-Mail/.planning/WINDOWS.md`. Found by comparing the two files, moved onto
this branch, and the shared checkout restored to its committed content. Worth
recording because the failure is silent: the command reports success and the
entries are simply somewhere else.

## Owed

**The phase-end guard sweep**, per `CLAUDE.md`. Three new records went in here
and one moved; every one was measured by hand.

**Nothing was merged and nothing was pushed.** `scripts/check.sh all` passed on
this branch at the end of the plan.

## Self-Check: PASSED

- All three commits are in `git log`: f70e1f2, 0d9d413, 7666286.
- `src/data/message_cache/messages.rs` holds 169 test functions, up from 157.
- `guards/guards.toml` holds 577 records, three of them new here, and the sweep
  header's two numbers add up to it.
- `cargo test --test house_style` passes, so every record names one place in the
  tree, every test a record names exists, and every count is the tree's.
- `grep -n '^version' Cargo.toml` reads `version = "0.48.0"`, unchanged.
- `docs/changelog.md` carries the entry under `[Unreleased]`, Fixed, with both
  limitations.
- `scripts/check.sh all` passed: 6,113 library tests, 0 failing.
- The working tree is clean apart from this document.

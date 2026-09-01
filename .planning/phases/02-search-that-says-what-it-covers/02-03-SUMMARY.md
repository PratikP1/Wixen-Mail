---
phase: 02-search-that-says-what-it-covers
plan: 03
subsystem: search
tags: [saved-searches, backfill, imap, allowed, accessibility, guards]
status: complete
requires:
  - "Allowed::reading and ImapSession::may_i_read, the read gate plan 02-01 built"
  - "MessageCache::how_much_message_text_is_stored_here, the coverage count plan 02-02 built"
  - "mail_controller::fetch_message_body and service::mime::parse, both already in the tree"
  - "service::outward::read_refusal and was_refused_by_the_gate, the one wording and the one question"
provides:
  - "MessageCache::messages_with_no_text_here and MessageToFetch, the list a fetch would walk"
  - "mail_sync::fetch_the_missing_message_text, the gated reporting backfill, and fetch_over_a_mailbox behind it"
  - "mail_sync::Backfill, Backfilled and Ending, four outcomes rather than a count and a flag"
  - "mail_sync::says_where_it_is, the bound on how often a run speaks"
  - "allowed::FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL, its own sentence beside EXPERIMENTAL_WARNING"
  - "wx_app::the_offer_to_fetch and the button above the message list"
  - "UIUpdate::WhatCouldBeFetched, which puts the offer on the screen and takes it off"
  - "one guard record, measured by hand against the whole library"
affects:
  - "Mailbox gains a sixth method, so every implementation of it must answer a body fetch"
  - "every saved search that reads message text: one extra list query beside the count"
  - "the mail content panel, which now holds a strip above the message list"
tech-stack:
  added: []
  patterns:
    - "a public concrete entry point delegating to a generic core, so the test seam stays out of the public API"
    - "a stub with the real control flow and fixed values, so red comes from a value and never from wiring"
    - "a pure decision function for a rule a window cannot be built to test"
    - "a progress bound asserted as a property over the whole range rather than by reading the constant"
key-files:
  created: []
  modified:
    - src/data/message_cache/bodies.rs
    - src/application/mail_sync.rs
    - src/application/allowed.rs
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - guards/guards.toml
    - docs/changelog.md
decisions:
  - "The offer counts the fetch list, not the difference between the two coverage numbers, because the two sets are not identical and a subtraction would offer to fetch mail with no server to ask."
  - "A refusal by the read gate stops the run; an ordinary failure does not. Going on after a refusal would report hundreds of failures for one setting."
  - "The backfill has a public concrete entry point and a crate-private generic core, because a public function cannot be bounded by a crate-private trait and async fn in a public trait is denied."
  - "The offer is announced on its own topic, not on status, because the coverage sentence is on status and the queue keeps only the most recent of a topic."
  - "No new accelerator, so docs/KEYBOARD_SHORTCUTS.md is unchanged. Tab reaches the button."
metrics:
  duration: one session
  completed: 2026-08-31
actuals:
  tokens: 30800
  tasks: 3
  commits: 6
---

# Phase 2 Plan 3: Fetching the text a search cannot reach Summary

Somebody told that some of this account's messages have no text stored here can
now press a button above the message list and have it fetched, one message at a
time, behind the read gate plan 02-01 built. The button says how many, appears
only while there is something to fetch, and carries a sentence saying the whole
thing is experimental and why.

## What works, and what has not been proved

**Working and exercised end to end in tests.** The list query, the backfill,
the gate, the counting, the reporting bound, the sentences, the offer, its
wiring both ways, and the guard.

**Working and traced to a person on a non-test path**, because that is the
criterion this exists for rather than a claim to make:

1. `run_a_saved_search` has two live callers, both in the window's event
   wiring: opening a saved-search row in the folder tree and Refresh while a
   saved search is open. Plan 02-02 established that and this plan sits on the
   same path.
2. Inside it, beside the coverage sentence, `what_could_be_fetched` is asked
   and its answer sent as `UIUpdate::WhatCouldBeFetched`.
3. `handle_update`'s arm for that update sets the button's label on both
   accessibility channels, shows the strip, lays the panel out again, and
   announces one sentence.
4. The button's `on_click` is registered in the frame's event wiring and calls
   `start_the_missing_text_fetch`, which connects, runs the backfill, and sends
   what it did to the status bar, which announces it.

Each link a test can hold has one, and the one that matters is measured rather
than argued: see the by-hand break below that the first version of the guard
sailed through.

**Not proved, and it needs a real mail server.** Nothing here has run against a
real account, and no test in this repository can settle the one question that
matters: whether a provider will permit a run of hundreds of whole-message
fetches at all, or will throttle it, or will drop the connection. That is
`02-RESEARCH.md`'s assumption A5 and threat `T-02-11`, and it is transferred to
the person rather than resolved: the sentence beside the button says it in
plain words, the run says how many it will attempt before it starts, and it
reports failures rather than swallowing them. Recorded in `.planning/WINDOWS.md`
as entry 11.

**Not proved, and it needs a screen reader.** Nobody has heard this. Three
things to listen for. That the button is announced with its full label when Tab
reaches it, rather than "button" alone. That the sentence beside it is read.
And that the offer's announcement is heard at all: it is on its own topic,
"message text", precisely so it does not silence the coverage sentence on the
"status" topic, and whether two announcements a moment apart on two topics both
arrive is a guess until somebody hears it. Recorded as entry 12.

**A limit worth stating rather than hiding.** The offer appears only while a
saved search that reads message text is running. Somebody who never uses saved
searches is never offered the fetch. That is where D-2-08 puts it, deliberately,
and it is a narrower reach than a menu command would have. Recorded as entry 13.

## Commits

| Task | Commit | What |
|------|--------|------|
| 1 | `bf6f205` | RED: eight failing tests for the list of messages with no text here |
| 1 | `d21ac4e` | GREEN: the query, and the equality that ties it to the coverage count |
| 2 | `6878698` | RED: ten failing tests for the backfill and its experimental sentence |
| 2 | `f108a59` | GREEN: the gated, reporting, one-message-at-a-time backfill |
| 3 | `1d815c9` | RED: four failing tests for the offer beside the sentence |
| 3 | `a1e02ba` | GREEN: the button, its warning, the guard record and the changelog |

## The red half, committed rather than described

Three RED commits, twenty-two named tests, each held to `scripts/red-commit.sh`:
every named test ran, every named test failed, nothing else failed.

**Every red came from a value, and getting there took work in two of the three
cases.** This is the part worth passing on.

**Task 1: the stub was the widest wrong answer, not the empty one.** A query
returning an empty vector makes every absence assertion vacuously green, which
is five of the eight tests. So the stub was the query with no conditions at all,
returning every message in the database. All eight then failed on values, four
of them on set comparisons that named the extra rows.

**Task 2: the stub had the real control flow and fixed numbers.** It walked the
list, asked the server for each message, and reported nought. That is stronger
than a stub returning a constant, because it proves the tests discriminate on
the values rather than on wiring being absent.

**Task 3: the source-reading tests were red against the unwired tree**, and the
pure decision function was stubbed to always offer something so that "no offer
when there is nothing to fetch" was a value failure.

## The by-hand break that found a hole in its own guard, again

The plan asked for one guard record: that the offer reaches the backfill. The
first version of its test looked for `fetch_the_missing_message_text` and
`what_the_fetch_did` in the command's body, in that order, with a window update
after.

Taking the break by hand, which is keeping the call and discarding what it
answered, left it **green**.

The reason is not the one the brief predicted and is worth recording. The
routine opened with

```rust
use crate::application::mail_sync::{fetch_the_missing_message_text, what_the_fetch_did};
```

so both names the test looks for were sitting together on one import line, in
the right order, with neither being called. The test was answered by an import.
A break that removed the entire body would still have passed.

The names are written out in full at their call sites now, with a comment above
them saying why, and the break reddens the test. This is the same defect class
as `02-02`'s reach test and as the acceptance criteria in `02-01` and `02-02`
that a mention answered instead of a use, and it is now the fourth time in this
phase that a source-reading check has been satisfied by something that is not
the thing it is about.

## What was measured, not assumed

**Every break, taken by hand.** Eleven, each reddening exactly what is written
beside it. The four in the first group are the ones the plan asked for, and they
are the reason the equality test is worth having:

| Break | Red |
|-------|-----|
| the list query drops `m.deleted = 0` | 2, the equality and the deleted test |
| the list query drops `f.account_id` | 2, the equality and the two-account test |
| the count query drops `m.deleted = 0` | 2, the equality and 02-02's deleted test |
| the count query drops `f.account_id` | 2, the equality and 02-02's account test |
| the up-front refusal is removed | 1 |
| a stored body is not indexed | 1 |
| an ordinary failure ends the run | 1 |
| a gate refusal is read as an ordinary failure | 1 |
| the fetch warning is folded into the write warning | 1 |
| the offer count is worked out and never said | 1 |
| the window is told a literal nought | 1 |
| the backfill runs and its report is thrown away | 1 |

**The equality test's fixture had to be strengthened before it proved
anything.** As first written it held five ordinary messages in one account, and
under that fixture three of the four breaks above left it green: with nothing
for a condition to exclude, removing the condition changes neither side. It now
carries a deleted message and a second account's message, one per shared
condition, and all four breaks redden it.

**The guard record.** One, measured by hand against the whole library on this
tree: `WIXEN_TEST_THREADS=4 cargo test --lib` with the break applied gave 5,874
passed and 1 failed, the failure being the single test the record names. That is
the same measurement `scripts/guards.sh` takes for one record.
**`scripts/guards.sh` itself was not run, per the brief**, so this record is
verified by that one run and by nothing machine-driven, and it says nothing
about the other 538. The sweep number at the top of `guards/guards.toml` goes
from 346 to 347 in the same edit, and `grep -c '^\[\[guard\]\]'` gives 539,
which is 192 + 347.

**No message text reaches any log line, error, or panic.** Checked over the
whole diff rather than assumed, because this is the one routine in the program
that holds hundreds of message bodies in a row. Three log lines were added in
total:

```
Could not fetch the text of message {uid}: {e}
Could not keep the form a signed message arrived in: {e}
What could be fetched for this account could not be counted: {e}
```

None carries a payload. The errors interpolated into them are `common::Error`
values worded in this codebase: `mime::parse` answers with the fixed string
"The message could not be read" and never the bytes, and `save_message_body`'s
error carries SQLite's own message and not its parameters. The four sentences
this plan sends to the window are built from counts and from
`service::outward::read_refusal`, and none of them touches a message.

**The POP question, checked rather than assumed.** The brief was right that a
POP account has no missing text to backfill, and the reason is stronger than
"in practice": `pop_sync` stores the body in the same call that writes the
message row, and every POP message carries a `pop_uidl`, which
`ONLY_COPY_IS_HERE` names. So the list query is **provably** empty for a POP
account, the offer never appears beside a search over one, and the backfill
would report "nothing to fetch" without opening a connection.
`test_a_pop_account_has_nothing_to_fetch` holds it.

## Deviations from Plan

### 1. [Rule 2 - correctness] The offer counts the list, not the subtraction

**Plan said:** `key_links` requires that "the number somebody was told and the
number the fetch attempts must be the same number, from the same query shape",
and Task 1 asks for a test asserting the list length equals the coverage count's
subtraction.

**Why that cannot hold as written:** covered under "Wrong premises found" below.
The two sets are not identical.

**Done instead:** the offer's number comes from the length of the fetch list, so
what somebody is told will be attempted is what is attempted, in the rare case
as well as the ordinary one. The equality is still asserted, over ordinary
server mail, and it is the load-bearing test the plan wanted. A second test,
`test_a_message_with_nowhere_to_ask_is_missing_text_and_still_not_offered`,
builds the divergent case and holds the chosen answer, so the difference is
written down rather than left for somebody to find.

**Commits:** `bf6f205`, `d21ac4e`.

### 2. [Rule 3 - blocking] The backfill has two entry points

**Plan said:** nothing about visibility.

**Issue:** `-D warnings` denies `dead_code`, and a routine with no caller is
dead until its window wiring lands, which is a different task. Three ways out
were tried and two are wrong. Making `Mailbox` public fails, because `async fn`
in a public trait is denied by clippy. Adding `#[allow(dead_code)]` is what this
project's own rules forbid. Folding the window wiring into the same commit would
have made Task 3's source-reading tests green before they were written.

**Done instead:** `fetch_the_missing_message_text` is public and concrete,
taking a `MailController`, and delegates to `fetch_over_a_mailbox`, which is
crate-private and generic over `Mailbox` and is where every test runs. That is
better design as well as the way out: `Mailbox` is the seam the tests need, and
a caller has a real controller and should not have to know the seam is there.

**Commit:** `6878698`.

### 3. [Rule 2 - missing case] A gate refusal part way through is its own ending

**Plan said:** two behaviours that are in tension. "Turning reading off part-way
through stops the run at the next message" and "a message that fails to fetch
does not stop the run".

**Done:** both, told apart by
`service::outward::was_refused_by_the_gate`, which is the one answer to that
question and already existed. A refusal stops the run and is reported as
`Ending::ReadingWasTurnedOff` carrying the sentence; anything else is counted
and the run goes on. Going on after a refusal would ask the server for every
remaining message, be refused once each, and report a folder full of failures
for one setting.

**Commit:** `f108a59`.

### 4. [Rule 2 - correctness] The progress bound is a tested property

**Plan said:** "reports as it goes", and separately that guardrail 5 forbids
flooding.

**Done:** `says_where_it_is` is pure and separate, and the bound is asserted
over totals from one to two hundred thousand rather than by reading the
constant. At most nine lines between the count a run starts with and the report
it ends with, and none at all for a run of five, where the two sentences either
side of a few seconds are the whole story.

**Commit:** `f108a59`.

### 5. [Rule 1 - bug] The command's imports answered its own guard

Covered above under "the by-hand break that found a hole in its own guard".

**Commit:** `a1e02ba`.

### 6. The plan's `<verify>` block asks for `bash scripts/guards.sh`

Not run, on the brief's explicit instruction that guard re-measurement is off
the critical path. The one record this plan adds was measured individually
instead, which is stronger for that record and says nothing about the other 538.

## Wrong premises found

Reported rather than built on, as asked.

1. **Task 1 asks for two things that cannot both hold, and they contradict each
   other on exactly the data the second one names.** It requires the list length
   to equal `messages - with_text` from `how_much_message_text_is_stored_here`,
   and it requires the list to exclude the mail `ONLY_COPY_IS_HERE` names. Those
   are the same requirement only while no message satisfying
   `ONLY_COPY_IS_HERE` is missing its text.

   That is nearly always true, and it is not an identity.
   `application::importing_messages` logs a `save_message_body` failure and
   carries on by design, with the comment saying why, so a filed message with no
   body row is reachable. The coverage count would call it missing text, which
   it is, and the list must not offer it, because there is no server to ask.

   Neither sentence is wrong on its own and no ordinary fixture contains the
   case, which is why this reads as careful. Resolved as deviation 1: the offer
   counts the list, the equality is asserted over ordinary mail, and the
   divergence has a test of its own. This is the same shape as `02-02`'s finding
   about two answers to one question, seen a step earlier: here the two answers
   were specified rather than written.

2. **The plan's own trap note is right about POP and understates it.** It says
   to "check what your query returns for a POP account before assuming it is
   empty". Checked: it is empty, and provably so rather than incidentally, since
   every POP message carries a `pop_uidl` and that is half of what
   `ONLY_COPY_IS_HERE` matches. The behaviour is deliberate and has its own
   test.

3. **Task 3's acceptance criterion asks for `bash scripts/guards.sh` to run
   unfiltered and pass**, which contradicts the brief and `CLAUDE.md`'s own
   current rule that guard re-measurement is off the critical path. Covered as
   deviation 6. Noted because it is the second plan in this phase whose verify
   block still carries the older instruction.

4. **Everything else in the plan held.** The premise corrections were both
   right: `fetch_body` and `fetch_message_body` exist and were reused rather
   than rewritten, `BODY.PEEK` and all; the single-message download near
   `wx_app.rs:17560` is the shape the backfill follows, signed-mail branch
   included; and `EXPERIMENTAL_WARNING` is about writes, is asserted verbatim by
   six assertions, and was left alone.

## Known stubs

None. Nothing here is a placeholder and nothing was left computed and never
said, which was the specific risk and is the thing three of the eleven by-hand
breaks were taken to check.

The one thing that looks like an absence is deliberate and named above: the
offer is reachable only from a saved search that reads message text, which is
where D-2-08 puts it.

## Threat flags

Nothing new. The register's five entries for this plan stand as written.
`T-02-12` is mitigated and measured: the gate is `fetch_body`'s first line and
is asked once per message because the loop calls that function once per message,
plus an up-front refusal, and removing the up-front check reddens exactly one
test. `T-02-14` is mitigated by the account join with a two-account test that
reddens when the narrowing goes. `T-02-15` is mitigated by the report carrying
both numbers always and by a test that a single failure is counted rather than
ending or hiding the run. `T-02-13` is mitigated by reusing the existing parse
and store rather than writing a second one. `T-02-11` is transferred, as the
register says, and is the honest gap this summary leads with. No dependency
added, `Cargo.toml` unchanged.

## Verification

- `cargo test --all-targets --no-fail-fast`: 5,875 library tests and every
  integration target green, 0 failed, 1 ignored.
- `bash scripts/check.sh` green on all six commits, three of them through the
  `red` path, which additionally proved the named failures were exactly the
  failures.
- `docs/KEYBOARD_SHORTCUTS.md` is **unchanged**, and that is a decision rather
  than an omission: the offer is reached by Tab and carries no accelerator of
  its own, because a new one would collide with the menus.
- `grep -n '^version' Cargo.toml` still reads 0.46.0.
- `grep -c '^\[\[guard\]\]' guards/guards.toml` gives 539, and the two sweep
  numbers add to 539.
- `grep -n 'ONLY_COPY_IS_HERE' src/data/message_cache/bodies.rs` shows it used
  by the new query as well as by the eviction query.
- No `--no-verify`, no `#[allow(...)]`, no new dependency, no AI attribution,
  no em-dash.

## Self-Check: PASSED

Every file in `key-files.modified` exists and differs from `main`. All six
commit hashes resolve on `gsd/plan-02-03`: `bf6f205`, `d21ac4e`, `6878698`,
`f108a59`, `1d815c9`, `a1e02ba`.

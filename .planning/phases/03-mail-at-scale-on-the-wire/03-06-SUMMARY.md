---
phase: 03-mail-at-scale-on-the-wire
plan: 06
executed: 2026-09-04
status: complete
tasks: 3
requirements: [SCALE-02]
subsystem: application, presentation, service
tags: [held-session, reconnect, connection-budget, census, guard-record, scale-02]
commits:
  - b48c713 test(03-06) failing tests for a session that outlives one piece of work
  - 91bff41 A session belongs to an account, not to one piece of work
  - 15a1afb test(03-06) the census asks for nought while eleven places still dial
  - b85e982 Every mail command in the main window uses the account's own session
  - 6a9af87 test(03-06) failing tests for a connection that goes while a session is held
  - e03a9cb A connection that goes is signed in again once, and a second failure is a sentence
  - 4926805 The connections one account opens is a number, and two guards say the tests would notice
merged: not merged, and not pushed
key-files:
  created: []
  modified:
    - src/application/mail_session.rs
    - src/application/mail_controller.rs
    - src/presentation/wx_app.rs
    - src/service/protocols/imap.rs
    - src/common/error.rs
    - src/common/answering.rs
    - src/application/sent_copy.rs
    - tests/one_sign_in_per_piece_of_work.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
requires: ["03-02, whose census is the arbiter of when this was done"]
provides:
  - "one authenticated session per account, held across pieces of work, closed when the account is removed or the program closes"
  - "one reconnect and one retry behind the single unwrap point, with the second failure carried as a sentence"
  - "a stated connection budget of two per account, counting the IDLE connection, with a test that counts sockets at a server"
  - "Error::InPlainWords, for a failure already worded for the person who will read it"
  - "a SELECT that notices the connection died in the middle of it"
affects:
  - "anything later that adds an IMAP command to MailController: it goes through the retry macro or it is the one command that does not retry"
  - "03-07 and anything touching sync_folder: the controller it is handed is now shared and outlives the sync"
decisions:
  - "The session is held per account in one map for the whole program, because the pieces of work that share it are spawn_blocking closures on different threads carrying the account and little else. A session threaded through them as an argument would be a session each in every place that forgot to."
  - "The map's lock is released before anything is dialled; the account's own lock is held across its sign-in, because that is what makes two pieces of work arriving together sign in once. Neither is held while the work runs."
  - "A piece of work that fails after signing in leaves the session where it is. A server that refuses a folder has answered, and a connection that answers is worth keeping. Only a connection that has gone is replaced, and the controller does that rather than the holder."
  - "The controller remembers the account rather than the credential. An access token lasts about an hour and a held session outlives one, so signing in again with the token the connection was opened with would fail on both providers this program supports."
  - "The retry is a macro rather than a closure. An async closure taking &mut ImapSession returns a future nothing stable can prove is Send, and the window spawns this work onto a runtime that requires it."
  - "Error::InPlainWords is a new variant whose Display is the words and nothing else. Every other variant names a layer, which is right for a log and wrong for a sentence a screen reader reads out."
metrics:
  duration: about 5 hours
  files: 11
  commits: 7
actuals:
  tokens: 46000
  tasks: 3
  commits: 7
---

# Plan 03-06: One sign-in per account, not one per command

**One-liner:** Every mail command in the main window now uses the session its
account is already signed in with, a connection that drops is signed in again
once and says so in plain words if that fails too, and the number of
connections one account opens is two with a test that counts sockets at a
server.

## What works

**The census reads nought.** Twelve places in `wx_app.rs` built their own
`MailController` and connected with it, once per command. Marking one message
read was a TLS handshake, a CAPABILITY, a LOGIN and a SELECT. All twelve now
ask `mail_session::the_session_at` for the session that account already has.

**Two pieces of work for one account are one sign-in**, counted by a loopback
server that records what it was told rather than by a counter this program
keeps. Two accounts are two sign-ins, because a session belongs to an account.
Closing an account, or the program, sends LOGOUT rather than dropping the
socket, so the session ends at the server as well as here.

**A connection that has gone is signed in again once and the work is tried
again.** Behind `require_imap`, which was already the one place the session is
unwrapped, so no caller has to remember. One attempt more, never a loop: a
server that drops every connection produces exactly two sign-ins, counted at
the server. If the second attempt fails too, the caller gets this sentence:

> The connection to the mail server was lost. Wixen Mail signed in again and
> tried once more, and that did not work either. Check that this computer is
> online, then try again.

No protocol string and no code, and nothing of the error type's in front of it:
that is what `Error::InPlainWords` is for. It reaches somebody through each
site's own reporting, which is `ErrorOccurred` for the flag path and
`CommandRefused` for the folder commands, and both announce at `High` priority
through `accessibility::announce`, above the `Low` that steady sync lines use.

**What the flag path really says is longer than that, and the extra is not
new.** `ErrorOccurred` writes "Error: " in front of everything, and the flag
path wraps a refusal in "The change did not reach the server, so it has been
undone here: ". So marking a message read against a connection that has gone
twice is spoken as:

> Error: The change did not reach the server, so it has been undone here: The
> connection to the mail server was lost. Wixen Mail signed in again and tried
> once more, and that did not work either. Check that this computer is online,
> then try again.

Every word of that is plain and the sentence still ends with what to do next.
Both prefixes were there before this plan and neither is inspected here. The
folder commands take the `CommandRefused` path and get no "Error: ".

**The budget is two per account and the reason is beside the number.** One is
held by `watch_folder` for IDLE and is open before any mail is fetched, which
is why a budget starting at one would have been wrong before it started. One is
the session this file hands out. Gmail allows fifteen per account and punishes
more, so two leaves room for several accounts on one provider.

## The defect this found

**A SELECT the connection died in reported a folder that opened, and that
deletes mail.**

Found by the red half of task 3 rather than looked for. One of the three
failing tests failed on the wrong sentence: it said a server that hangs up on
every folder open opened one. async-imap 0.11.3's `parse_mailbox` reads
responses until the stream ends and hands back whatever it collected, so a
socket closed before the tagged line comes back as `Ok(Mailbox::default())`:
no UIDVALIDITY, nought messages.

**Corrected on review, 2026-09-04.** This passage said `mail_sync` reads an
absent UIDVALIDITY as a renumbering and calls `forget_folder_messages`, and
that this is the arm 03-01's guard record calls the one that would delete mail.
That is not what happens, and it reads as though 03-01's protection failed. It
held. The test is `matches!((status.uid_validity, stored), (Some(now),
Some(before)) if now != before)`, so an absent value does not match and nothing
is discarded. `Mailbox::default()` gives `uid_validity: None`, which is exactly
the arm 03-01 made safe.

The real chain is worse, because the safety check and the hazard share a cause.
On a dead connection all three commands truncate together. `folder_counts`
comes back with nought messages, the SELECT comes back as an empty folder, and
`list_uids` comes back empty. `sync_folder` claims that empty listing as
`ServerListing::TheWholeMailbox`, honestly, because it really did ask for the
whole folder. Every uid held is then absent from it, so `uids_to_forget` names
all of them and `forget_messages` empties the cache.

`listing_contradicts_the_count` exists to catch precisely that, and it is
`listed == 0 && counted > 0`. The count is nought, so it does not fire. **The
same failure that empties the listing disarms the only check standing in front
of it.**

So a connection dropping across a folder sync would have deleted that folder's
cached copy and reported a sync that worked. The fix below is right either way,
because it stops a command that never completed being reported as one that did,
which is the defect underneath both readings.

`select_folder` is written as a command line now and goes through
`read_command`, which raises a stream that ends before the tagged line as a
lost connection. Same shape and same reason as `set_flag`, whose own comment
records the library helper reporting a refusal as a change that worked.

**This plan is what made it reachable.** Before it, every command opened its
own connection, so a drop mid-SELECT was a moment wide. A session that sits
open between commands is the thing a provider drops.

`folder_counts` has the same shape through `session.status` and is not fixed.
On a dropped connection it answers nought messages and nought unread, which is
a wrong number in the folder tree rather than a deletion. Ledger entry 68.

## What each converted site did on failure, and does now

The plan asks for this per site, because the shapes differ and the failure
paths differ most.

| site | before | now |
| --- | --- | --- |
| marking read, flagging, labelling | `refuse(reason)`, three causes with one sentence each | the same, one sentence carrying whichever cause it was |
| making a folder | "so no folder was made", three sentences | "This account could not be signed in to, so no folder was made. {why}" |
| deleting folders | the same shape, "so nothing was deleted" | the same, one sentence |
| emptying a folder | "so nothing was emptied" | the same, one sentence |
| moving a folder | "so nothing was moved" | the same, one sentence |
| renaming a folder | "so nothing was changed" | the same, one sentence |
| copying or moving one message | `fail(reason)` for all three causes | unchanged: the words were already the same |
| folder subscriptions | silent to the person, nothing logged | silent to the person, and a line in the log |
| the bytes of an attachment | an `Error` to the caller | unchanged in shape |
| fetching missing message text | three sentences | "Signing in to X did not work. {why}" |
| a body in the background | a line in the log | unchanged in shape |
| checking for mail | `fail(reason)`, and a connection status | unchanged in shape |

**Every `disconnect_imap` in the window is gone**, thirteen calls at eleven
sites. Under a held session that call closes a session a sync or another
command is part way through, which is the third behaviour task 2 asks about.
Nothing else closes one now: the account being removed and the program closing
do, and both went in with the holder, bounded at three seconds so a server
that has stopped answering cannot hold the window shut.

**Two things got worse and are said rather than glossed.** The per-site
unusable-port checks are gone, because `a_session_at` asks the same question
and answers it in the same words, so each was a second answer to one question.
Eleven lost nothing by that. Checking for mail lost the offending value: it
used to say "has an IMAP port that is not a number: 14 3" and now says the
account has no usable IMAP port. Ledger entry 69. And
`no_longer_signed_in_to_anything` closes sessions one at a time under one
three-second bound, so several accounts whose servers have stopped answering
share it.

## Verification

Every commit went through `scripts/check.sh` by way of the commit hook.
Nothing used `--no-verify`. Two commits changed `Cargo.toml`, so
`which-checks.sh` answered `all` for both and each ran the whole suite and the
release build. `scripts/check.sh all` passed on the finished branch: the
library is **6,135 passed, 0 failed, 1 ignored**, every integration target
green, and the release build clean.

**Three reds, all accepted by `scripts/red-commit.sh`.**

| commit | named | what they failed on |
| --- | --- | --- |
| b48c713 | 3 | 2 sign-ins where 1 was wanted; no LOGOUT at all, twice |
| 15a1afb | 1 | the census, naming all eleven remaining sites |
| 6a9af87 | 3 | 1 sign-in where 2 were wanted, twice; and a folder that "opened" |

The first red's holder was written naive on purpose, signing in every time and
holding nothing, so each test failed on the number it asserts rather than on a
build that never happened.

**Which tests were vacuous at their red, said plainly.** Three of the nine in
b48c713 passed there and could not have failed: two accounts being two
sign-ins, a refused sign-in leaving nothing held, and one account's sign-in not
waiting on another's. Against a holder that holds nothing, the state they guard
does not exist. They are what would notice the holder being wrong later, and
the first guard record below is what proves two of the others would.

**The budget test passed on arrival and says nothing by doing so.** The holder
that satisfies it landed two commits earlier, so no ordering within this plan
would have made it red. What stands in for that red is the first record below,
measured by hand.

**Six guard records touched, every one measured by hand, none edited until it
passed.**

| record | what really went red |
| --- | --- |
| the count of sign-ins that go round the helper (turned round) | 1 test, the census |
| a delete opens the folder it names (re-pointed) | 4 named, all 4 |
| saving a copy does not change which folder is open (re-pointed) | 2 named, both |
| a second SELECT of an already-open folder still asks (re-pointed) | 1 named |
| one sign-in serves every piece of work (new) | 2 tests, reuse and budget |
| a dropped connection is tried once more and not twice (new) | 1 test |

The census record was turned round rather than moved. Its break was "convert
the mark-read site", which this plan does for real, so the text it named left
the file and the commit gate refused everything until it was fixed. Moving it
to one of the other eleven would have needed doing again a few commits later,
because once they have all gone through the helper there is nothing left to
convert. The break now puts the old sign-in back, which discriminates at eleven
and still discriminates at nought.

**What stayed green under the new records is the finding.** Breaking the reuse
leaves both closing tests green, because the break only stops the held session
being handed back and still writes one down. So that record is about reuse and
says nothing about the closing, which has no record and is defended by its two
tests alone. And breaking the retry into two retries reddens only the
drop-always test: a second retry that is never reached changes nothing about a
server that answers the first one, which is why that fixture exists.

**The break for the retry is one more attempt rather than a loop, deliberately.**
A loop against a server that drops every connection never ends, so the run
would hang rather than report, and a guard that hangs says nothing.

**One test outside the guard registry had to move with the code.** `sent_copy`
counts the routes to a mail server and expected one in `mail_session.rs`. The
route moved into the controller, which is what remembers the account. It counts
both halves now, one call asking to be signed in and one call reaching a
server, because either alone is satisfied by the other moving.

**No guard re-measurement was owed for changed test counts.** The count check
never printed a `--remeasure` command: every test added here went into
`src/application/mail_session.rs`, which no record named until the last commit,
and the two records added there carry the finished count of 14.

## Premises that were wrong

### 1. Task 2 cannot be committed the way the plan describes

The plan asks for the census to be committed red at nought "while sites remain"
and then for the sites to be converted "in small groups, one commit per group,
with the census number quoted in each commit message". Those two cannot both
happen. The census asserts equality against one number, so the first group
commit after a red at nought leaves it red, and the gate refuses every commit
between the red and the last group.

Taken as: one red at nought, then one green converting all eleven. The plan's
reason for wanting groups is honoured differently, by converting each site
individually against its own failure paths rather than by pattern, with the
per-site table above as the record.

### 2. The budget test cannot live where the plan puts it

The plan asks for it in `tests/one_sign_in_per_piece_of_work.rs`, "beside the
census, since it is the same subject", counting connections at a loopback
server. `common::answering` is `#[cfg(test)]`, so an integration test links a
library built without it and cannot reach the loopback server at all. The two
tests share a subject and cannot share a home.

Moving the fixture is the alternative and it is a bigger change than the budget
test warrants: it means putting a loopback mail server into the library's public
surface behind a second feature, the way `what_ships` was moved. The budget test
lives in `mail_session.rs`, where the server is. The two are different kinds of
test anyway: the census reads source, this counts sockets.

### 3. "Reconnect once behind require_imap" needed the retry somewhere else

`require_imap` fires before a command, and a held session that has died is
`Some`: the failure appears when the command runs. So the retry could not sit
inside `require_imap` itself. It sits in a macro that wraps every command,
which is the same claim the plan makes, that no caller has to remember, reached
a step lower down.

### 4. The plan's line numbers were stale again, and so was one of its counts

Five of the twelve sites had moved. This is the fourth plan in this phase to
find that, and everything here was located against the tree.

The plan says sixty-one guard records name `wx_app.rs` and twenty-seven name
`mail_controller.rs`. Those are counts of mentions. What the count check
fingerprints is different: 34 and 23. It did not matter here because nothing
changed the test count of either file, but it is the third correction of the
same figure in this phase.

### 5. Error mapping was wrong in a way the plan assumed was right

The plan says to map at the boundary and not match on message text, which
implies the boundary already classifies. It did not: `protocol_error` mapped
every async-imap failure to `Error::Protocol`, including `Io` and
`ConnectionLost`, so a lost connection reached the caller looking like a server
that had said no. That is the one thing a retry must not fire on and the one
thing it must, so it had to be fixed before the retry could be written.

## What this cannot see

**Nothing here has met a real mail server**, and this plan is mostly about how
a real server behaves. Four things are recorded in the ledger rather than
ticked:

- whether a provider accepts a fresh sign-in straight after dropping a
  connection, or slows it down or refuses it (entry 64)
- what a provider does with a session held open and idle for minutes, which is
  the whole premise of holding one (entry 65)
- whether the refusal sentence is heard once rather than once per failed
  request, which needs a screen reader and a mailbox where every command meets
  a dead connection (entry 66)
- whether two connections per account is welcome, and how a provider counts
  them when several accounts share one (entry 67)

Two more are deviations: `folder_counts` still swallows a dropped connection
(entry 68), and the check-mail port refusal stopped naming the value it could
not read (entry 69).

**The census still reads source.** It says nothing new dials for itself; it
cannot say the holder holds what it hands out. That is what the first new guard
record is for.

**Three of `a_session_at`'s callers still sign in per piece of work** and are
not part of the census: filing a draft, filing a sent copy, and deleting a
message at the server. Each opens a connection of its own and closes it, which
is wasteful rather than wrong, and none of them can close the held session
because they hold their own. Filing a draft runs once a minute while somebody
writes, so it is the one worth converting next.

## Owed

**The phase-end guard sweep**, per `CLAUDE.md`. Six records were touched here
and all six measured by hand; the sweep is the one that asks about the other
575.

**`scripts/guards.sh --touched-by 306c800`** after the merge. This branch
changes `wx_app.rs`, `mail_controller.rs` and `imap.rs`, which 34, 23 and 34
records fingerprint, so it is an overnight job and must not block the merge.

**Nothing was merged and nothing was pushed.**

## Requirements and criteria

SCALE-02 is closed structurally. Criterion 2 of the phase, that opening several
messages in a row reuses one authenticated session and a dropped connection
reconnects once and says so if the retry also fails, is met in the code and
proved by a server that counted. It is not closed in the field, and the four
ledger entries above are why.

## Self-Check

- All seven commits are in `git log`: b48c713, 91bff41, 15a1afb, b85e982,
  6a9af87, e03a9cb, 4926805.
- `src/application/mail_session.rs` holds 14 test functions, all passing.
- `tests/one_sign_in_per_piece_of_work.rs` holds 7, all passing, with the
  census answering nought.
- `guards/guards.toml` holds 581 records, two of them new here and four
  re-pointed, and the sweep header's two numbers add up to it.
- `scripts/check.sh all` passed on the finished branch: 6,135 library tests
  passed, 0 failed, 1 ignored, and the release build is clean.
- `grep -n '^version' Cargo.toml` reads `version = "0.51.0"`, up from 0.49.1.
- `docs/changelog.md` carries both entries under `[Unreleased]`, with the
  known limitations named.
- The working tree is clean apart from this document and the ledger committed
  with it.

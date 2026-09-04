---
phase: 03-mail-at-scale-on-the-wire
plan: 01
executed: 2026-09-03
status: complete
tasks: 2
requirements: [SCALE-01]
commits:
  - 99694f8 test(03-01) failing tests for a listing that covers part of a folder
  - 54b2b32 A listing of part of a folder deletes nothing
  - b3d6cb2 test(03-01) failing tests for a folder discard nobody is told the size of
  - 71fd243 A renumbered folder says what it discarded
merged: 6bca1f8, 8ee4e84
---

# Plan 03-01: A page of a mailbox cannot reach the code that forgets

**One-liner:** The rule that only a whole-mailbox listing may drive deletion is
a type rather than a doc comment, and a folder the server renumbers now says
out loud how many messages it discarded.

## What works

**Task 1.** `ServerListing` carries what the server answered and how much of the
folder that answer covers, because the two are never safely apart.
`uids_to_forget` matches on the arm rather than reaching past it for the uids,
so a page or an empty answer returns nothing to forget instead of everything
held. The empty page was the case that would take a whole mailbox: nothing
named, everything held, so everything held reads as gone. `sync_folder` makes
the whole-mailbox claim on the one line that knows it is true, so plan `03-07`
has to rewrite that line to narrow the question, and deletion stops the moment
it does.

The fetch and flag paths still take the bare uids, because asking about fewer
messages than exist is a smaller sync rather than a deletion. Only the forget
path is held to the whole folder, so only the forget path is given the type.

**Task 2.** A folder the server renumbers announces it, naming the folder and
how many messages went, through a new `UIUpdate::FolderWasRenumbered` on the
topic `"renumbered"` at normal priority. The count comes from
`forget_folder_messages`'s return value, which `sync_folder` used to discard,
worded through the same `how_many` helper as the refusal two lines below it.
The existing `tracing::info!` stays: said and logged, not said instead of
logged.

## Verification

Both tasks red first, both reds accepted by `scripts/red-commit.sh`, and
`scripts/check.sh all` passed before each merge. The library was 6,072 tests at
the first merge and 6,075 at the second, none failing.

**The absent-UIDVALIDITY arm had no defence before this.** Widening the
`matches!` so an absent server value counts as a change reddens exactly one test
in the whole library: the one written for it here. That is the new guard
record's finding rather than an aside, and it is the arm that would delete mail
on every sync against a server answering no UIDVALIDITY at all.

Guard records re-measured: 3 for task 1, 35 for task 2, all agreeing with what
they name. Thirteen of the 35 reported "the break did not build" on a first
pass, with cargo exiting `0xC0000142` and saying nothing; all thirteen built and
agreed when asked again, which is the process-start failure `CLAUDE.md` already
documents.

## Three premises that were wrong

**1. "Nothing reads the renumbered fact."** The one that changed the work.
`FolderSync.renumbered` predates this phase, and `what_the_folder_sync_did`
already turned it into the clause "read again after the server renumbered it",
sent as `StatusUpdated`. The fact did reach somebody: as a clause with no
number, at low priority, on `"status"`, where the next line replaces it. What
was missing was the count and the channel, not the fact.

Built on the premise as stated, this would have shipped the same event twice a
moment apart with nobody having compared the two. Built on the corrected one,
the status clause is the seen half and the announcement is the heard half with
the number, and the doubling is a stated cost rather than an accident. The
research document is corrected.

**2. "Do not run the re-measure on the critical path."** Not possible, and this
instruction was wrong rather than merely awkward. It is right about the sweep
and wrong about the count check: the count check runs inside the commit gate,
the gate refuses a commit while it is red, and its only sanctioned remedy is
that run. There is no "later" between the two.

**3. `files_modified` omitted `src/presentation/ui_types.rs`**, where `UIUpdate`
lives. Adding a variant has to touch it.

## Two things worth carrying forward

**A source-reading test can be defeated by the comment that explains the code.**
The wiring assertion checked the whole match arm for `"status"`, and so read the
comment saying why the topic is *not* `"status"`, failing correct code with a
message stating the opposite of the truth. A comment justifying a choice always
names the alternative it rejected, so the better the comment the more reliably
it defeats the check. `the_announcement_in` now cuts the arm down to the call's
arguments, with a companion holding it to telling a comment about a topic from a
topic.

**A commit whose gate outlasts a ten-minute cap has to be detached.** The
version bump to 0.47.0 makes `which-checks.sh` answer `all` for any commit
touching `Cargo.toml`, so both the task 2 commit and its merge paid the release
build and ran about seventeen minutes. The first attempt at each was killed with
the hook most of the way through and nothing committed. No stale lock was left
either time.

## Not closed

**Criterion 1's announcement half is not ticked**, and is recorded as ledger
entry 52 rather than claimed. Three questions only a screen reader settles:
whether the sentence is spoken at all mid-sync, whether a topic of its own is
right against `"status"` given the argument for splitting is reasoning about how
the queue coalesces rather than an observation of it, and whether a
normal-priority announcement during a sync cuts across something the person was
reading. Compounded by no real server having ever renumbered a folder for this
program, so the whole path has run only against a scripted one. The changelog
says so under Known limitation.

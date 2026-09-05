---
phase: 03-mail-at-scale-on-the-wire
plan: 07
executed: 2026-09-05
status: complete
tasks: 3
requirements: [SCALE-01, SCALE-03]
subsystem: application, data, presentation, service
tags: [resume, condstore, qresync, seam, deletion-detection, whole-folder, snippet, scale-01, scale-03]
commits:
  - 990d8ed test(03-07) failing tests for a folder that should resume rather than re-list
  - 8365f4a A folder that was synced before asks what changed, not what it holds
  - d2b8640 test(03-07) failing tests for a seam nothing asks and a bound nothing keeps
  - 11aa51a A message deleted on a phone still goes, and the sync asks how rather than doing it
  - 0b784fe test(03-07) failing tests for a column that lies and a request nobody wrote
  - d288f9f Ask for a whole folder once, and stop telling people their mail is empty
  - e8f7b30 test(03-07) failing tests for the two commands the resume answers with nothing
  - "the green half of the above, and this document"
merged: not merged, and not pushed
key-files:
  created:
    - src/application/finding_what_was_deleted.rs
    - src/application/asking_for_a_whole_folder.rs
    - tests/a_whole_folder_moves_both_bounds.rs
  modified:
    - src/application/mail_sync.rs
    - src/application/mail_controller.rs
    - src/application/allowed.rs
    - src/application/conversations.rs
    - src/service/protocols/imap.rs
    - src/service/protocols/imap/abilities.rs
    - src/data/message_cache/folders.rs
    - src/data/message_cache/bodies.rs
    - src/data/message_cache/messages.rs
    - src/data/message_cache/mod.rs
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - src/presentation/message_rows.rs
    - src/presentation/read_aloud.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/KEYBOARD_SHORTCUTS.md
    - Cargo.toml
requires:
  - "03-01, whose ServerListing is what makes the narrowing safe"
  - "03-06, whose held session is what sync_folder is now handed"
provides:
  - "a resume: a folder synced before asks UID SEARCH UID n:* instead of UID SEARCH ALL"
  - "a named seam over how a folder learns what was deleted, with the uid comparison built and VANISHED declared"
  - "Abilities::qresync, detected the way its neighbours are"
  - "a bounded full comparison, every six hours per folder, stored per folder"
  - "a whole-folder request that carries on by itself, with progress on a topic of its own"
  - "a snippet column that tells not-fetched from no-text"
affects:
  - "anything that starts a sync: sync_folder now asks what the caller wants it for, and the answer decides whether it resumes"
  - "anything reading MessageItem.snippet or ConversationItem.snippet: both are Option<String> now, and None means nobody fetched it"
decisions:
  - "The resume gate reads the mailbox's own HIGHESTMODSEQ rather than the connection's CONDSTORE capability. It is the number the resume actually uses, and a server can advertise CONDSTORE and report none for a particular mailbox."
  - "the_way_this_server_offers reads the real qresync answer and still hands back the comparison, because the other member is not built. Without that check every Fastmail and Dovecot account would select an implementor that refuses and every sync would fail."
  - "listing_contradicts_the_count takes the listing rather than its length. A resumed sync is answered with nothing on every quiet folder, and the old shape read that as a server contradicting its own count."
  - "FolderSync.total_on_server is the STATUS count rather than the length of the listing, because on a resume the listing is what arrived rather than what the folder holds."
  - "A sync is told what the caller wants it for. Bringing a folder up to date can resume; reaching further back cannot, because the mail it wants is below the highest uid held."
  - "The whole-folder census is an integration target coupled to wx_app.rs rather than a test inside it. Thirty-four records fingerprint that file's test count, at a build and a full library run each."
metrics:
  duration: about 8 hours
  files: 21
  commits: 8
actuals:
  tokens: 63000
  tasks: 3
  commits: 8
---

# Plan 03-07: Ask the server what changed, and ask for a folder once

**One-liner:** A folder synced before asks for the uids above the highest one
held instead of every uid it has, a message deleted elsewhere is still found by
a comparison that runs every six hours behind a named seam with VANISHED
declared beside it, and somebody can ask for a whole folder once and hear about
it a handful of times.

## What works

### The resume

`sync_folder` asks `UID SEARCH UID {highest}:*` where it used to ask
`UID SEARCH ALL`. On a folder of forty thousand messages that is the difference
between a handful of numbers and forty thousand.

Four facts have to line up: a stored UIDVALIDITY, the server reporting the same
one, a stored HIGHESTMODSEQ, and the server reporting one now. Anything else is
read out in full, which is every first sync, every renumbered folder, and every
Gmail account. A fifth thing overrules all four: a folder due a full comparison
is listed in full whatever is stored.

The answer is carried as `ServerListing::PartOfIt`, which is the claim plan
03-01 turned into a type for exactly this change, so the forget path hands back
nothing for it. **No whole-mailbox witness is constructed on any resume path.**
Checked by reading `sync_folder`: `ServerListing::TheWholeMailbox` appears on
one line, inside the `WhatToAskFor::EveryUid` arm of the match that asks the
question, and the other arm constructs `PartOfIt`.

### The seam

`src/application/finding_what_was_deleted.rs` names two ways a folder can learn
what the server no longer has:

```rust
pub enum HowAFolderLearnsWhatWentAway {
    ByComparingUids,
    ByAskingWhatVanished,
}
```

`ByComparingUids` is built. `ByAskingWhatVanished` is declared and not built,
and its own documentation says what building it costs:

> One answer naming what went, with no listing of the folder at all, on a
> server that offers QRESYNC. **Declared and not built.** It needs a select
> response of its own, because a raw `SELECT x (QRESYNC (...))` does not come
> back through the parsing `async-imap`'s own `select` does, and that is the
> work adding it means.

Asking for it returns a `common::Error` rather than an empty answer, in words
somebody can read, because an empty answer is a folder that finds no deletions
and looks like it worked.

`sync_folder` asks the chosen way and does not name the comparison. A census
walking the shipping half of `src/application` finds exactly one place that
names `uids_to_forget`, and asserts it is the seam's own implementor.

### The probe reads a real answer, and still chooses the comparison

`Abilities` has a `qresync` field, detected beside its neighbours:

```rust
} else if name.eq_ignore_ascii_case("QRESYNC") {
    abilities.qresync = true;
```

`the_way_this_server_offers` reads it and hands back `ByComparingUids` anyway,
because a way that is not built is not a way this program can take. **That
check is load-bearing rather than tidiness, and the plan's premise about it was
wrong.** The plan says `qresync` "will be false against every server this
project has met", so a probe keyed on it would answer the comparison
everywhere. It will not: `test_a_full_featured_server_is_read_in_full` models
Fastmail and current Dovecot and its capability list has said QRESYNC since it
was written. A probe wired the way the plan describes would have selected the
unbuilt member for every account on those servers, and every sync would have
failed with a refusal.

### The bound

Six hours per folder, with both halves of the reason beside the number, and a
folder nothing has ever compared is compared on its next sync. The time is
stored per folder in an additive column:

```rust
self.ensure_column_exists("folders", "last_fully_compared", "TEXT")?;
```

Only a sync that really listed the whole folder writes it down. A narrow
question has compared nothing, and dating it as though it had would put the next
comparison six hours away for a comparison that never happened.

### The whole-folder request

Folder then Download This Whole Folder. The loop lives in
`application::asking_for_a_whole_folder` where it can be run without a window,
and it is bounded three ways: the folder is complete, a chunk brought nothing
new, or a chunk failed. Progress goes out on a topic of its own with the
constant beside the loop, and the last thing said is a count rather than a
progress line that happens to be last.

Both bounds move together. The request hands each sync `INITIAL_FETCH_LIMIT`;
the update it sends once per chunk moves `FOLDER_LIST_PAGE_SIZE`. It cannot
move both itself, because it runs on a worker thread and the view's limit lives
on the interface thread.

Marked experimental on the label and in the description, and the description
says which kind: not that it has never run, but that a provider is entitled to
refuse or slow down a request that asks for a large folder page after page, and
that there is no way to stop it once it starts.

### The snippet column stops lying

A message whose text has never been fetched and a message that genuinely has no
text both left the column blank, and a blank column reads as the second. That
was wrong for most of a large mailbox. Both wordings, quoted:

- not fetched: `"Message text not downloaded"`
- fetched and empty: `"No message text"`

Three states in one type, `Option<String>` on both row types, written where the
body is saved and answered by one function for both views.

## The defect this found, and it is the largest thing in this plan

**The resume broke Get Older Messages, and it broke the whole-folder request
built beside it.** Found by asking whether the new command could actually work
rather than by anything going red.

A resumed sync asks for the uids above the highest one held. That is exactly
right for bringing a folder up to date, because everything that arrived is
numbered above it. The mail Get Older Messages exists to fetch is *below* it, so
`uids_to_fetch` was choosing what to bring down from a list that could not
contain it, and brought down nothing.

From outside: Shift+F9 doing nothing on any folder that can resume, and Download
This Whole Folder stopping after one chunk saying the mail server stopped
sending. The first chunk worked because a folder nothing has compared is
compared, which lists it in full; every chunk after it resumed.

Neither is hypothetical and neither is old: a regression on a shipped command
and a new command that never worked, in the same fault, three commits and one
commit after their causes.

The fix is that a sync is told what it is being asked for. Bringing a folder up
to date can resume; reaching further back cannot. The three places that start a
sync each name their answer at the call site.

**What this says about the plan.** Task 1 and task 3 are written as independent
tasks, and they are not: task 1 changes the question `sync_folder` asks and task
3 depends on the old answer. Nothing in the plan's `key_links` names that, and
the plan's own verification list would have passed against a build where the
command it asks for does nothing.

## Verification

Every commit went through `scripts/check.sh` by way of the commit hook. Nothing
used `--no-verify`. Two commits changed `Cargo.toml`, so `which-checks.sh`
answered `all` for both and each ran the whole suite and the release build;
both were run detached, because that gate outruns a ten-minute foreground cap.

**Four reds, all accepted by `scripts/red-commit.sh`.**

| commit | named | what really failed |
| --- | --- | --- |
| 990d8ed | 4 | 2 CONDSTORE conditions missing, 1 existing deletion test, the count check |
| d2b8640 | 6 | the probe, the seam, the bound, the stored time, the count check |
| 0b784fe | 10 | 3 snippet, 1 existing status-line guard, 4 census, the count check |
| e8f7b30 | 3 | the two commands the resume answers with nothing, the count check |

**Which tests passed at their own red, said plainly.** Three.

`test_a_folder_that_was_synced_before_is_not_listed_again` is the requirement's
own assertion and it passed at its red. Under `-D warnings` there is no red for
this: a stub answering "list everything" never constructs
`WhatToAskFor::TheUidsAbove`, so the variant is dead code and the build is
refused. The red was written as half a decision instead, and what stands in for
the missing red is the guard record, measured in both directions.

`test_a_folder_compared_recently_is_not_listed_and_forgets_nothing` passed at
its red because the resume that landed one commit earlier already forgets
nothing. It is named in the resume record now.

The nine tests in `finding_what_was_deleted.rs` passed on arrival. A new pure
module has no before, and saying so is more honest than dressing the first write
of it as a red.

**Five guard records, four of them new, every one measured by hand against the
whole library.**

| record | break | what really went red |
| --- | --- | --- |
| the resume (new) | the decision never resumes | 3 |
| the bound (new) | the interval widened to 114 years | 3 |
| the seam (new) | the sync calls the comparison directly | 1 |
| both bounds (new) | the view bound taken out of the arm | 1 of 7 in that target |
| counted then not listed (re-pointed) | the check always answers no | 2 |

**Two of those measurements found something, and both are the point of taking
the break by hand.**

The bound's first measurement reddened one test, and that one was the range
assertion beside the constant rather than anything about the rule. Both
behaviour tests set their fixture to the interval plus a minute, so the offset
moved with the break: widen the interval and the fixture is a hundred and
fourteen years stale and still due. **A bound that can be widened to never
without a test failing is exactly the defect the plan asks about, and it was in
the tests written for it.** Both fixtures say twenty-four hours now, written
out, and the break reddens three.

The resume record fell behind inside this plan, one commit after it was written:
the bound added a test that resumes a folder, so the break reddens three rather
than two. Caught by the re-measure the count check asked for rather than by
anybody thinking to look.

**Both directions were taken for the two records the plan asks that of.** The
resume: never resuming reddens two, always resuming reddens three, one of them
the deletion detection saying it has stopped working. The bound: widened to
never reddens three, narrowed to nought reddens four.

**Guard re-measurements: twenty-four record measurements across five runs**, all
agreeing except the one named above.

## Premises that were wrong

### 1. The probe would answer the comparison for every server

The largest of them, and it would have shipped a refusal on every Fastmail and
Dovecot account. Premise correction 1 says `qresync` "will be false against
every server this project has met, which is what makes the choosing always
answer the first member". The capability list this project models a modern
server on has advertised QRESYNC since it was written. The choosing has to
refuse to hand back a member that is not built, and that check is the reason
the seam is safe rather than a nicety.

### 2. Task 1 and task 3 are independent

They are not, and the plan has no link that says so. See the defect above.

### 3. `sync_folder` cannot ask the connection's capability list

The plan's action for task 1 says to gate the resume on "the server offers
CONDSTORE". The `Mailbox` trait had no capability method, so the resume reads
`MailboxStatus.highest_modseq` instead, which is the number it actually resumes
from and a stronger fact: a server can advertise CONDSTORE and report no
HIGHESTMODSEQ for a particular mailbox. Task 2 added the capability method for
the seam's probe, so both facts are available now and each is read where it
belongs.

### 4. The snippet distinction cannot reuse the coverage count

The plan says to reuse the question `message_bodies` answers. Two things stop
it. Plan 03-03 built `a_listing_reads_no_message_text`, a guard that runs every
listing query against a database with the body tables removed, so a listing that
joined `message_bodies` would fail it. And the coverage count answers "is the
text here now", which is a different question from "was it ever fetched": bodies
are evicted under a budget and the snippet outlives them. The distinction is in
the snippet column itself now, null against empty.

### 5. `listing_contradicts_the_count` had to change, and the plan does not say so

A resumed sync is answered with nothing on every folder nothing has arrived in,
which is most folders on most syncs. Read through the old shape that is a server
contradicting its own count, and the sync is refused. The check takes the
listing now, so only a listing claiming the whole mailbox can contradict a
count of it.

### 6. `FolderSync.total_on_server` had to change too

It was the length of the listing. On a resume that is what arrived since the
last sync, so a forty thousand message inbox would have reported holding three,
taking Get Older Messages off the menu and saying "3 of 3" to somebody looking
at a full mailbox. It is the STATUS count now.

### 7. The plan's guard-record counts are counts of mentions again

The plan says five records name `mail_sync.rs`, sixty-one name `wx_app.rs`, and
one names `message_rows.rs`. What the count check fingerprints is different: six,
thirty-four and one. This is the fourth plan in this phase to find the same
figure quoted the same way.

## Deviations

**The changelog and the version bump for tasks 1 and 2 are in task 2's commit,
not task 1's.** The plan asks for that and it is also the honest shape: task 1
alone removes deletion detection for any folder that can resume, and there is no
state there worth describing to anybody. Task 1's commit says so.

**`uids_to_forget` and `ServerListing` stayed in `mail_sync.rs` and the seam
calls them.** Moving them would rename the tests four guard records name and
move the anchor of a fifth, for no change to what the seam buys: `sync_folder`
names the seam and nothing else, which is the property the record measures.

**The whole-folder census is an integration target rather than tests in
`wx_app.rs`.** Thirty-four records fingerprint that file's test count, at one
build and one full library run each, so a test added there is hours at the next
commit. The handler needs a window to reach either way, so it is a source read
wherever it lives. A `guards.toml` coupling makes it run on the commits that
could change the answer, which is what 03-02 did for the sign-in census.

**No new tests were added to `wx_app.rs`, and that is a deliberate trade with a
cost.** What is therefore not guarded from inside that file: that the progress
arm uses the constant rather than a literal topic, and that the menu item
carries the experimental warning. Both are asserted by the new target instead,
and the record couples it so it runs.

**`Down&load` rather than `&Download`.** `test_no_two_items_on_one_menu_claim_the_same_letter`
refused the first spelling: `Delete Fol&der` already claims d on that menu.

**The census asked the wrong file position first.** It read the handler for
`FOLDER_LIST_PAGE_SIZE`, which cannot be there: the request runs on a worker
thread and the view's limit lives on the interface thread. It asks the handler
and the arm now, which is the shape of the answer rather than a weakening of the
question.

**One red failed on an assertion it should not have made.** The test for the
older mail asserted the stored uids in sorted order; `stored_uids` has no
`ORDER BY` and hands back insertion order, and the older half arrives second. It
was read before it was adjusted: the fix had worked and the assertion was about
the query rather than about the fix.

## What this cannot see, and what it costs

**Nothing here has met a real mail server**, and most of this plan is about how
one behaves. Six ledger entries rather than ticks, entries 70 to 75, five of
them `unrun-verify` and one a deviation:

- whether any provider grants CONDSTORE, which is what the resume needs
- whether a hand-built `SELECT (QRESYNC ...)` parses back, which is the whole
  cost of the second implementor
- whether a provider tolerates a whole-folder request
- three things about the announcements that only a screen reader settles,
  including whether the progress belongs on its own topic at all, which is the
  one open question in this plan
- whether a column of "Message text not downloaded" reads well when somebody
  crosses it

**The snippet distinction is right going forward and not backwards.** A message
fetched before this that held no text was stored as null and reads as one nobody
has fetched. There is no backfill, because the fact is not in anything the
database still holds. Ledger, as a deviation.

**What the whole-folder request costs in the conversation view, and it is
arithmetic rather than a measurement.** Each chunk re-reads the open folder, and
in conversation view that runs `conversations_query`, which has no `LIMIT` and
groups the whole account. Plan 03-04 measured that query at **1.2 s on 200,000
rows in 10,000 conversations, release build, warm, on an account with a folder
holding all mail**, and 0.86 s without one. A whole-folder request over 40,000
messages is 80 chunks, so on such an account it is 80 of those reads, about a
minute and a half of query time spread across the run. That figure is 03-04's
measurement with its conditions, multiplied by an exact chunk count; it is not a
new measurement, and it is not the whole cost of the run, which is dominated by
the network. `conversations_query`'s missing `LIMIT` was SCALE-03's subject
before this plan and is untouched by it.

**Guard re-measurement was run throughout and the phase-end sweep is still
owed.** Five records were touched here; the sweep is the one that asks about the
other 580.

**`scripts/guards.sh --touched-by 8d73579` after the merge.** This branch changes
`wx_app.rs` and `mail_sync.rs`, which 34 and 6 records fingerprint.

**Nothing was merged and nothing was pushed.**

## Requirements and criteria

**Criterion 1 is closed structurally.** Reopening a folder synced before resumes
from the stored sync state instead of re-listing every uid, and deletions are
still found on a stated schedule. It is not closed in the field: Gmail offers no
CONDSTORE, so a Gmail account sees no change, and whether any provider grants it
is unknown.

**Criterion 3 is closed structurally.** A whole folder can be asked for, the
list is usable from the first chunk, and progress speaks as one superseding
topic. Whether it is heard as intended is a ledger entry.

**QRESYNC is settled.** The uid comparison ships, VANISHED is a declared and
unbuilt second answer, and a capability probe chooses. Adding QRESYNC is one
implementor plus one line in `is_built`, and it touches nothing that decides
policy.

## Self-Check: PASSED

- All eight commits are in `git log`: 990d8ed, 8365f4a, d2b8640, 11aa51a,
  0b784fe, d288f9f, e8f7b30, bc56caa.
- The three new files exist: `src/application/finding_what_was_deleted.rs` (9
  tests), `src/application/asking_for_a_whole_folder.rs` (7 tests),
  `tests/a_whole_folder_moves_both_bounds.rs` (7 tests). All passing.
- `src/application/mail_sync.rs` holds 134 test functions, up from 114.
- The library is **6,175 passed, 0 failed, 1 ignored**.
- `guards/guards.toml` holds 585 records, four of them new here and one
  re-pointed, and the sweep header's two numbers, 192 and 393, add up to it.
- `grep -n '^version' Cargo.toml` reads `version = "0.53.0"`, up from 0.51.0 in
  two steps, one per behaviour change.
- `docs/changelog.md` carries three entries under `[Unreleased]` from this plan,
  each naming its limits, including that a Gmail account sees no change from the
  resume.
- `.planning/WINDOWS.md` holds entries 70 to 75, all open.
- `docs/KEYBOARD_SHORTCUTS.md` names the new menu item.
- The working tree is clean apart from this document.

---
phase: 03-mail-at-scale-on-the-wire
plan: 09
executed: 2026-09-05
status: complete
tasks: 4
requirements: [SCALE-06]
subsystem: application, data, presentation
tags: [conflict-choice, contacts, caldav, flag-changes, guardrail-7, accessibility, scale-06]
commits:
  - f14f835 test(03-09) failing tests for a contact held rather than written over
  - ce3e89b A contact changed in two places is still both, waiting for somebody to choose
  - fd19ba9 test(03-09) failing tests for a choice somebody can hear and make
  - a2a046d Both copies, told which is which, chosen without a mouse
  - ce86e12 test(03-09) failing tests for a calendar that asks rather than decides
  - 171223d The calendar asks the same question, and mail's defect is named as itself
  - f81e597 test(03-09) failing tests for a flag change kept rather than undone
  - 8b05528 A flag change made with no server keeps, and goes when there is one
  - "one more: the summary, the corrected guard record and its re-measured counts"
merged: not merged, and not pushed
key-files:
  created:
    - src/application/conflict_choice.rs
    - src/application/calendar_conflict.rs
    - src/application/flag_changes_waiting.rs
    - src/data/message_cache/held_conflicts.rs
    - src/data/message_cache/waiting_flag_changes.rs
    - src/presentation/wx_conflict_choice.rs
    - tests/the_conflict_choice_can_be_heard.rs
    - tests/nothing_sends_a_flag_change_unasked.rs
  modified:
    - src/application/contacts_sync.rs
    - src/application/caldav_sync.rs
    - src/application/calendar.rs
    - src/application/mail_session.rs
    - src/application/deleting_at_the_server.rs
    - src/application/sent_copy.rs
    - src/data/message_cache/mod.rs
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
    - .planning/REQUIREMENTS.md
    - .planning/WINDOWS.md
requires:
  - "whose_copy_wins, the four-armed decision contacts already had, unchanged"
  - "etag and If-Match in caldav_sync, already there and already compared"
  - "common::Error's Network and Protocol variants, told apart by plan 03-06"
  - "the announcement queue's topic superseding, used by 03-07 and 03-08"
provides:
  - "conflict_choice: both copies of one thing, the fields that differ, the labels, the sentences and what a press means, with no window and no database"
  - "held_conflicts: the hold, kept across restarts, and the settling that follows a choice"
  - "calendar_conflict: the calendar's fields and its raising, sharing everything else"
  - "flag_changes_waiting: why a push failed, drawn once on Error's variants, and the two sentences"
  - "waiting_flag_changes: a table whose primary key collapses two changes to one flag"
  - "wx_conflict_choice: the window that asks, keyboard-only, every control named where a screen reader reads"
  - "a census of every place a waiting flag change reaches a server"
affects:
  - "any later plan touching the contacts or calendar merge: the losing arm now holds rather than writes"
  - "the push retry in contacts_sync, which is the one both-changed state still resolving without asking"
decisions:
  - "The hold gets its own table rather than a column on contacts or calendar_events. The provider's copy is a whole second version of a row, and a row cannot hold two of itself."
  - "Waiting flag changes get their own table rather than outbox_queue. Confirmed against the schema: to_addr, subject and body are all NOT NULL. The deciding half is that an outbox row is a thing to send once and a waiting flag change is a state to reconcile."
  - "Settling a hold writes no provider copy. Both endings leave the next ordinary sync to do it through the decision that already exists, so there is no second merge to keep in step and the copy that arrives is current rather than a snapshot."
  - "Settling writes the provider's version marker down whichever copy is chosen. Without it the next sync sees the same disagreement and asks again, and a question that comes back every sync is not one anybody has answered."
  - "Only work typed here is held. Once an address book's copy has replaced an edit, what is waiting is that address book's own words on their way to the other one, and asking somebody to choose between two copies neither of which is theirs is a question with no right answer."
  - "the_address_book_moved is renamed the_marker_moved and shared with the calendar. An etag and a contact version marker are the same fact and the same comparison."
  - "The choosing window opens with Decide later focused. Nothing is the default answer, so Enter pressed by reflex does not lose a copy."
  - "A menu item rather than a raised window. The sync's sentence names it, and a modal that opens itself takes focus from somebody mid-task."
  - "Authentication counts as the server never having been asked. A token that expired is fixed by signing in again, and putting a star back over that loses work to something that clears on its own."
  - "the_session_at and a_session_at hand back common::Error rather than String. The sign-in failure is exactly the no-network case, and it was arriving as text."
  - "Waiting flag changes go from where a mail check has already signed in. Nothing watches the network."
  - "Tests about caldav_sync, wx_app and the choosing window live outside those files, traded against guard re-measurement time. Every expensive file's test count is unchanged, so no record was flagged."
metrics:
  duration: about 9 hours
  files: 22
  commits: 8
actuals:
  tokens: 96000
  tasks: 4
  commits: 8
---

# Plan 03-09: Both copies kept, and mail's real defect fixed

**One-liner:** A contact or calendar item changed both here and at the provider
is kept whole and asked about instead of one copy being written over, and a mail
flag change made with no server is kept and sent later instead of being quietly
undone.

## What works

### A contact changed in two places is still both

The address book used to win. The edit went, and the sync summary said "1 of
your change replaced by the address book" afterwards. Being told is not being
asked, and an edit that disappeared with a sentence about it in a summary
nobody heard is an edit that disappeared.

`whose_copy_wins` is untouched, and that was checked rather than claimed: it
still returns the same four arms from the same two facts, comparing version
markers rather than clocks. What changed is what happens in one of them.

Three places had to agree for a hold to mean anything, and only the first is
what the plan described:

1. The arm keeps both copies instead of writing one over the other.
2. **The push offers nothing for a contact waiting on a choice.** Without this
   the change is still owed, so the very next push sends this computer's copy
   over the one nobody has chosen to give up.
3. **The read leaves a held contact exactly as it is.** Without this the sync
   after the one that raised the question answers it, and holding is a slower
   overwrite.

### The window that asks

Both copies are shown as a labelled pair. Each list is headed and named by
which copy it is, and both strings are built in the application layer and
tested there:

> What is on this computer

> What your address book has

The sentence on opening, for a pair differing in one field:

> You changed this contact here and it changed in your address book as well.
> 1 field is different: Telephone. Choose which copy to keep.

Native controls in the tab order, so everything is reachable by keyboard and
each already meets the target size. No new shortcut, so
`docs/KEYBOARD_SHORTCUTS.md` is unchanged: this is pressed once per
disagreement, and a key nobody presses twice is a key in the way of one
somebody presses daily.

**Nothing is the default answer.** The focus opens on Decide later. Escape, the
close box and that button are one answer, and it leaves the hold exactly where
it was.

Choosing writes no provider copy. Keeping what is here leaves the change
waiting and brings the marker up to date, so the next sync offers it carrying a
marker the address book will take. Taking theirs stops the change waiting, so
the next sync answers `TakeTheAddressBooks` and folds their whole copy in
through the merge that already exists.

### The calendar asks the same question

Through the same module, the same window and the same words. Only the fields
worth reading out and the word for the other copy differ, and both are
parameters. The calendar's labels:

> What is on this computer

> What your calendar has

The sentence both syncs say is built in one place, so the two cannot drift:

> 1 contact changed here and in your address book as well. Use Choose Which
> Copy to Keep, on the Tools menu, to say which copy to keep; nothing is sent
> until you do

### A flag change made with no server survives

The two sentences, side by side, which the plan asks for:

| what happened | what is said |
| --- | --- |
| the server was not there | `The mail server could not be reached, so your change is saved here and goes the next time Wixen Mail talks to it` |
| the server said no | `The mail server refused your change, so it has been put back: {reason}` |

They share no opening clause and no verb, and a test holds them to that. Twenty
failures in one sync are one sentence carrying a count, through the same topic
superseding 03-07's progress and 03-08's network sentence use.

The distinction is drawn in `why_the_push_failed`, its only caller on that path
is one closure, and it matches on `common::Error`'s variants and never on
message text. Three answers rather than two, because this computer's own write
gate refusing before anything is sent is neither of the other two.

Waiting changes go from where a mail check has already signed in. Nothing
watches the network.

## Premises that were wrong

### 1. `sent_over_a_newer_copy` is not the telling of the losing case

The plan's key link and SCALE-06's evidence both name it as "the telling [that]
already works and is the model for how the new question reaches somebody". It
is the other direction. It counts a change made here that the push re-sent
**over** the address book's newer copy after a stale-marker refusal: a second
both-changed state, resolved in this computer's favour, at the provider, with a
sentence afterwards. The losing case's counter was `replaced`, three hundred
lines away.

Following the plan literally would have changed a working arm and left the one
the requirement is about. Reading every writer of the counter is what caught
it; its definition and doc comment are consistent with the plan's reading.

### 2. CalDAV did not resolve in the server's favour

The plan and the requirement both say CalDAV "resolves automatically, showing
the user nothing", which reads as the server's copy winning. The read skipped
any event with a change waiting, so the **server's** copy was dropped on the
floor and nothing was said at all. Opposite direction from the contacts defect,
same failure, and it changes what the fix is: the etag was not being consulted
at that point at all.

### 3. The guard record counts are counts of mentions, for the sixth time

The plan says 135 records name `contacts_sync.rs`, 61 name `wx_app.rs`, 33 name
`caldav_sync.rs` and 18 name `mod.rs`. What the count check fingerprints is 74,
37, 23 and 11. This is the sixth plan in this phase to find the same figure
quoted the same way. The conclusion the plan draws from it is still right, and
the numbers it draws it from are not.

### 4. `common::Error` distinguished the two cases; the plumbing did not

The plan says that if `common::Error`'s variants "do not currently distinguish
these two cases well enough, that is the first thing to fix". They do, thanks
to plan 03-06. What did not was `the_session_at` and `a_session_at`, which
flattened the error to a `String`. That path is exactly the no-network case for
a flag change, so the variant was gone before anything could read it. Both hand
back the typed error now.

### 5. The line numbers were stale again

`caldav_sync.rs`'s etag sites are not at 870, 1028, 3424 and 3716; two of those
are inside tests. `contacts_sync.rs`'s call sites are at 2289 and 2485 in the
plan and were at 2287 and 2483. Everything here was located against the tree.

### 6. Task 3's own commit message repeated a count of mentions

It says `calendar.rs` holds 69 test functions before and after. 69 is the
number of records that fingerprint it; the file holds 194. The count is
unchanged either way, so nothing rests on it, and it is recorded here because
making the same mistake in the same phase that keeps finding it is the joke.

## Three tests that could not have failed, and how each was found

None was found by reading. All three were found by applying a break by hand and
watching a green run.

**One.** The assertion that a held contact is offered to nobody passed with the
offer filter removed, twice over. First because the script gave an account open
for reading only, so the write gate refused the push before anything was
offered. Then because a script that takes the change clears the waiting flag,
so the next sync owed nothing and offered nothing for that reason. The script
turns the change down now, so the address book is still owed it and the filter
is the only thing that can stop the offer.

**Two.** The two loopback tests for the flag distinction could not be driven
through `MailController`. `connect_imap` opens the write gate only when
`allowed_for` says the account may change things, which is a setting on the
machine running the tests, so both servers answered `Error::Security` before
either was asked anything: the tests proved the gate works and nothing about
the distinction they are named for. They drive the session below it now, which
skips the gate and exercises the socket, the library's own error and this
project's mapping of it.

**Three.** The guardrail 7 check stayed green against its break twice, for two
different reasons. It asked only whether the network-return arm called the
sending itself, and the mention survived because it lives in the function the
arm stopped calling; nobody wires the network's return straight to a flag
change, they wire a mail check, because a check when the network returns sounds
useful and a check now sends the waiting changes on the session it opens. Then,
after it was widened to a list of everything that ends at a server, it searched
for the bare variant name and found the line that **sends** the update,
hundreds of lines above the arm that handles it, so it read an unrelated
function. It matches the arm's own opening now.

The same shape appeared in a fixture: a test wrote `"google"` where
`AddressBook::Google.as_stored()` says `"gmail"`, `from_stored` answered
`Other("google")`, and two tests failed against correct code. The fixture takes
the word from the enum now.

## Verification

Every commit went through `scripts/check.sh` by way of the commit hook. Nothing
used `--no-verify`. Two commits changed `Cargo.toml`, so `which-checks.sh`
answered `all` for both and each ran the whole suite and the release build; both
were run detached, because that gate outruns a ten-minute foreground cap.

The final commit's full gate passed: the library is **6,245 passed, 0 failed, 1
ignored**, every integration target green, release build clean.

**Four reds, all accepted by `scripts/red-commit.sh`.**

| commit | named | what really failed |
| --- | --- | --- |
| f14f835 | 18 | 9 of the 12 pure tests, 4 of 5 storage tests, 5 sync-path tests |
| fd19ba9 | 7 | 2 decision tests, 2 settling tests, 3 of the 5 census tests |
| ce86e12 | 4 | the calendar sync-path test and the three sentence tests |
| f81e597 | 12 | 8 decision tests including both loopback ones, 3 storage tests, the census count |

Every red is a compiling stub that reproduces the shipped behaviour rather than
a missing symbol, so every failure is an assertion. The stubs are worth naming
because each is the real defect: the holding writes nothing, the wording gives
two unlabelled blocks, choosing answers that the provider wins whatever was
asked, the window is built with `set_name`, and every push failure is read as a
refusal.

**Which tests passed at their own red, said plainly.** Twelve. Three of the
first twelve pure tests, because the stub already answered them correctly. One
storage test, because only the writer was stubbed. Two of the five census
tests, written against a stub that already had the right shape. Four more
storage tests in task 4, for the same reason. And the sentence test asking that
a sync holding nothing says nothing about choosing, which the old wording also
satisfied.

**The count check was never red, and that is the result worth quoting.** Every
file the plan warns about holds exactly the test count it held before:
`contacts_sync.rs` 281, `wx_app.rs` 199, `caldav_sync.rs` 67, `calendar.rs` 194,
`message_cache/mod.rs` 23. Nothing was added to or removed from any of them.
The tests that were about an edit being replaced are the tests about it being
held, renamed and repointed in place, and everything new went into six files no
record names. So none of the 74, 37, 23, 11 or 69 records that fingerprint those
files was flagged, and the overnight remeasure the plan budgets for was not
needed.

**Nine guard records, seven new and one removed.** `guards.toml` holds 596, up
from 588. Every new one was measured by hand against the whole library with
`--no-fail-fast`, each break applied on its own.

| record | break | what really went red |
| --- | --- | --- |
| a contact changed in both places is held rather than written over (new) | the arm falls through to the write | 7 |
| only work typed here is held for a choice (new) | the written-here gate dropped | 1 |
| a contact held for a choice is offered to nobody (new) | the push filter dropped | 3 |
| a contact held for a choice is left alone by a later read (new) | the read's skip dropped | 1 |
| every control in the choosing window is named where a screen reader reads (new) | one control named with `set_name` | 2 |
| the choosing window can be reached from the menu (new) | the menu arm emptied | 1 |
| a calendar item both sides changed is held rather than dropped (new) | the etag comparison dropped | 1 |
| a lost connection is told apart from a server that said no (new) | a wildcard arm | 4 |
| the network coming back starts nothing that reaches a server (new) | a mail check in the arm | 1 |
| a lost edit is said once, not on every later sync | removed | the code it named counted a loss, and there is no loss to count |

**The seven-test record's last test is the one worth naming.**
`test_the_other_address_book_setting_is_named_only_while_the_change_is_somebodys_work`
is about a setting, not about conflicts, and nobody filtering for this feature
would have run it. That is the case `CLAUDE.md` warns about, met in practice.

**Four existing records' anchors moved and were updated rather than
re-measured**, because in each case the code did not move: a field was renamed,
a call gained an argument, an expression gained a filter, or a closure was
renamed. Each was checked against the file after editing and
`test_every_guard_record_still_names_one_place_in_the_tree` passes.

**Four records were re-measured because they name tests this plan renamed, and
one of them had gone stale.** A rename can change what a break reddens, and here
it did.

| record | measured |
| --- | --- |
| an edit written here and then lost is still counted and said | all 19 named went red, nothing else |
| a change no setting let out is kept, not replaced | all 10 named went red, nothing else |
| Allow Changes is one of the two settings that keeps a change | all 7 named went red, nothing else |
| the Google merge asks what the whole contact is owed | 4 of 5 named went red |

The fifth is `test_a_held_contact_is_not_resolved_by_the_next_sync`, which was
`test_an_edit_an_address_book_replaced_is_said_once_and_not_on_every_later_sync`.
Its subject moved with its name: it used to be about a loss being said once, and
it is about a second sync leaving a held contact alone, which is decided before
that question is asked at all. It is out of the record's red list, corrected by
hand and then re-measured: **all 4 go red and nothing else does**, so the guard
still discriminates.

That is the case `CLAUDE.md` describes and it is worth saying plainly: the
per-commit check that names records when a test count moves could not have seen
this, because the count did not move. Only asking which records name a renamed
test, and then running them, found it.

## Deviations

**Six new files rather than the two the plan names.** The plan's `files_modified`
lists `conflict_choice.rs` and `flag_changes_waiting.rs`. Storage needed two
data modules, the calendar needed one for its fields and its raising, and the
window needed one of its own. Every one of them is a file no guard record names,
which is what kept the count check green.

**The calendar's sync-path test lives in `calendar_conflict.rs`, not in
`caldav_sync.rs`, and the window's assertions live in an integration target.**
The cost: a test about the CalDAV sync sits one file away from the sync, and the
window's behaviour is asserted by reading source rather than by building a
window. What is therefore not guarded from inside `caldav_sync.rs` is that the
read consults `calendar_conflict` at all, and from inside `wx_conflict_choice.rs`
that the dialog builds what the source says it builds. Both are covered by the
new records, coupled through `guards.toml` so they run on the commits that could
break them. Ledger 85.

**The contacts sentence became a whole sentence rather than an item in the count
list**, and both syncs share one function for it. A question read out as a count
is a question nobody answers.

**Holding one contact stops it being offered to the second address book as
well**, so one refusal is reported where two used to be. That is what
`test_one_edit_to_one_contact_both_books_hold_is_said_once_and_not_twice` now
measures.

**Three of the new status lines were refusals on the status channel**, which
`test_a_refusal_is_not_written_to_the_status_line` caught. They go through
`send_refusal`.

**`the_session_at`'s signature change touched six call sites outside the one
that needed it.** Five take `.to_string()`; one is better for it, an attachment
fetch that used to stringify a typed error and re-wrap it as `Error::Other`.

**Labels are left out of the waiting queue.** A label is a keyword, of which a
message can have many, and replaying one needs the keyword as well as the value.
The arm names the case and says why, and the changelog says so under Known
limitations. Ledger 87.

## What this cannot see, and what it costs

**Nothing here has met a real account, a real address book or a real calendar
server**, and no screen reader has heard any of it. Seven ledger entries rather
than ticks, 82 to 88, five `unrun-verify` and two deviations:

- whether the two copies are understood by ear as a labelled pair, and whether
  announcing each as focus reaches it beats reading both on opening (82)
- whether the count of waiting choices is useful or is a sentence somebody stops
  hearing (83)
- the push retry that still overwrites at the provider without asking (84)
- two files holding tests about code that lives elsewhere, and what that costs
  (85)
- whether the two mail sentences are told apart by ear (86)
- labels being left out (87)
- whether `Authentication` is the right side of the line against a real provider
  (88)

**The first sync of a divergence still offers the local copy once**, and it has
to: the push runs before the read, so the disagreement is only visible after the
offer has been made. What protects the provider is that a change carrying a
stale marker is refused for exactly that reason, and the hold then stops every
later push. Against an address book that ignores markers, that first offer would
land. Recorded here rather than in the ledger because it is a consequence of the
push-then-pull order the module doc argues for, not a defect of this change.

**The push retry is the one both-changed state left that resolves without
asking.** A change typed here is still sent over the address book's newer copy
when a stale marker is refused. It is guarded by two records, its behaviour is
argued for in `guards.toml`, and changing it means changing what those guards
are about. Left alone deliberately and recorded as ledger 84, the same way 03-08
left the Undo Send hold as ledger 81.

**The guard sweep is owed**, per `CLAUDE.md`, once the phase is complete. Nine
records were touched here and every new one measured; the sweep is the one that
asks about the other 587.

**`scripts/guards.sh --touched-by 612126f` after the merge.** This branch changes
`contacts_sync.rs`, `wx_app.rs`, `caldav_sync.rs` and `calendar.rs`, which 74,
37, 23 and 69 records fingerprint. It must not block the merge.

**Nothing was merged and nothing was pushed.**

## Requirements and criteria

**Criterion 6 is closed structurally for contacts and for the calendar.** When a
local copy and a provider copy have both changed, both are kept, somebody
chooses, and nothing is pushed until they do. The choice is keyboard-only and
the two versions are announced as a labelled pair. Two divergent local states
driven through each sync path produce a held conflict, which is the fourth
deliverable.

**It is not closed by ear**, and entries 82, 83 and 86 are why. It is not closed
in the field either: no provider has ever been used with this program.

**Mail is out of the criterion on purpose and that is now a finding rather than
an omission.** SCALE-06's evidence is corrected on three counts and the
reasoning that keeps somebody from building a mail conflict chooser is kept.
**Mail's real defect is fixed** rather than only named.

## Self-Check: PASSED

- All eight commits are in `git log`: f14f835, ce3e89b, fd19ba9, a2a046d,
  ce86e12, 171223d, f81e597, 8b05528.
- The eight new files exist: `src/application/conflict_choice.rs` (17 tests),
  `src/application/calendar_conflict.rs` (5),
  `src/application/flag_changes_waiting.rs` (16),
  `src/data/message_cache/held_conflicts.rs` (8),
  `src/data/message_cache/waiting_flag_changes.rs` (4),
  `src/presentation/wx_conflict_choice.rs` (0, its assertions are the target
  below), `tests/the_conflict_choice_can_be_heard.rs` (5),
  `tests/nothing_sends_a_flag_change_unasked.rs` (3).
- The final commit's `scripts/check.sh all` passed: 6,245 library tests passed,
  0 failed, 1 ignored, every integration target green, release build clean.
- `guards/guards.toml` holds 596 records, seven new here, one removed, four
  anchors updated.
- `grep -n '^version' Cargo.toml` reads `version = "0.58.0"`, up from 0.56.0 in
  two steps, one per behaviour change.
- `docs/changelog.md` carries two entries from this plan under `[Unreleased]`,
  each naming its limits.
- `.planning/WINDOWS.md` holds entries 82 to 88, all open, and they landed in
  this worktree rather than in the shared checkout, which was checked rather
  than assumed: the shared checkout's ledger still ends at 81.
- Every expensive file's test count is unchanged: 281, 199, 67, 194 and 23.
- The working tree is clean apart from this document.

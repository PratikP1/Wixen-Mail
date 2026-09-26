---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 9
subsystem: application::undoing, data::message_cache::taking_back, presentation::managers, presentation::wx_app
status: complete
tags: [undo, redo, contacts, calendar, tasks, notes, reminders, deletion-notes, accessibility, GAP-02]
requires: [13-08]
provides:
  - application::undoing, LastAction::OnAnItem and what undoing or redoing an action on an item does, from what the store says
  - data::message_cache::taking_back, the record a delete keeps, take_a_deletion_back, make_it_again, and ASyncUnderWay
  - managers::undo_or_redo_on_an_item, the undo carried out through the paths the action took
  - pim_command, and the contact and reminder filing paths, answering what they did so it is remembered
affects: [13-42, 13-51]
tech-stack:
  added: []
  patterns:
    - "the one caller allowed to drop an owed deletion note changes the row and the note in one transaction"
    - "a background sync counts itself in the process, and the take-back holds that count's lock to its commit"
key-files:
  created:
    - src/data/message_cache/taking_back.rs
  modified:
    - src/application/undoing.rs
    - src/application/deletions.rs
    - src/application/pim_command.rs
    - src/application/allowed.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/contacts.rs
    - src/presentation/managers.rs
    - src/presentation/wx_app.rs
    - tests/undoing_a_mark_names_the_message.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "A deleted item comes back as it was while its deletion note is owed, through take_a_deletion_back; once taken, as a new item with no identity at the account, the taken note left to mask the reads."
  - "No sync marks its push, so each of the window's four syncs of items counts itself (ASyncUnderWay), and a take-back holds that count's lock from its check to its commit so no sync can begin in between."
  - "A delete's question says Undo brings the item back, since 'This cannot be undone' stopped being true."
  - "Undo's help names the account rather than the mail server, since it is Undo's help in all six modules."
  - "A day taken off a repeating event clears the undo step rather than leaving an older action for Ctrl+Z."
metrics:
  duration: about 4 hours
  completed: 2026-09-25
actuals:
  tokens: 29764
  tasks: 3
  commits: 7
---

# Phase 13 Plan 09: Undo in the other five modules Summary

In Contacts, Calendar, Reminders, Tasks and Notes, Edit, Undo (`Ctrl+Z`) with the list
focused takes back the last Mark Done or Not Done, Pin or Unpin, Delete, Move to or Copy
to, and Redo (`Ctrl+Y`) does it again, named on the menu: "Undo Delete: Dentist". It is one
step with the message list, so the last action anywhere is what Undo takes back, and Undo
in a list whose module did not take it says where it was. A deleted item comes back with
every field it had while its account has not been told, and the deletion is never sent; one
the account has already taken comes back as a new item there, and the sentence says so.
Nothing is decided while that account's sync of that kind runs. A move goes back where it
came from, a copy's undo takes away the copy after asking, never the original, and the
cursor lands on the item that came back when its list is read back. #47 is closed from the
merge.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards` and `docs`
against `main` at `cb38ec08`.

## How each sync marks its push, read before the red

None does. `push_deleted_contacts_to_google` (`contacts_sync.rs:2021`), `push_tasks`
(`tasks_sync.rs:640`), `push_what_is_waiting` (`notes_sync.rs:359`) and
`push_to_the_calendar_server` (`caldav_sync.rs:114`) read the owed notes and send them with
nothing written anywhere while they run, and the window's `spawn_contacts_sync`,
`spawn_tasks_sync`, `spawn_notes_sync` and `spawn_calendar_sync` (`wx_app.rs:27194` to
`27512` at `cb38ec08`) open their own store on a worker thread. So the refusal is built on
`ASyncUnderWay`, a count per account and kind in the process, the shape 13-08's
`APushUnderWay` took for mail, held by each of the four for the whole of its closure. This
was also written into the second red's message before that commit.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `7a3347e9` | red | the item types with empty answers, 9 cases in `undoing`, the count check | 134 s (refused once at 26 s, below) |
| `1257b33b` | green | the decision and the words; two records new, five re-measured | 165 s |
| `0a4eaaf9` | red | `taking_back`'s calls answering errors, 9 of its 10 cases | 174 s |
| `543740ce` | green | the take-back, making again, the count; `write_the_contact`; `deletions.rs`'s paragraph; two records | 167 s |
| `a69ab076` | red | the reading's four cases and two companions, three sentences' tests, the count check | 156 s |
| `d3738710` | green | the recordings, the undo carried out, the menu, the syncs' counts, the landing, the pages; one record new, one rewritten, five re-measured | 259 s |
| this commit | docs | the ledger, the requirement, this summary, the four marks | |

**Test counts, taken again.** `cargo test --lib application::undoing::` 30, 21 before (the
plan asked for at least 28). `cargo test --lib data::message_cache::taking_back::` 10, new
(at least 8). `cargo test --lib application::deletions::` 4, unchanged.
`cargo test --lib presentation::managers::` 137 before and after, no test added.
`src/presentation/wx_app.rs` 199 tests, unchanged. `cargo test --test
undoing_a_mark_names_the_message` 21, 15 before. `cargo test --lib application::pim_command::`
38 and `application::allowed::` 27, each unchanged in count with one test's words changed.
`house_style` 74, `wired` 77, `the_words_that_say_nothing` 10,
`the_planning_files_agree_with_themselves` run before this commit.

**Acceptance readings.** `grep -c 'take_a_deletion_back' src/application/deletions.rs` is 1.
`grep -c 'last_action' src/presentation/managers.rs` is 3. The guide's section from
"## Other modules" to "## Keyboard Shortcuts" names Undo on 7 lines, read with Grep over
that range (the plan's `awk` is refused by this brief). Carriage returns 0 on every file
touched, by `tr -cd '\r' < FILE | wc -c`; no em dash on an added line; none of the six
words.

**T-13-32, the gate the undo meets.** Nothing in the undo sends anything itself: a take-back
or a new item is written here with the fields a sync reads, and goes out through that sync's
own push. Each push's client is built by `for_account`, which reads
`allowed_for(account_id)` and gives the client a way to change things only when
`allowed.personal_information` says so, for instance `TasksClient::for_account` at
`src/service/tasks_api.rs:861`: `http: if allowed.personal_information {
Outward::may_change_things(...) } else { Outward::default() }`.

**The tracer** the plan names, a task deleted with the network off, then Ctrl+Z, was not
run: the program is not started on this machine. The store case
`test_an_owed_task_is_taken_back_with_every_field_and_its_note_gone` holds its store half
on a temporary cache, the decision case `test_undoing_a_deleted_task_puts_back_every_field_it_had`
the decision, and the reading the wiring from the key to `take_a_deletion_back`; CI's scan
and NVDA jobs build the window.

## Guard records

| Record | Red | Run |
|--------|-----|-----|
| an undone delete the account was never told of comes back as it was (new, `undoing.rs`) | the deleted-task case | 90 s |
| an undone copy of an item deletes the copy, never the original (new, `undoing.rs`) | the copy case | 89 s |
| five of 13-07's and 13-08's on `undoing.rs`, flagged by the count check | each unchanged | one call with the two new, about 12 minutes |
| a take-back drops the deletion note only with the row it puts back (new, `taking_back.rs`) | the rollback case | 97 s |
| a take-back never drops a deletion the account has taken (new, `taking_back.rs`) | the taken case | 85 s |
| a delete answered No leaves the undo step as it was (new, `managers.rs`, suite the reading) | the recording reading | 18 s |
| a refused move is said on the refusal channel rather than the status line (its break rewritten) | unchanged, 1 | 112 s |
| four naming the reading, flagged by the count check | each unchanged | one call with the two above |

The arrived-since line at the head of `guards/guards.toml` went from 325 to 330. The plan's
third break, "records a delete that was cancelled at its confirmation", is written as a
cancelled delete that clears the step, since a recording the reading can see before the
command is carried out is exactly that fault.

Records by anchor text, read before the edits: three anchored in `pim_command`'s region
(`both exits after a confirmed delete say the row has gone`, `one place words a row that has
gone`, `a refused move is said on the refusal channel rather than the status line`), one in
`move_a_contact_between_groups` (`the question about which group a contact is leaving is
asked of the groups it is in`), two in `wx_app.rs` regions near the edits (`how many task
lists arrived is said, not only shown`, `Cut and Copy ask the box what is selected`), none
in `deletions.rs`'s header or `save_contact`. Only the refused-move record needed changing:
its `before` still names one place, but its `after`, `Ok(why)`, no longer compiles against
the arm's new answer, so it is `Ok((why, None))` now and was measured again.
`test_every_guard_record_still_names_one_place_in_the_tree` passed on every green.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3] A clippy refusal at the first red (26 s).** `large_enum_variant` on
`LastAction` and `ItemDid` once they held a whole record; the record is boxed in
`ItemDid::Deleted`, `UndoAnItem::TakeTheDeletionBack` and `UndoAnItem::MakeItAgain`.

**2. [Rule 1] "Mark Not Done" is read by `wired` as a command no window carries.** The item
is "Mark Done or Not Done". The two ways are named "Mark as Done" and "Mark as Not Done",
the way Mark as Read is; the red's expected names were changed in the green to match, and
the commit says so.

**3. [Rule 1] A delete's question said "This cannot be undone."** With Undo in these
modules that stopped being true, and a person who deleted the wrong row would be told to
type it again. `confirm_delete` now asks `Delete "Dentist"? Undo brings it back until your
next action.`; `pim_command.rs`'s test was changed in the third red to hold it, and the
shortcuts page's Deleting paragraph says the same.

**4. [Rule 1] Undo's help named the mail server.** It is Undo's help in all six modules now,
so it names the account; `allowed.rs`'s test holds that it does not say "mail server".

**5. [Rule 3] `save_contact` opens its own transaction**, and SQLite takes one at a time, so
a contact could not be put back inside the take-back's. Its writes moved, unchanged, into
`write_the_contact`, which takes the caller's transaction; `save_contact` calls it. No
record was anchored there.

**6. [Shape] The types differ from the plan's sketch.** `LastAction::OnAnItem(OnAnItem)`
holds the kind, the name, the id the item has now and `ItemDid` (`MarkedDone`, `Pinned`,
`Moved { from, to }`, `Copied { copy_id, into }`, `Deleted(record)`) rather than an
`ItemBefore` of optional fields. `WhatTheItemStoreSays` has a fifth answer, `Gone`, for an
item not here with no note, which is how a deleted reminder reads and how a toggled item
deleted since reads. `UndoAnItem` answers `MarkDone(bool)` and `Pin(bool)` rather than a
toggle, so an undo after somebody else changed the item does not turn it the wrong way,
and `MoveTo`, `CopyInto` and `DeleteIt` for Redo. The record type lives in
`data::message_cache::taking_back` so the store's call takes the store's types.

**7. [Shape] `make_it_again` takes the new identifier from its caller**, as
`copy_reminder_to_account` does, so `managers::new_id` stays the one minter.

**8. [Rule 2] What the undo is for a contact and a reminder.** A contact's move is between
two groups and its copy a place in a second group, so undoing the copy takes her out of
that group and never deletes her; a reminder's move and copy are between accounts. The
contact and reminder filing paths now answer what they did, `Option<ItemDid>` or the group,
so `pim_command` can remember it, and a membership that did not change remembers nothing.

**9. [Rule 2] A day taken off a repeating event clears the step.** It is a save rather
than a delete and has no undo here, and leaving the older step would let Ctrl+Z undo
something the person did before it. Ledger 629.

**10. [Test] The reading's five lists are matched without their opening parenthesis.** The
window hands the Edit menu its lists through `pim_refs`, where the red's anchor assumed
local names; changed in the green, and the commit says so.

**11. [Brief] No `awk`.** The plan's acceptance line for the guide uses `awk`; the range was
read with Grep instead.

### Found and left

- A deleted task's subtasks stay out from under it after Undo (ledger 629).
- A race narrower than 13-08's: the count's lock is held from the check to the commit, so a
  sync cannot begin in between; a sync started some way other than the window's four
  spawns would not be counted (ledger 628).
- For a reminder the help's "sent to your account" describes nothing, since reminders are
  kept here; left, as the sentence stays true for the other five.

## Threat Flags

None beyond the register. T-13-30: the note is dropped only inside `take_a_deletion_back`,
in one transaction with the row and only while owed, held by the rollback case and the taken
case and a record each; `deletions.rs` names it the one path. T-13-31: `ASyncUnderWay` and
the sync case, the window's four syncs counted, read by the reading. T-13-32: quoted above.
T-13-33: the record is held by `last_action` in memory, replaced by the next action, never
written or logged. T-13-SC: nothing added to the manifest; `Cargo.lock` unchanged.

## Known Stubs

None. Each stub answered nothing in its red commit only.

## Ledger

Opened: 627 (`unrun-verify`, the tester's ear in each of the five modules), 628
(`unrun-verify`, an undone action reaching Google, Microsoft or a calendar server, owed and
taken, REAL-01's lines for Gmail and none yet for the others), 629 (`todo`, a deleted task's
subtasks and a day taken off a series). Closed: none. Both halves of each.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does
not move.

## Self-Check: PASSED

- `src/data/message_cache/taking_back.rs`, `src/application/undoing.rs`,
  `tests/undoing_a_mark_names_the_message.rs`: present.
- `7a3347e9`, `1257b33b`, `0a4eaaf9`, `543740ce`, `a69ab076`, `d3738710`: in `git log` on
  `13-09-undo-in-the-other-modules`.

# Phase 5: The other five modules keep up

Eight plans, one per wave. Assembled 2026-09-06 against `main` at `9611b70`,
version `0.75.0`, `guards/guards.toml` holding 632 records, Rust floor `1.88`.
Nothing in the repository was changed while these were written.

This phase is what used to be the first half of a single nine-plan phase 5.
Pratik's decisions of 2026-09-06 grew that work past what one phase should carry,
so it was cut into three. Phase 5.1 takes notes and CardDAV, phase 5.2 takes
OneNote. The reasoning for both cuts is in this directory's siblings.

**Goal.** Contacts, calendar, tasks, notes and reminders support the same moves
mail already does, and a task that lives on a provider can move without being
lost between two calls.

**Requirements:** PIM-01, PIM-02, PIM-03, PIM-06.

**Roadmap success criteria this phase owns:** 1, 2 and 3 of the six the roadmap
gives phase 5 today. Criteria 4 and 5 go to phases 5.1 and 5.2, criterion 6 to
phase 5.1.

## The plans

| Plan | Wave | Requirements | Depends on | Human | What it does | Source |
|---|---|---|---|---|---|---|
| 05-01 | 1 | PIM-03 | none | no | Settle by running it whether a moved day of a series is already shown once, then make a changed day say it differs | carried |
| 05-02 | 2 | PIM-06, PIM-03 | 05-01 | no | Week and month as narrower windows over the list that exists, with Prev and Next that move and announce | carried |
| 05-03 | 3 | PIM-02 | none | no | Copy beside move for events, tasks and notes | carried, checkpoint struck |
| 05-04 | 4 | PIM-02 | 05-03 | no | A contact moves between groups, and the move asks which group it is leaving | new |
| 05-05 | 5 | PIM-02 | 05-03 | no | A reminder moves to another account, which is the first PIM move to cross one | new |
| 05-06 | 6 | PIM-01 | 05-03 | no | The local half of "exactly one list" proved, a move that says what has and has not happened, and PIM-01's text corrected to match | carried, checkpoint replaced by the edit it was asking permission for |
| 05-07 | 7 | PIM-01 | 05-06 | no | A half-finished move survives the program closing, with no provider called yet | new |
| 05-08 | 8 | PIM-01 | 05-07 | yes, at the end | The provider move itself: delete there, create here, and a recovery that reads from the local copy | new |

Requirement coverage: PIM-01 by 05-06, 05-07 and 05-08. PIM-02 by 05-03, 05-04
and 05-05. PIM-03 by 05-01 and 05-02. PIM-06 by 05-02.

Every plan is a wave of its own. That is not caution: 05-04, 05-05 and 05-06 all
write `pim_command.rs`, `context_menu.rs` and `managers.rs`, and every plan in
the phase writes `guards/guards.toml`, `docs/changelog.md` and `Cargo.toml`. Two
plans in one wave would be two branches fighting over the same four files.

## The risk ordering, which is the point of the last three plans

Nothing deletes anything at a provider until the state its failure leaves is a
tested object.

`05-06` proves the local half and makes the move honest about what has and has
not happened. `05-07` builds the stored in-progress state and the resume path,
and calls no provider at all. Only then does `05-08` start deleting at a
provider. After `05-07`, a half-finished move is a visible, recoverable, testable
thing; before it, a failure between the delete and the create is a lost task and
nothing can see it.

This follows a rule the codebase already settled three times, always the same
way: put the destructive write last. `rename_task` saves the new task then drops
the old one. `one_day_kept_out_of_the_series` saves the changed day then the
series. `02-06-SUMMARY.md` records writing a saved search with the parent stamped
last on purpose, because stamping it first put the only reachable failure before
anything was destroyed and left no test able to tell a transaction from three
loose statements. `05-04`, `05-07` and `05-08` all cite it and none of them may
choose the other order without saying why in writing.

## What Pratik answered, so no plan here asks it again

Three of the four checkpoints the old nine-plan set carried belonged to this
half, and all three are struck. `.planning/decisions-2026-09-06.md` is the
record.

1. **Does PIM-02 mean five modules or three?** Five, decision 3. The old
   `05-03`'s checkpoint is deleted. `05-04` adds contacts and `05-05` adds
   reminders.
2. **Is a provider task move in scope for PIM-01?** Yes, build it, with the local
   cache as the safeguard, decision 1. The old `05-04`'s checkpoint is deleted.
   `05-07` and `05-08` build it.
3. **What does PIM-01's Allowed criterion mean?** Reword the criterion, keep the
   behaviour, decision 5. `05-06` task 3 makes that edit rather than asking
   permission to make it.

Decision 4 answers a gap under decision 3: a reminder moves to another account,
which is the container reminders already have, so there is no new table and no
invented concept.

### What the rejected options lost on

The checkpoints were struck from the plans, and that would have thrown away the
most expensive part of them: the reasons the other answers lost. They are kept
here, with the date and the fact that decided each, so a later reader can tell a
settled decision from an unexamined assumption, and can tell a rejection that
still holds from one whose premise has expired.

**Decision 3, does PIM-02 mean five modules or three.** Answered 2026-09-06:
five. Three options were on the table.

| Option | Taken | What it lost on |
|---|---|---|
| Three modules is what criterion 2 means; correct the requirement | no | It matches what the code already argues, and the reasons in `pim_command.rs` were written deliberately rather than by omission. It lost because the roadmap's own wording of criterion 2 names all five, so it would have needed correcting too, and anybody reading only the roadmap later would think a module was dropped. |
| Add contacts through group membership, leave reminders out | no | It closes four of five and most of the machinery exists. It lost to the option above it on completeness, not on cost. Its cost estimate carried one error worth keeping: it claimed `test_move_is_offered_exactly_where_it_means_something` and its copy twin both go red, which is false for a correct widening. |
| All five, give reminders a container | taken, with a correction | Its stated cost was "a reminder has no container and inventing one is a product decision, a schema change and a new idea somebody has to be taught". Decision 4 removed that cost rather than paying it: a reminder moves between accounts, which is the container `reminders.account_id` already gives it, so there is no new table and no invented concept. The option as written was rejected; the option as decision 4 reshaped it was taken. |

**Decision 1, is a provider task move in scope for PIM-01.** Answered
2026-09-06: build it, with the local cache as the safeguard.

| Option | Taken | What it lost on |
|---|---|---|
| The refusal is the answer; correct both criteria and say so in the alpha notes | no | Nothing gets built that can lose a task, and PIM-01 would close on what ships. It lost because somebody with a Google or Outlook task in the wrong list still could not fix it here, and the roadmap's criterion 1 says "including when the move fails at the provider". |
| Build the provider move, as its own plan, with the half-finished state handled first | taken | Its stated cost stands and is what `05-07` and `05-08` are shaped around: it is the only work in phase 5 that can lose data, the failure it is about cannot be produced without an account, and it needs stored state for a half-finished move that survives the program closing. |

**Decision 5, what PIM-01's Allowed criterion means.** Answered 2026-09-06:
reword the criterion, keep the behaviour.

| Option | Taken | What it lost on |
|---|---|---|
| Refuse the move outright when Allow Changes is off, for an item a provider syncs | no | It would make the criterion true as written, with no local divergence to reconcile. It lost because it takes away an edit that works, and because it disagrees with every other write in the program: an event, a note and a contact are all written locally and held. Making a task move the exception is worse for somebody moving by keyboard. |
| Keep the behaviour and reword the criterion, and make the move say so sooner | taken | The gate stays where each HTTP client is built. `05-06` task 2 makes the move say at the moment it happens that nothing has left the machine, and task 3 rewords PIM-01 to describe the gate where it really is. |

One checkpoint remains, at the end of `05-08`, and it is not a question the old
set asked. It is a human read-back of what the recovery does and what somebody
hears when a move is half finished, and it is there because that plan is the only
work in the three phases that can lose data.

## Premise corrections that must survive into execution

Each was taken by reading or running the code rather than by reasoning about it.
They are the reason the plans are shaped the way they are, and losing one costs
real work.

**The calendar double-booking may not exist, and `05-01` runs it rather than
assuming.** `05-RESEARCH.md` assumption A1 says a moved occurrence appears twice,
once expanded from the series and once as its own row. Reading the code says the
opposite: every path that stores a moved day also takes that day off the series,
and the two sites that cannot do so log a line saying the day cannot be taken off
the series and the meeting may be shown twice, and count it. Neither reading is
evidence. `05-01` task 1 is the test that settles it, and the plan is arranged so
either answer is useful.

**`file_under`'s last arm lumps reminders and contacts into a silent `Ok(())`,
and this is the most dangerous trap in the widening.** The arm is
`ItemKind::Mail | ItemKind::Contact | ItemKind::Reminder => Ok(())`, with a
comment saying it is never reached because `kept_in` gives these no container,
and that it is written out rather than caught by a catch-all so a new kind of
item is a compile error there. That is true of a new `ItemKind` variant and false
of an existing kind changing category. Widening
`PimCommand::Move.applies_to` to `Contact` and `Reminder` without touching that
arm compiles, runs, reports success and writes nothing. `05-04` and `05-05` both
name it and both require a test that would catch it.

**`test_move_is_offered_exactly_where_it_means_something` does not go red on a
correct widening, and three earlier documents said it does.** Read it at
`src/application/context_menu.rs:421`: it loops `ItemKind::ALL` and asserts the
menu offering `Action::MoveItem` equals `PimCommand::Move.applies_to(kind)`. It
is an agreement test in both directions. Widening the menu and `applies_to` in
one commit leaves it green; widening one of them alone reddens it. It is a guard
against a half-widening, not the red half for this work. A plan that promises a
red which does not arrive teaches its executor to write a test that was never
able to fail.

**Copy for a contact already ships, under another name.** `Action::AddToGroup`
exists at `src/application/context_menu.rs:143` with the doc comment "Put the
chosen contact in a group", and
`change_the_group_a_contact_is_in(Membership::PutIn, ..)` in `managers.rs` is its
handler. Putting a contact in a second group without taking it out of the first
is what a copy of a contact is. So `PimCommand::Copy` for a contact routes to
that code rather than growing a second door to it, which is `05-04` task 2 and is
why contacts are one plan rather than two.

**A provider task move needs no new provider call.** `push_tasks` in
`tasks_sync.rs` already has both halves: the first loop reads
`cache.deleted_tasks` and calls `google_delete_task` or `ms_delete_task`, and the
second reads `cache.pending_tasks` and calls `push_one`, which creates when
`provider.is_local(&task.id)` and then calls `settle`, which calls `rename_task`
when the provider hands back a new id. Writing a new identity over an old one is
already done, in the function a provider move extends. What is missing is the
in-progress state between the two, which is why `05-07` exists and why `05-08` is
smaller than it looks.

**The task read path was already built expecting a task to move between lists.**
`held_everywhere` in `sync_google_tasks` carries the comment "a task moved out of
one list comes back in another, so one list at a time makes a move look like a
deletion", and the pass gathers `held_everywhere` and `arrived_everywhere` across
the whole account for that reason. `05-08` must confirm the Microsoft pass does
the same rather than assuming it, and says so. If it does not, that plan is much
larger than it says and it is required to stop and report.

**`held_in` answers one container and has no arm for a contact.** It returns
`Option<String>` and matches `Event`, `Task`, `Note` and then `_ => None`. A
contact is in as many groups as somebody put it in, so the question a move has to
ask is not "which container is it in" but "which of the several is it leaving".
That shape change is the whole of `05-04` task 1.

**Mail moves between accounts now, and this paragraph used to say it did not.**
Corrected 2026-09-08 against `main` at `abb1218`. When phase 5 was assembled on
2026-09-06 the claim was measured and right: every `Branch` built outside
`#[cfg(test)]` held one account, so decision 4's "the way mail already moves
between accounts" described something that did not happen, and the machinery was
unreached rather than missing. Phase 4.1 reached it.
`destinations::where_mail_can_go` builds a `Branch` per account from the drawn
sidebar rows, `where_this_message_can_go` wraps that in `offer`, and
`wx_app.rs:17310` calls it from the move somebody makes.

What that costs phase 5 is one framing and no design. `05-05` still uses
`pick_one` rather than the destination tree, because the tree answers a folder
inside an account and never an account itself: `build_destination_dialog` pushes
`None` for every account row on purpose, with a comment saying an account
heading is somewhere to look rather than somewhere to put something. So `05-05`
is the first *PIM* move to cross an account, it is not the first move in the
program, and its summary must not claim to be unprecedented.

Phase 4.1's `PLANS-README.md` predicted this correction and named four places to
make it. The sweep it gave, `grep -rn "first move\|first cross\|crosses one"`,
missed two more that word the same claim as a negative, which is the failure
that document was itself warning about one paragraph earlier. Six were
corrected. Whoever writes the next such note greps for the negative as well as
the positive.

**`offer` deletes empty branches, so a reminder needs something to stand in an
account's places.** `offer` ends with
`.filter(|branch| !branch.places.is_empty())` and `anywhere` is
`branches.iter().any(|branch| !branch.places.is_empty())`. `05-05` names the
three ways out, rules out two with reasons, and makes the executor confirm the
third by reading how `wx_destination::ask` announces a branch and its places
before writing anything.

**`reminders.related_event_id` is written `None` at every production site and
resolved by nothing.** Three writes, at `managers.rs:3063`, `ui_types.rs:2206`
and `wx_app.rs:22643`, all `None`. So a reminder move across accounts cannot
break a link today, because no link is ever made. `05-05` records it as a
measured fact and as a trap for whoever makes the first one.

## Costs every plan is written around

**Guard records, re-measured at `9611b70`: 632 records.** The old drafts of these
plans said 617 and then 619. Phases 4 and 4.2 landed in between. Count records
rather than mentions, with the awk in `CLAUDE.md`; a grep for `contacts_sync`
answers 363 against 77 real records.

| file | records | test functions |
|---|---|---|
| `src/application/contacts_sync.rs` | 77 | 281 |
| `src/application/calendar.rs` | 69 | 194 |
| `src/presentation/wx_app.rs` | 42 | 199 |
| `src/presentation/managers.rs` | 40 | 137 |
| `src/data/message_cache/contacts.rs` | 34 | 103 |
| `src/application/tasks_sync.rs` | 19 | 106 |
| `tests/wired.rs` | 10 | 64 |
| `src/application/allowed.rs` | 4 | 26 |
| `src/application/pim_command.rs` | 3 | 18 |
| `src/application/occurrences.rs` | 3 | 62 |
| `src/data/message_cache/tasks.rs` | 3 | 21 |
| `src/data/config.rs` | 2 | 53 |
| `src/presentation/read_aloud.rs` | 2 | 46 |
| `src/presentation/wx_destination.rs` | 2 | 6 |
| `src/application/destinations.rs` | 1 | 28 |
| `src/application/contact_groups.rs` | 1 | 24 |
| `src/presentation/pim_rows.rs` | 1 | 23 |
| `src/presentation/wx_settings.rs` | 1 | 0 |
| `src/application/context_menu.rs` | 0 | 15 |
| `src/presentation/ui_types.rs` | 0 | 54 |
| `src/application/deletions.rs` | 0 | 4 |
| `src/data/message_cache/reminders.rs` | 0 | 5 |
| `tests/calendar_immediate_actions.rs` | 0 | 1 |
| any new file | 0 | 0 |

Four figures moved since the drafts and are corrected in every plan that quotes
them: `wx_app.rs` 40 records to 42, `tests/wired.rs` 8 records and 61 tests to 10
and 64, `config.rs` 52 tests to 53, `reminders.rs` 4 tests to 5. Two were wrong
when they were written and are corrected in `05-01`: `pim_rows.rs` 12 tests to 23
and `tests/calendar_immediate_actions.rs` 3 tests to 1.

**The census at the top of `guards/guards.toml` moved with it.** Line 79 says 192
and line 80 says 440, and 192 plus 440 is 632, which is what the file holds.
`test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
adds them and compares, inside the commit gate, so a plan adding a record bumps
line 80 in the same commit. Read both lines before trusting either: `04-05` met
this as a refused commit.

**The Rust floor is 1.88, raised by `04-09` from 1.87 because rPGP declares it.**
Clippy only suggests a construct once the declared floor allows it, so the bump
turned on `collapsible_if`'s let-chain form and cost seven fixes across six files
that plan never opened. `main` is clean at that floor, so nothing is owed. What
it means for this phase is that new code is linted against 1.88 with
`-D warnings`, where a let-chain suggestion is a build failure and not a hint.

**Land all of one file's additions in one commit.** Two tasks that each add a
test to one fingerprinted file pay the scoped remedy twice, and the second run
finds nothing the first did not.

**A commit touching `Cargo.toml` makes `scripts/which-checks.sh` answer `all`**,
the whole gate at roughly 311 to 353 seconds warm. Version bumps go in the same
commit as the user-visible change, so most GREEN commits here pay it. Run them
detached, never inside a ten-minute foreground cap, and never pipe `check.sh`
into anything whose exit status is then read.

**A red commit is allowed only on a branch**, must name every failing test in
`Fails-until-green:` trailers at column 0, and every named test must have run and
failed with nothing else failing. Name test functions, never assertions inside
them. Where adding a test makes the count check red at the same time, name that
check as one of the failures, which is what `04-06` did.

**`WIXEN_TEST_THREADS=4`** halves a guard run and does nothing for `check.sh`.

**A guard living in `tests/` needs a record with `suite = "<file name without
.rs>"`**, or it runs on every commit except the ones that could break it.

## Estimates

Carried plans keep their `estimate` blocks except `05-03` and `05-06`, whose
checkpoints were struck. New plans use the same derivation, and it should be
re-derived once phase 5 has actuals: `raw_tokens` is 35,000 per task, the mean
raw projection per task across the nine phase-4 plans, and `tokens` is that
multiplied by 1.03, the mean of `actuals.tokens / estimate.raw_tokens` over the
six phase-4 plans with recorded actuals (1.17, 1.29, 1.29, 0.48, 1.18, 0.78). Six
samples with that spread is `med` confidence and not `high`.

`05-06` is the one place where the carried instructions and the shipped plan
disagree, and it is recorded here rather than smoothed over. `CARRY-FORWARD.md`
said its estimate drops to two tasks and, four paragraphs later, told the plan to
add a third. The plan has three `<task>` blocks, so `tasks: 3` with
`raw_tokens: 105000` is what shipped, which is the derivation every other
three-block plan in the set uses. Two would have made it the only plan in
seventeen whose frontmatter contradicted its own body.

## What no plan in this phase can close

- **Whether a provider accepts a task move done as delete-there-create-here.**
  That is the failure PIM-01's first criterion is entirely about. `05-06`
  measures the local half, `05-07` makes the half-finished state visible and
  recoverable, `05-08` builds the calls, and none of the three can produce the
  failure between the two calls without a provider.
- **Every accessibility criterion in PIM-02 and PIM-06.** Whether a week view
  reads in date order, whether a month of rows in one list is navigable, whether
  "copied to" and "moved to" are told apart at speed, and whether the new "which
  group is it leaving" question is one more stop than somebody wants in a move.
  `05-08`'s checkpoint covers the half-finished move and nothing else; the rest
  are ledger entries.
- **Whether `Ctrl+Shift+V` really reaches the handler in the non-mail modules**
  rather than only appearing in a menu label. `05-RESEARCH.md` assumption A2 is
  not settled by any test in this repository.

**`.planning/WINDOWS.md` held 192 entries on 2026-09-08, not the 119 this
paragraph used to say, and it will hold more by the time you read it.** The
number was right when phase 5 was assembled on 2026-09-06 against a tree where
phases 4.1 and 4.2 had not landed; both have since, and 4.1 alone opened
fourteen. Every plan in this phase repeated the 119, so all eight were corrected
on 2026-09-08 to say the same thing this paragraph now says: **read the last row
of the file before choosing a number, rather than trusting any figure written
down here.**

What it costs to get wrong is not a wrong number in a document.
`tests/the_planning_files_agree_with_themselves.rs` refuses a duplicated ledger
id, the file stores every entry twice as a markdown table and a JSON block that
must agree, and that target runs inside the commit gate. So the commit is
refused, and the message is about the two halves disagreeing rather than about
the numbering, which reads as a broken ledger rather than as a plan quoting a
stale figure.

Each plan says which entries it owes, one entry per unrun thing rather than one
entry for all of them, because an entry saying "this has never been tried" tells
the next reader nothing about which part to try first.

## What is owed to documents, and belongs to whoever lands these

Checked against the repository on 2026-09-06 at `9611b70`.

1. **The roadmap's phase-5 entry says `**Plans**: TBD`.** It wants the count and
   the plan list, and its six success criteria want splitting across the three
   phases the way this file and its siblings split them.
2. **Two new roadmap entries**, `### Phase 5.1: ... (INSERTED)` and
   `### Phase 5.2: ... (INSERTED)`, following the shape of the 2.1 entry.
3. **The roadmap's phase-3 plan list is still half stale.** The checkbox list has
   03-09 unticked while its summary exists, and the plan list below it has every
   entry unticked.
4. **`.planning/STATE.md` has drifted again.** It says `current_phase: 02` and
   `status: executing` while its own `stopped_at` says `Completed 04-09-PLAN.md`,
   its `last_activity_desc` is about 02-06, its `state_head` is three commits
   behind `main`, and `progress` says 50 of 50 plans complete at 0 percent. It
   carries a note recording that this exact confusion happened before and was
   corrected on 2026-09-01.
5. **PIM-01's `Allowed` criterion is reworded by `05-06` task 3** and PIM-02's
   first criterion by `05-05`. Both edits belong to the plan that makes them
   true, not to this pass, for the reason this project keeps rediscovering: a
   document corrected ahead of the code describes unbuilt work as built.
6. **The recurrence editor's stated limitation was not checked**, and PIM-03's
   third `[D]` line asks that both stated limitations are restated in the product
   rather than left in the changelog. `can_be_honoured` already refuses in the
   product with a sentence; the editor's half is owed, and `05-01` says so in its
   summary rather than letting the requirement read as closed.

Two items from the earlier draft of this list are resolved, recorded here so
nobody re-does them. `.planning/decisions-2026-09-06.md` is in the repository
already, so the seven plans that reference it will find it. And the phase-5
blocker line about PIM-04 needing a sync target chosen is no longer in the
roadmap entry.

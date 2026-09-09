---
phase: 05-the-other-five-modules-keep-up
plan: 04
subsystem: ui
tags: [contacts, groups, move, copy, transaction, guards]

requires:
  - phase: 05-the-other-five-modules-keep-up
    provides: "05-03's Filing, the shared Move and Copy arm in applies_to, the public file_under, and the menu agreement tests that hold the menus to applies_to"
provides:
  - "MessageCache::move_contact_between_groups and MovedBetweenGroups: one transaction, the put-in first and the take-out second, with three answers"
  - "contact_groups::Group, could_leave and could_join: the groups a contact is in and the ones it is not, exact complements from one predicate"
  - "contact_groups::moved_between, in_no_group and in_every_group: the sentence a move says and the two refusals before it"
  - "pim_command::is_not_kept_in_a_container: the refusal file_under's last arm now gives instead of reporting success"
  - "PimCommand::Move and Copy apply to a contact, and the contact context menu offers both"
  - "managers::move_a_contact_between_groups and which_of_these_groups, and the dispatcher branch that routes a contact away from the one-container path"
  - "three guard records: the transaction, the leaving filter, and the call site that asks it"
affects: [05-05, 05.1-03]

actuals:
  tokens: 121000
  tasks: 3
  commits: 6

tech-stack:
  added: []
  patterns:
    - "Two answers that are exact complements are built from one shared predicate, so the complement really is the complement and a filter cannot let a group through both ways"
    - "A refusal discovered after a write has already happened is what makes a transaction load-bearing, and it is the only kind of failure a test can reach from outside"
    - "A defect with a pure half and a call-site half is two guard records, because the break for either leaves the other's tests green"

key-files:
  created: []
  modified:
    - src/data/message_cache/contacts.rs
    - src/data/message_cache/mod.rs
    - src/application/contact_groups.rs
    - src/application/pim_command.rs
    - src/application/context_menu.rs
    - src/presentation/managers.rs
    - src/presentation/wx_context_menu.rs
    - tests/a_contact_moved_between_groups.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/KEYBOARD_SHORTCUTS.md
    - Cargo.toml
    - .planning/WINDOWS.md

key-decisions:
  - "A contact does not go through file_under at all. It has its own path in the dispatcher, because a contact's move is two membership writes over a join table and file_under is read, change, write on one row, and because the copy has to be the put-in that already ships"
  - "file_under's last arm refuses all three kinds with no container rather than only losing Contact, so 05-05 does not meet the same silent success with reminders"
  - "Action::AddToGroup is retired and the contact menu's Put in a group line raises Action::CopyItem. Two lines that both put a contact in a group would be two doors to one act"
  - "The destination chooser offers the groups the contact is not in, so in every group there is means there is nowhere to move to. The storage still handles the already in case, for a route that did not go through the chooser"
  - "No changelog entry and no version bump in task 1. Nothing reached the write until task 2, so an entry would have described something nobody could do"
  - "The break kept for the leaving filter is the widened one rather than the negated one, although the two redden identically, because the widened one is the defect the record is named for"

patterns-established:
  - "A move between two memberships is testable for atomicity only because the refusal arrives after the first write. The order that makes the failure correctable is also the order that makes the transaction measurable, which is the opposite of what 02-06 found for a different shape"

requirements-completed: []

coverage:
  - id: D1
    description: "A contact in two or more groups can be moved out of one and into another in one action, and the question asked is which group it is leaving, offered from the groups the contact is really in"
    requirement: PIM-02
    verification:
      - kind: integration
        ref: "tests/a_contact_moved_between_groups.rs#test_a_contact_moved_from_one_group_to_another_is_in_the_new_one_and_not_the_old"
        status: pass
      - kind: unit
        ref: "src/application/contact_groups.rs#test_the_groups_a_contact_can_leave_are_the_ones_it_is_in"
        status: pass
      - kind: integration
        ref: "tests/a_contact_moved_between_groups.rs#test_each_group_question_is_asked_of_the_groups_that_question_has_an_answer_in"
        status: pass
    human_judgment: false
  - id: D2
    description: "At no instant between the two writes is the contact in neither group"
    requirement: PIM-02
    verification:
      - kind: integration
        ref: "tests/a_contact_moved_between_groups.rs#test_a_contact_that_is_not_in_the_group_it_would_leave_is_not_put_in_the_other_one"
        status: pass
      - kind: integration
        ref: "tests/a_contact_moved_between_groups.rs#test_a_move_into_the_group_it_is_leaving_writes_nothing"
        status: pass
    human_judgment: false
  - id: D3
    description: "A move says which group the contact left and which it joined, and the counts of both afterwards"
    requirement: PIM-02
    verification:
      - kind: unit
        ref: "src/application/contact_groups.rs#test_a_completed_move_names_both_groups_and_both_counts"
        status: pass
    human_judgment: false
  - id: D4
    description: "Copy for a contact is the put-in that already ships, reached from the copy command rather than reimplemented beside it"
    requirement: PIM-02
    verification:
      - kind: unit
        ref: "src/application/pim_command.rs#test_a_contact_can_be_filed_although_it_has_no_one_home"
        status: pass
      - kind: unit
        ref: "src/application/context_menu.rs#test_a_contact_can_be_put_in_a_group_from_its_own_menu"
        status: pass
      - kind: integration
        ref: "tests/a_contact_moved_between_groups.rs#test_a_contact_is_filed_by_its_groups_rather_than_by_the_path_that_names_one_container"
        status: pass
    human_judgment: false
  - id: D5
    description: "A move on a contact in no group says so and writes nothing, rather than opening a chooser whose answer is thrown away"
    requirement: PIM-02
    verification:
      - kind: unit
        ref: "src/application/contact_groups.rs#test_a_contact_in_no_group_can_leave_none_of_them"
        status: pass
      - kind: unit
        ref: "src/application/contact_groups.rs#test_a_contact_in_no_group_is_told_which_command_puts_it_in_one"
        status: pass
    human_judgment: false
  - id: D6
    description: "Filing a contact through the path that names one container is refused rather than reported as done"
    requirement: PIM-02
    verification:
      - kind: integration
        ref: "tests/a_contact_moved_between_groups.rs#test_filing_a_contact_through_file_under_is_refused_rather_than_reported_as_done"
        status: pass
      - kind: unit
        ref: "src/application/pim_command.rs#test_a_kind_with_no_container_is_refused_rather_than_told_its_filing_worked"
        status: pass
    human_judgment: false
  - id: D7
    description: "Two questions for one move, a sentence naming two groups and two counts, and two choosers that read alike, all heard by somebody working by keyboard through a screen reader"
    verification: []
    human_judgment: true
    rationale: "Nothing here has been heard. Ledger 205, 206, 207, 209 and 210."
  - id: D8
    description: "A contact really moves when the key is pressed in the running program"
    verification: []
    human_judgment: true
    rationale: "The join between the key and the write is read from the source text rather than run. Ledger 208."

duration: 195min
completed: 2026-09-08
status: complete
---

# Phase 5 Plan 04: The question that made a contact's move impossible

**It works. A contact in two or more groups can be moved out of one and into
another with `Ctrl+Shift+V` or from the context menu, and copied into a second
group with `Ctrl+Shift+Y`. The contact is never in neither group, not even for
an instant. Nobody has heard any of it, and no contact has been moved by
pressing a key in the running program.**

## Performance

- **Duration:** about 195 minutes
- **Tasks:** 3
- **Files modified:** 13
- **Commits:** 6

## What works, and what does not

**Works, and was run against a real store.** A contact in Team A and Team B,
moved from A to C, is in B and C and not in A. A move where the contact turns
out not to be in the group it was leaving writes nothing at all, including the
put-in that had already run. A move into the group being left writes nothing. A
move into a group the contact is already in still takes it out of the one it
left, and the target's count does not go up. Nobody else in the group the move
emptied goes with the contact, and no group the move was not about changes.

**Works, and was run.** The two filters are exact complements from one shared
predicate, so the leaving list is the groups the contact is in and the joining
list is the ones it is not. The five sentences say what they say. `file_under`
refuses a contact rather than reporting success. The menus and the command
agree about contacts in both directions.

**Works, and was read rather than run.** The routing. `managers::pim_command`
sends a contact to the group path before the chooser that names one container,
and the two choosers are handed `could_leave` and `could_join`. Both facts are
read out of the source text by
`tests/a_contact_moved_between_groups.rs`. Nothing presses the key. Ledger 208.

**Does not work, in the sense that nobody has checked.** Nothing has been
heard. A contact's move asks two questions where every other kind asks one, the
sentence afterwards is the longest status line this program says about a single
act, and the two choosers are the same widget holding the same kind of list one
after the other. Ledger 205, 206 and 207. The contact menu's wording changed
meaning without changing words, and the two refusals name a menu line inside a
spoken sentence. Ledger 209 and 210.

**Not closed.** Criterion 2 asks for five modules and this makes four.
Reminders are `05-05`. `requirements-completed` is empty and PIM-02 is not
marked, per the plan's own instruction not to correct a requirement ahead of
the code.

## The trap, and what would have happened

`file_under`'s last arm was
`ItemKind::Mail | ItemKind::Contact | ItemKind::Reminder => Ok(id.to_string())`.
It returned success and wrote nothing. The comment above it said these are
written out rather than caught by a catch-all "so a new kind of item is a
compile error here", which is true of a new `ItemKind` variant and false of an
existing kind moving off the never-reached list. Giving a contact a move does
exactly the second thing.

The test that catches it was written before the arm changed and its failure
message is the trap in plain sight:

```
filing a contact into a group here reported success: Ok("contact-ada")
```

The arm now refuses **all three** kinds rather than only losing `Contact`. That
is wider than the plan asked and it is the same defect: Mail and Reminder were
being told their filing had worked for exactly the same reason, and `05-05` is
about to move Reminder off that list. A refusal that only covered the kind this
plan happens to touch would have left the next plan the same silence.

**The plan's premise 4 quotes the arm as `Ok(())`.** It returns
`Ok(id.to_string())` on this tree, because `05-03` made `file_under` answer with
the identifier of the row that ended up in the destination. The trap is
unchanged and the literal is not.

## A contact does not go through `file_under` at all

The plan's task 2 says the arm "loses `ItemKind::Contact`", which reads as
routing a contact through `file_under` with a real arm. It does not, and the
reason is in the plan's own must-have list.

Truth 4 says the copy for a contact must be the put-in that already ships, so
that there is one piece of code that adds a contact to a group and one set of
words for it. That code is `change_the_group_a_contact_is_in`, which has its own
chooser and its own four sentences and knows nothing about `file_under`. A move
that went through `file_under` while its twin went somewhere else would be two
families of code for two halves of one pair.

And the shapes do not meet. `file_under` is read, change, write on one row, and
it is handed one destination. A contact's move is two writes over a join table
and needs two answers, because the container it is leaving is a choice rather
than a fact. Threading a second identifier through `file_under` for the benefit
of one of six kinds would put the contact's question into the signature every
other kind reads past.

So `file_under` keeps its arm as the arm nothing reaches, and what changed is
that reaching it now says so. The dispatcher is what keeps it unreached, and
that is the wiring the source-text check watches.

## The transaction is load-bearing, and it took a second look to see why

The first design put the "is the contact really in the group it is leaving"
question before both writes. That is the obvious safe shape and it makes the
transaction **untestable**: with the check first, two loose statements and one
transaction behave identically for every input, because the only failure
reachable from outside happens before anything is written.

That is `02-06`'s finding restated. Its summary records that stamping the parent
row first "put the only reachable failure before anything was destroyed and left
no test able to tell a transaction from three loose statements".

The shape that works asks the question **with the take-out itself**, by how many
rows it removed. That is also the race-free way, and it is the reason
`change_membership`'s doc comment already gives for having four answers rather
than two: the delete ignores an absence, so a move that read first would report
success for something that did not happen and could be stale by the time it
wrote. The answer then arrives after the put-in has already run, so the put-in
has to be undone, and the transaction is the only thing that undoes it. Dropping
it rather than committing is the undo, and the comment in the code says so
because it is invisible otherwise.

```rust
if taken_out == 0 {
    // Returned without committing, so the transaction is dropped and
    // rolls back, which is what takes the put-in above away again.
    return Ok(MovedBetweenGroups::NotInTheGroupItWouldLeave);
}
```

**The order and the measurability point the same way here, which is the opposite
of what `02-06` found.** Put-in first is the order that leaves a failure
correctable, and it is also the order that puts a reachable failure after a
write. Both arguments give the same answer, and only one of them is the reason
written in the code.

## The break that is not a break

The plan says the guard break for task 1 is "the transaction becoming two loose
statements on the connection". The obvious way to write that is to run the two
statements on `self.conn` instead of on the transaction handle. **That is not a
break.** Both go through the same connection, so SQLite runs them inside the
open transaction whichever handle issued them, and every test stays green.

It was tried first, and the record says so, because the obvious way to break
atomicity here does not. The break kept is committing before the early answer,
which is the line somebody adds meaning to be safe.

## The RED halves, and how many failures each produced

**Task 1: five tests written, five red.** The naive implementation is the put-in
under a new name, which is the mistake the whole plan exists to prevent. One
prediction was wrong and worth recording: I expected
`test_a_move_leaves_every_other_contact_and_group_where_they_were` to be green
on arrival, because it is a scope test about other contacts. It went red,
because the contact staying in the group it should have left also changes that
group's membership. The test does more than it was written for.

**Task 2: ten tests written, ten red.** The two filters answer every group,
which is the state before a filter exists. The move sentence is `put_in`, which
names where the contact arrived and nothing about where it came from, so the one
fact that tells a move from a copy is missing from the only thing that reports
it. The two refusals share one sentence, which is what somebody writes when both
cases look from the inside like "there is nowhere to go". The refusal for a kind
with no container is `something_no_longer_there`, the nearest existing sentence,
which says the row has gone when it has not.

No test in either RED half was green on arrival. `05-03` reported one and this
is the same technique applied earlier.

## The tests are where they are because of what a record costs

`managers.rs` ends at 137 tests and `contacts.rs` at 103, both unchanged, as the
plan's acceptance criteria require. Between them they carry 76 guard records and
each one costs a build and a full library run to re-measure, inside the commit
gate.

**The two source-text checks are in
`tests/a_contact_moved_between_groups.rs` rather than in `tests/wired.rs`.** The
prompt dispatching this plan said the wiring check should live in `wired.rs`
with `suite = "wired"`, on the pattern of
`test_a_refused_move_is_said_where_a_refusal_is_said`. Two corrections to that.
**That test is not in `wired.rs`**; it is a library test in
`src/presentation/managers.rs` at line 4833, which is the one file this plan may
not add a test to. And `wired.rs` is named by 14 guard records against this
file's one, so putting them there would have cost fourteen re-measurements
instead of one. `wired.rs` reads source text through `what_ships`, and so does
this file; the instrument is the same and the price is not.

## Two records for the chooser, and both breaks measured

The plan's own correction of 2026-09-08 is right and the measurement confirms
it. The defect has two homes.

| break | reddens | where |
|---|---|---|
| `could_leave` returns every group | 2 | library, `contact_groups.rs` |
| `could_leave` negated to the groups it is not in | the same 2 | library, `contact_groups.rs` |
| the chooser handed `&groups` where `&leaving` was | 1 | the integration suite |

**No library test reddens for the wiring break and no integration test reddens
for the filter break**, which is what makes it two records rather than one.

Both candidates for the pure-function record redden **exactly the same two
tests**. Neither is stronger by measurement, so the choice is on the meaning: the
widened filter is the defect the record is named for and the state the code was
in before the filter was written, and the negation produces a different wrong
list. That the two cannot be told apart is written into the record, because it
says something about the tests: they are sensitive to the set being wrong at
all rather than to it being too wide.

All measured with
`WIXEN_TEST_THREADS=4 cargo test --all-targets --no-fail-fast` on a clean tree
with nothing else building, and `scripts/guards.sh --remeasure` agrees with all
three records this plan added.

## The menu, and the action that was retired

`Move` and `Copy` share one `applies_to` arm, so the word that adds a contact
adds it to both, and `test_move_is_offered_exactly_where_it_means_something` and
`test_a_copy_sits_beside_the_move_it_is_the_twin_of` then require **both** lines
on the contact menu, adjacent.

That would have put a "Copy to another group" line one row from "Put in a
group", which does the same thing. So the line that was already there raises the
copy now:

```
&New contact
Mo&ve to another group      [MoveItem]
Put in a &group             [CopyItem]
Take &out of a group        [RemoveFromGroup]
&Delete
```

`Action::AddToGroup` is retired with its mapping. **`ID_CONTEXT_ADD_TO_GROUP` is
untouched**: it is still declared, still handled, and still raised from the
contact group's own sidebar menu in `wx_app.rs`, so
`test_every_group_command_is_raised_by_something` stays green and no id is left
handled with nothing raising it. Two ids reach one handler, and they reach it
through the same function.

**The letter is `v` for the move and the copy keeps `g`.** The other three menus
use `y` for the copy, so a contact's copy is the one that does not follow. The
reason is that "Copy to another group" describes what happens to a calendar
entry and not what happens to a person, and this module already chose to say
putting somebody in and taking them out rather than adding and removing. It is a
trade and it has never been heard. Ledger 209.

## A test whose claim this plan overturns

`test_a_copy_means_something_exactly_where_a_move_does` asserted
`!PimCommand::Copy.applies_to(ItemKind::Contact)`. It was green through the RED
commit, because `applies_to` was untouched there, and went red on the widening.
Its loop, which is the guard, is unchanged; the example line was corrected and
the comment above it now says why the old reason was right about duplicating a
person and wrong about what a contact copy is.

**`test_move_is_offered_exactly_where_it_means_something` was green throughout
and was named in no red trailer**, which is what the plan's premise 11 asks. It
is an agreement test in both directions and only reddens on a half-widening.

## A test I edited after seeing it fail, and which half it was

`test_a_contact_is_filed_by_its_groups_rather_than_by_the_path_that_names_one_container`
went red in the RED commit exactly as intended and then stayed red after the
routing landed, because its anchor was wrong. It split the source on
`"PimCommand::Move | PimCommand::Copy => "` and my contact arm carries a guard,
`... if kind == ItemKind::Contact => {`, so the split landed on the arm below.

**The assertion was not touched; the anchor was.** It now reads the text between
the first and second occurrence of the shared pattern, which also asserts there
are two arms. That is a stronger anchor than the original: merging them back
into one arm now fails at the `expect` rather than passing silently. Said here
because the cheapest way to satisfy a wrong prediction is to break the test, and
knowing which half moved is the difference.

## The account question, which premise 17 marked inferred

Read rather than inferred. `wx_app::sources_for(account_id)` answers
`[account_id, LOCAL_ACCOUNT_ID]`, and both `every_group_here` and `a_group_here`
flat-map `load_contact_groups` over it.

**So a contact can be moved between an account group and a local group, and
that is deliberate rather than an accident this plan is leaving alone.** The
membership rows carry no account column: `contact_group_members` is
`(group_id, contact_id, added_at)` with a composite primary key and no foreign
key, so membership is by identifier in both directions. `change_membership` has
worked this way since groups gained members, reading contacts from
`group.account_id` and from `LOCAL_ACCOUNT_ID`. The chooser offers what a person
can already see in the sidebar, so refusing a move between two groups both
visible on the same screen would be a refusal with no reason a person could act
on. T-05-04-05 is answered by the chooser being built from `sources_for`, which
is the same set the panel draws.

## `T-05-04-04`, confirmed by reading

`groups_in` is now `every_group_here` with the members counted rather than
named, so both come from one load. `load_contact_groups` already loads
`member_ids` for every group on every call, and the three existing choosers and
the contacts panel already pay for it. This plan adds a filter over a list in
hand. The two counts in the sentence are read after the write rather than from
the chooser's list, so what is said is what the sidebar will show.

## Deviations from plan

**1. [Deviation, deliberate] No changelog entry and no version bump in task 1.**
- **Found during:** Task 1, writing the green commit.
- **Issue:** The plan puts both in task 1 and lists `Cargo.toml` in its file
  list. Nothing in the running program reached the write until task 2, so an
  entry saying a contact can be moved between groups would have described
  something nobody could do.
- **Fix:** Both went in task 2's green commit, with the wiring. It also kept
  `which-checks.sh` off the `all` path for task 1's three commits.

**2. [Deviation, deliberate] Task 1 is three commits rather than two.** The
guard record needs the break measured against a tree where the green code is
settled, and measuring it means editing the file and putting it back. Committing
green first made `git checkout --` safe. The record and the census correction are
in one commit together, which is the rule that matters.

**3. [Rule 2 - Missing critical] `file_under`'s refusal covers all three kinds
with no container, not only `Contact`.**
- **Issue:** Mail and Reminder were being told their filing had worked for
  exactly the reason Contact was, and `05-05` moves Reminder off that list.
- **Fix:** The whole arm returns `Err`. No behaviour changes for anything that
  runs, because nothing reaches the arm.

**4. [Deviation, deliberate] `Action::AddToGroup` is retired and the contact
menu's put-in line raises `Action::CopyItem`.** Covered above. The alternative
was two menu lines doing one thing, met one after the other by somebody who
cannot skim.

**5. [Deviation, deliberate] A third source-text check, and the wiring records
that go with it.** The plan budgets task 3 for the chooser. The dispatcher
routing needed one too: without it, removing the contact branch leaves a key
that does nothing and says nothing, and nothing in the tree would notice.

**6. [Deviation, deliberate] `which_group` was split rather than filtered.**
`which_of_these_groups` takes the groups; `which_group` is that with every group
somebody can see. The two questions a contact's move asks are about different
sets, and a chooser that fetched its own list could not be handed either.

**7. [Rule 1 - Bug] One guard record became unmeasurable and was re-anchored.**
- **Found during:** Task 2 green.
- **Issue:** "a refusal names the kind with the article that belongs with it"
  breaks on `capitalise(a_thing(kind))`, which named one place until
  `is_not_kept_in_a_container` used the same call for the same reason. Two
  places is a break that cannot be applied at all.
- **Fix:** The anchor carries the line above it, which only `cannot_be_moved`
  has, and it was re-measured rather than edited until it applied. It still
  reddens exactly the one test it names.

**Total deviations:** 7. One of them, the widened `file_under` refusal, is a gap
the plan's own threat register only half covered.

## Issues encountered

**The routing test's anchor**, covered above. Cost one edit and one run.

**The plan's record table is measured against `main` at `9611b70` with 632
records, and this tree started at 672.** Two of its six rows had moved:
`pim_command.rs` from 3 records to 3 but 18 tests to 22, and `context_menu.rs`
from 0 records to 1. `managers.rs` was 40 and is 42; `contacts.rs` was 34 and is
34; `contact_groups.rs` was 1 and is 1. The steer the table exists for held.

**Six records were flagged by the count check across the two green halves**, and
all six were re-measured with `WIXEN_TEST_THREADS=4` detached. Every one still
reddens exactly what it names. The scoped remedy was run each time the gate
printed it.

**The commit gate's `all` path ran once**, on task 2's green commit, because it
bumps the version. Detached, and it passed all four checks including the release
build.

## Version and ledger

0.96.0 to 0.97.0 in task 2's green commit, with the feature. Neither red commit
carries a bump, and neither does task 1's green or task 3, because none of them
changes anything a person can reach.

The ledger ended at 204 and ends at 210. Six entries, one per unrun thing:

- **205** nobody has been asked two questions for one move
- **206** the move sentence, naming two groups and two counts, has never been
  heard
- **207** the two choosers have never been told apart by ear
- **208** no contact has been moved by pressing a key in the running program
- **209** the contact menu's put-in line changed meaning without changing words
- **210** the two refusals have never been heard

## Guard records

675 records, from 672. Line 79 is 192 and line 80 went 480 to 483, and
192 + 483 = 675 holds.

Three added: the transaction that undoes a move that did not happen, the groups
a contact can leave, and the question being asked of them. All three verified by
`scripts/guards.sh --remeasure`.

`scripts/guards.sh --touched-by c7e6778` is owed to the phase-8 sweep and does
not block the merge. The three new records have never been through a sweep,
which the census now says.

## Test counts before and after

| file | records | tests before | tests after |
|---|---|---|---|
| `src/data/message_cache/contacts.rs` | 34, now 35 | 103 | 103 |
| `src/presentation/managers.rs` | 42, now 43 | 137 | 137 |
| `src/application/pim_command.rs` | 3 | 22 | 24 |
| `src/application/contact_groups.rs` | 1, now 2 | 24 | 30 |
| `src/application/context_menu.rs` | 1 | 17 | 17 |
| `tests/a_contact_moved_between_groups.rs` | 1, now 3 | 5 | 8 |

## Next plan readiness

`05-05` gives reminders a move and is the last of the five modules. Three things
it should know.

`file_under`'s last arm now **refuses** rather than reporting success, so
widening `applies_to` to `Reminder` without a real path gives a refusal somebody
can hear instead of a silent lie. That half of `05-05`'s trap is already closed.

`Move` and `Copy` still share one `applies_to` arm, so the word that adds
`Reminder` adds it to both at once, and the two menu agreement tests will then
require both lines on the reminders menu, adjacent. A reminder has no container,
so what a reminder's copy and move mean is `05-05`'s question and it has to be
answered for both commands or for neither.

The dispatcher now has a kind-guarded arm above the general one, and the
source-text check asserts there are exactly two. A reminder branch is a third,
and that check's second `expect` is what it will meet.

## The merge

Recorded after `scripts/check.sh all` and the merge, below.

## Self-Check: PASSED

Every file this summary names exists on disk and all six commit hashes are in
`git log`. No carriage returns and no em dashes in this file, in `STATE.md`, in
`ROADMAP.md` or in `WINDOWS.md`.

`tests/the_planning_files_agree_with_themselves.rs` caught two counters this
plan's own state update had left behind: `STATE.md` holds the plan number twice
and only the frontmatter half had moved, and the roadmap's progress row said
3 of 8 against four summaries on disk. Both corrected, and the suite is green.
That is the check doing exactly what `04.2-06` left it there for.

---
*Phase: 05-the-other-five-modules-keep-up*
*Completed: 2026-09-08*

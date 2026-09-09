---
phase: 05-the-other-five-modules-keep-up
plan: 08
subsystem: application
tags: [tasks, move, provider, sync, deletions, guards, repudiation]
status: complete
requirements-completed: []

requires:
  - phase: 05-the-other-five-modules-keep-up
    provides: "05-07's deleted_tasks.waiting_for_task_id, move_a_task_the_provider_holds and the question in the deletion loop; 05-06's Waiting, filed and will_have_to_be_sent; 05-03's public file_under and public tasks_sync::a_provider_holds"
provides:
  - "moving_can_be_told's task arm answering false, so no task move is refused, with the event arm untouched"
  - "file_under sending a provider-held move to move_a_task_the_provider_holds, and mapping its three answers"
  - "managers::move_what_a_provider_holds: the new identifier minted and the three answers turned into a result"
  - "managers::a_removal_will_have_to_be_sent, public: whether the provider is owed the removal of a copy it already holds, asked of the item where will_have_to_be_sent is asked of the destination"
  - "tests/a_provider_task_moves_lists.rs: ten integration tests over the local half of the move"
  - "eight tests in tasks_sync.rs beside the fake, driving the act and the syncs that carry it out"
  - "two guard records added and four corrected, all measured by hand"
affects: []

actuals:
  tokens: 28000
  tasks: 2
  commits: 5

tech-stack:
  added: []
  patterns:
    - "Two questions with two subjects kept as two functions. Whether anything is waiting is answered by the destination and by the item, and a move of a task a provider holds is the first filing where the second can be yes while the first is no. Widening the destination question to take the item would have put the wrong answer one edit away in a function whose whole argument is that it cannot see enough to give one."
    - "A refusal lifted for one kind and left standing for another, in a match arm rather than by deleting the question. The event arm still refuses for the reason the task arm used to, and a guard record stands behind the task arm going back."
    - "A record measured correctly in the morning and short by the afternoon, inside one plan. Task 2's tests all begin by making the move, so they redden task 1's record and nothing predicted it."

key-files:
  created:
    - tests/a_provider_task_moves_lists.rs
  modified:
    - src/presentation/managers.rs
    - src/application/pim_command.rs
    - src/application/tasks_sync.rs
    - tests/a_moved_task_is_in_one_list.rs
    - tests/a_copy_leaves_the_original_where_it_was.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/ALPHA_TESTING.md
    - .planning/WINDOWS.md
    - Cargo.toml

key-decisions:
  - "The task arm of moving_can_be_told is false rather than a narrower refusal. Nothing about a task makes a move untellable now, and the two candidate refusals the plan named are both about the destination, which that function is never given"
  - "cannot_be_moved keeps its wording and its signature. The sentence is right for an event, which is the only kind that reaches it; what was untrue was its doc comment, which opened by saying no provider is asked to move a task"
  - "A move into a list made on this computer is allowed, tested and reported rather than refused. It loses nothing and the person can put it right by moving the task into a synced list, and the checkpoint asks Pratik whether the program should say more"
  - "a_removal_will_have_to_be_sent is a second function rather than a wider will_have_to_be_sent, because it has a second subject and that one's seven fixtures are all about the destination being the whole of the answer"
  - "The tracer lives in tasks_sync.rs beside the fake, because push_tasks, TaskService and Provider are all private and file_under is public, so that is the one place both halves of the move are reachable at once"
---

# Phase 05 Plan 08: A task a provider holds moves lists, Summary

## Does it work

**Yes, against a script, and that qualification is the whole of what is not
known.** A task a provider holds can be moved to another list from the same menu
and the same key as a task made on this computer. The refusal is gone. The move
writes a new copy in the destination list under an identifier minted here, points
the moved task's subtasks at it, records that the provider is owed a deletion of
the old identifier, and takes the old row away, all in one transaction. The next
sync creates the copy at the provider; the sync after that sends the delete.

**Nothing here has met a real provider.** Every provider call in every test is
answered by a fake that was told what to say. That the two calls in this order
really leave the state asserted below is a claim about Google and Microsoft, and
no account has ever been used with this program. A green suite here is not
evidence about that and must not be read as any.

Version `0.100.0` to `0.102.0`. 683 guard records. Ledger 221 to 230.

## What the plan was right and wrong about

**Premise 1's stop condition did not fire, and it was checked rather than
assumed.** `05-07-SUMMARY.md` does not say what task 3 found about the Microsoft
pass, so the code was read. `sync_microsoft_tasks` gathers `held_everywhere` and
`arrived_everywhere` across the whole account and only removes anything when
`read_every_list` is true, with a comment giving the same reason the Google pass
gives: "a task moved out of one list comes back in another, and reading one list
at a time makes a move look like a deletion followed by a new task". So Microsoft
does not decide absence one list at a time and this plan is the size it says.

The other two things premise 1 asked for: the column is
`deleted_tasks.waiting_for_task_id`, holding the identifier of the copy here that
must reach the provider first. Whether the create has landed is asked as whether
that copy is still waiting to be sent, and the arrived case is told from the
genuinely-missing case by not being told apart on purpose: `find_task` answering
nothing means either the copy was renamed by `settle` or somebody deleted it, and
both mean send, for reasons written out in the loop.

**Premise 5 holds, confirmed by reading.** No new provider call was added.
`push_one` opens with `let new_here = provider.is_local(&task.id)` and calls
`google_create_task` or `ms_create_task` when that is true; `settle` calls
`cache.rename_task(&was.id, &stored)` when `new_here`. The deletion loop already
called `google_delete_task` and `ms_delete_task`. The provider half of this move
is two calls the program already makes.

**Premise 6 holds and the loop order is unchanged.** `git diff main` over
`tasks_sync.rs` touches neither `for gone in ...` nor `for task in ...`.

**The plan missed a third place the refusal was asserted.** It named
`moving_can_be_told`'s two call sites, which is right, but three existing tests
asserted the refusal and the plan named none of them.
`test_a_task_the_provider_holds_is_told_no_before_the_chooser_opens` in
`managers.rs` was found only by running the suite after the green half. It is the
chooser's half of the pair and it now asserts that the chooser opens.

## What is still refused, and what is not

**No task move is refused by `moving_can_be_told` at all.** The arm is `false`.
The two candidates the plan offered are both questions about the destination,
which that function is never given: it is asked before the chooser opens, with
only the item in hand. A task in a list this computer made and a task belonging to
the other provider are both decided later, by `push_tasks`, and neither is a
reason to refuse the act.

**The event arm is exactly as it was.** `ItemKind::Event` still answers
`event.provider_event_id.is_some()`, an event a server holds is still refused, and
the sentence it says is unchanged. Decision 1 of 2026-09-06 is about tasks.
`presentation::managers::tests::test_moving_an_event_the_provider_holds_is_refused_and_writes_nothing`
is untouched and is what holds it there.

**`file_under` branches on the same question `moving_can_be_told` asks**, with a
comment saying the two answers have to agree and what goes wrong in each
direction if they do not: one refusing what the other would move means the chooser
never opens, and the other way round means somebody answers a question and is then
told no.

## `cannot_be_moved` was narrowed where it was untrue

The sentence itself is unchanged, and that is a measured decision rather than an
omission. Read whole, every clause of it is true of an event: moving one to
another calendar is still not something this can do, nothing has been moved, and
an event made on this computer really can be moved. What was untrue was its
opening doc comment, which said no provider is asked to move a task to another
list or an event to another calendar. That is now half false, so it says which
half went and why, and it says that the arm reaching it is what holds the line
rather than the type.

`test_a_refused_move_says_it_did_not_happen_and_what_does_work` in
`pim_command.rs` asked the sentence about a task. A task is not a kind that
sentence is ever said about now, so a test fingerprinting its wording for one
would hold the shape of something nobody can hear. It asks about an event.

**No guard record named `cannot_be_moved` in its red list.** Two mention it in
prose. The record whose break is inside that function, "a refusal names the kind
with the article that belongs with it", is anchored on `capitalise(a_thing(kind))`
with the line above it and was not disturbed.

## Every field of the task survives the local move

Asserted one by one in `test_every_field_of_the_task_survives_the_move`, from a
fixture where no field is left at its default, because a field empty before and
empty after says nothing: title, description, due date, whether it was done, when
it was done, priority, order, parent, when it was made, the provider's own
progress word, and the account.

Three fields deliberately do not survive and each has its own assertion. The copy
is marked as waiting to be sent, or nothing ever creates it. It carries no
`remote_updated`, because no provider has seen this copy and a stamp for one
would make the next pull decide nothing had changed. And `updated_at` becomes now.

**What no test here asserts is that a field survives the round trip through the
provider's answer**, and that is worth saying plainly rather than leaving to be
inferred. The `Scripted` fake's `google_create_task` ignores the body it is sent
and returns a bare task, so after the create the row is rebuilt by
`google_task_to_entry` from that bare answer and the title comes back as
"Untitled task". A real provider echoes the body. The claim these tests support is
about the local write, which is where the content actually lives across the gap.
Ledger 228.

## The order, and the two syncs

The plan's correction was right and it was watched happening. `push_tasks` sends
deletions before creations. `05-07` put a question in the deletion loop that skips
a note whose copy is still waiting to be sent. So in the first sync the moved row
is still pending when the deletion loop reaches its note, the note is skipped and
counted, and only then does the create run. The delete goes in the second sync.
Each sync calls `push_tasks` once.

The effect is that the provider is asked to create before it is asked to delete,
even though the loops run the other way round. The tracer asserts both halves in
order, including that nothing was asked to be deleted during the first sync.

**PIM-01's third criterion, "never in both and never in neither", is true of this
computer and is deliberately not true of the provider.** On this computer exactly
one row is that task at every instant, held by the transaction. At the provider a
failure between the two calls leaves it in two lists and never in none. Choosing
"never in neither" over "never in both" is the whole of the safety argument: two
copies are visible to anybody who looks and correctable by anybody, and none is
visible to nobody.

## Every way it can be interrupted

| What happens | What is left | Test |
|---|---|---|
| The create is refused | No delete sent, the copy still pending, the note still owed, one problem counted | `test_a_move_whose_create_is_refused_sends_no_delete_and_keeps_both_halves` |
| Allow Changes is off | Neither call sent, the create counted as waiting on the setting and the delete as waiting on the copy, nothing reported as a problem | `test_a_move_held_by_allow_changes_is_counted_apart_from_one_held_by_the_copy` |
| The sign-in has run out | `needs_sign_in` set, nothing counted as a problem, both halves still owed | `test_a_move_that_needs_a_sign_in_is_not_reported_as_a_problem` |
| The delete is refused | The copy renamed, the note still owed, the next sync sends it and stamps it taken | `test_a_move_whose_delete_is_refused_is_finished_by_the_sync_after_it` |
| The program closes between the two calls | The next sync after it opens again sends the delete | `test_a_move_interrupted_by_the_program_closing_is_finished_when_it_opens_again` |
| The destination is a list made here | The create is never sent, the note waits, nothing is lost | `test_a_move_into_a_list_made_here_is_never_sent_and_the_note_waits` |
| Microsoft rather than Google | The same two syncs, the same two calls | `test_a_task_microsoft_holds_moves_list_the_same_way` |

The program-closing test drops a real `MessageCache` on a `tempfile` path and
opens a second one on the same path, so what the second sync reads is what the
program wrote rather than what a test arranged.

The gate-off sentence is reached rather than copied. `TaskSyncResult::summary`
calls `crate::application::allowed::changes_waiting_here(self.waiting_on_the_setting)`,
and the test asserts `said.contains(&crate::application::allowed::changes_waiting_here(1))`
rather than a third hand-written copy of those words.

## Seven tests were green on arrival, and each has a measured break behind it

Named, with the change each was measured against and what that change did.

**Six of the seven, plus the tracer, plus one of `05-07`'s, go red when the
hold-back is removed.** `if still_waiting { result.waiting_on_the_new_copy += 1;
continue; }` replaced by `let _ = still_waiting;`, measured with
`cargo test --all-targets --no-fail-fast`: 8 tests red tree-wide, being
`test_a_move_whose_create_is_refused_sends_no_delete_and_keeps_both_halves`,
`test_a_move_held_by_allow_changes_is_counted_apart_from_one_held_by_the_copy`,
`test_a_move_whose_delete_is_refused_is_finished_by_the_sync_after_it`,
`test_a_move_interrupted_by_the_program_closing_is_finished_when_it_opens_again`,
`test_a_move_into_a_list_made_here_is_never_sent_and_the_note_waits`,
`test_a_task_microsoft_holds_moves_list_the_same_way`,
`test_a_task_google_holds_moves_list_and_two_syncs_carry_it_out` and
`test_a_deletion_waiting_on_a_copy_that_has_not_gone_is_not_sent`.

**The seventh does not depend on that and needed its own break.**
`test_a_move_that_needs_a_sign_in_is_not_reported_as_a_problem` still passes with
the hold-back gone, because a refusal on permission refuses both calls either way.
Removing `Err(e) if refused_for_permission(&e) => result.needs_sign_in = true`
from the create loop reddens it and
`test_a_task_change_refused_on_permission_asks_the_person_to_sign_in`.

## The one test that was really red in task 2, and why it is a fix rather than a feature

Whether anything is waiting was answered by the destination alone. That is right
for every filing that existed before this plan. A move of a task a provider holds
is the first one that leaves something to send **wherever it goes**, because the
provider still holds its own copy in the list the task started in.

Move such a task into a list made on this computer and the two answers come apart.
`push_tasks` counts a task in such a list as `local_only` and never sends it, so
`will_have_to_be_sent` answers no and answers correctly. The move was then
announced as "Buy milk moved to Mine" with nothing said about the account, while a
deletion at the provider was owed for ever. That is T-05-08-03 in this plan's own
threat register, listed as `mitigate`, delivered by the plan that lists it.

`a_removal_will_have_to_be_sent(kind, filing, id)` is the fix, and `file_it` asks
it beside the other. Two functions rather than one wider one, because they have
two subjects: `will_have_to_be_sent` asks about the destination and its whole
argument, with seven fixtures behind it, is that the answer comes from there.
A copy answers no, because the provider's own item stays exactly where it is. Only
a task, with the other kinds written out arm by arm so that lifting the event
refusal later is a decision here rather than an inherited "nothing is owed".

The red half was today's answer said through the new shape rather than a missing
symbol: the function existed, was public, was called, and answered `false`. Under
`-D warnings` an unused function is a build failure, and a build failure is a
weaker red than an assertion, because it says a name is missing rather than that a
behaviour is.

## Guard records: two added, four corrected, all measured

### Added

**"a task a provider holds is not refused a move".** The break is
`ItemKind::Task => false` going back to
`ItemKind::Task => crate::application::tasks_sync::a_provider_holds(id)`, one word,
and the way somebody resolving a merge between this branch and anything written
before it would undo the whole plan without meaning to.

**"a deletion the provider refused is not written down as taken".** The break
moves the stamp out of the arm that runs when the provider took the deletion, so
every deletion is recorded as done whether it landed or not. It reddens exactly
one test and that test is the recovery assertion.

**The second candidate was measured and rejected, as the plan asked.** The
pending row's create not being retried, broken by clearing the flag when the
create failed, reddens three tests, of which two are older ones about a list going
and about a priority nothing understands. So that break already has guards that
predate this plan and this one had none. Kept the one with nothing else watching
it.

### Corrected, and this is the finding worth keeping

`--remeasure` over the 21 records the count check named reported four that were
short. All four are correct, none was predictable, and one of them is this plan's
own.

Three are reddened by `test_a_move_held_by_allow_changes_is_counted_apart_from_one_held_by_the_copy`:
"a count and the thing it counts agree in number", "a task change held by Allow
Changes is a wait, not a problem" and "the waiting count is said as a sentence
naming the setting". A move half done with the setting off meets both waiting
counts and the sentence that names the setting, so it names all three.

**The fourth is "a task a provider holds is not refused a move", written by task 1
of this plan and short by task 2 of the same plan.** It named 3 tests, measured by
hand and verified by `scripts/guards.sh` the same morning, with all 3 red and
nothing else. It names 10 now. Every test task 2 added begins by making the move,
so with the refusal back every one of them panics where it sets up rather than
failing on what it is about. Hours, inside one plan, with the record's own author
still working on the file.

All four corrected by hand with the reason written into each, then re-measured:
all 4 redden exactly what they name.

### The record that came out weaker

"the second question before a move is written is asked as well as the first"
guards the second `moving_can_be_told` call in `file_under`. Measured by hand
before and after, tree-wide with `--no-fail-fast`: it used to redden five tests
across three targets, four of them about a task. **It now reddens one.** A task no
longer passes through that question at all, so taking the block out changes nothing
about a task, and the event test is the whole of what would notice. That is
written into the record rather than left for somebody to rediscover as a record
that came out short.

### Counts

| | Before | After |
|---|---|---|
| `guards/guards.toml` records | 681 | 683 |
| Census, line 79 + line 80 | 192 + 489 | 192 + 491 |
| `src/presentation/managers.rs` `#[test]` | 137 | 137 |
| `src/application/tasks_sync.rs` `#[test]` | 111 | 119 |
| `src/application/pim_command.rs` `#[test]` | 38 | 38 |

**No `#[test]` was added to `managers.rs`.** Two were rewritten and one renamed,
so the count is unchanged. Every `#[test]` added to `tasks_sync.rs` in each task
landed in one commit, so the remedy was paid once per task rather than per test.

The scoped `--remeasure` the gate printed was run detached at
`WIXEN_TEST_THREADS=4`, twice. The first, 21 records, took 45 minutes and every
red list was already right. The second, 20 records, took 48 minutes and found the
four above. A third run over just those four took 10 minutes.

## Deviations from the plan

**1. [Correction] The tracer and the failure tests are in `tasks_sync.rs`, not in
one integration file.** The plan's task 1 asked for one test file driving the
sequence against the fake. It cannot be under `tests/`: `push_tasks` is private,
`TaskService` is `pub(crate)`, `Provider` is private and the fake is inside that
module's `#[cfg(test)]` block. `file_under` is public, so `tasks_sync.rs` is the
one place both halves of the move are reachable at once, and the tracer calls
`file_under` there rather than the storage write directly, so the refusal really
had to be lifted for it to pass. `tests/a_provider_task_moves_lists.rs` holds the
local half, where every field assertion lives. Nothing was widened.

**2. [Rule 2 - Missing critical] `a_removal_will_have_to_be_sent`.** Not in the
plan. It closes T-05-08-03, which the plan's own threat register lists as
`mitigate` and which this plan would otherwise have delivered.

**3. [Correction] Three existing tests asserted the refusal and the plan named
none of them.** Two in `tests/a_moved_task_is_in_one_list.rs`, one in
`managers.rs`, plus one in `tests/a_copy_leaves_the_original_where_it_was.rs`
whose name said the move could not happen. All four now assert what the refusal
was standing in for, and each says in its own comment what it used to assert and
when that stopped being true.

**4. [Correction] The event arm is not tested from `tests/`.** The plan's
behaviour list did not ask for it, but the obvious place to prove it is untouched
is beside the task tests. Building a calendar and an event fixture in an
integration file would be a second copy of one that already exists in
`managers.rs`, for an assertion that already runs. The new file's header says so
and names the test that does the work.

**5. [Deliberate] A move into a list made on this computer is allowed rather than
refused.** Argued above and put to Pratik at the checkpoint.

**6. `Cargo.lock` was left out of the first green commit** and carried by the next
one, which says so. The gate judges the working tree rather than the index, so it
could not be committed on its own with a red test in the tree.

## What is not checked

Nine ledger entries, 221 to 230. Five are the ones the plan's success criteria
named. Four were found on the way.

- **222** whether a real provider accepts a create into a second list while the
  first still holds the task, which is the state this move passes through on
  purpose
- **223** whether a real provider's list still names the old copy on the pull that
  follows the delete, and for how long. The seven-day memory is sized against a
  number nobody has measured
- **224** whether a change waiting on a create is distinguishable by ear from one
  waiting on Allow Changes
- **225** whether Microsoft's four progress words survive a move
- **226** whether the identifier a provider hands back is accepted by the same
  provider's delete for the old one, on an account signed in to both
- **227** a move into a list made here never drains, and nothing tells the person
  they can put it right by moving the task into a synced list
- **228** `google_task_to_entry` sets `created_at` to empty, so a moved task loses
  when it was made once Google names the new copy. Older than this plan and
  reachable by every task created here and synced; a provider move now goes
  through it too
- **229** no test asserts that a field survives the round trip through a provider's
  answer to the create
- **230** nobody has heard what a move of a provider-held task says

## Known limitations

**Nothing has run against a real account.** Every provider call is answered by a
fake that was told what to say. The failure this whole ordering was chosen to
produce, the second call failing after the first succeeded, has been produced only
by telling a fake to fail. That proves what this code does about it. It proves
nothing about whether Google or Microsoft behaves the way the fake was told to.

**`requirements-completed` is empty.** PIM-01's first criterion is closed as far
as anything here can close it, and the requirement is complete when a real account
has done this once.

**Nobody has heard any of it.** The sentence a move says is `05-06`'s and is
unchanged by this plan.

## Known Stubs

None. Every function added here is reached from `file_it`, which is reached from
the Move command in five modules, and from `file_under`, which the sync tests
drive end to end.

## Self-Check: PASSED

Files created, checked on disk:

- `tests/a_provider_task_moves_lists.rs` FOUND
- `.planning/phases/05-the-other-five-modules-keep-up/05-08-SUMMARY.md` FOUND

Commits, checked in `git log`:

- `821bc53` FOUND: test(05-08), the RED half of task 1
- `f9f8ac5` FOUND: feat(05-08), the GREEN half of task 1
- `e0cb2bc` FOUND: test(05-08), the RED half of task 2
- `5628c1b` FOUND: feat(05-08), the GREEN half of task 2
- `675b873` FOUND: the merge into `main`

`scripts/check.sh all` was run on the branch before the merge, detached, and
reported all four checks passed. The branch was
`a-task-a-provider-holds-moves-lists`.

`guards/guards.toml` holds 683 records, and lines 79 and 80 sum to 683.
`.planning/WINDOWS.md` reaches 230, and 215 open plus 15 fixed equals 230.
`src/application/tasks_sync.rs` holds 119 test functions;
`tests/a_provider_task_moves_lists.rs` holds 10.

No file this plan wrote holds a carriage return, measured with
`tr -cd '\r' | wc -c` rather than with grep.

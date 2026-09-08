---
phase: 05-the-other-five-modules-keep-up
plan: 03
subsystem: ui
tags: [copy, move, destinations, menus, guards]

requires:
  - phase: 05-the-other-five-modules-keep-up
    provides: "nothing. 05-01 and 05-02 had both landed and neither touched anything this opens"
provides:
  - "application::destinations::Filing: the two acts as one value, with the three answers that make a copy not a move written beside the move's own"
  - "PimCommand::Copy, and PimCommand::filing() pairing the two filing commands to their acts"
  - "pim_command::filed and cannot_be_filed_into: one sentence each taking the act, replacing moved and cannot_be_moved_into"
  - "managers::file_under, public, taking the act and returning the identifier of the row that ended up in the destination"
  - "Action::CopyItem and ID_CONTEXT_COPY_ITEM, on the event, task and note menus beside Move"
  - "tests/a_copy_leaves_the_original_where_it_was.rs: seven integration tests spanning the write and the store"
  - "three guard records: two on the copy making a new row, one on the menus agreeing with the command"
affects: [05-04, 05-05, 05.1-03]

actuals:
  tokens: 96000
  tasks: 2
  commits: 4

tech-stack:
  added: []
  patterns:
    - "A pair of acts that share one path is one enum with one const predicate per difference, each documented beside the other act's answer, rather than a bool threaded through four signatures"
    - "Behaviour that lives in managers.rs is reached from tests/ through a public function, because a #[test] in that file costs 42 guard records a re-measurement each"

key-files:
  created:
    - tests/a_copy_leaves_the_original_where_it_was.rs
  modified:
    - src/application/destinations.rs
    - src/application/pim_command.rs
    - src/application/context_menu.rs
    - src/application/tasks_sync.rs
    - src/presentation/managers.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_context_menu.rs
    - src/presentation/wx_destination.rs
    - tests/wired.rs
    - tests/theme_reach.rs
    - tests/tree_dialogs_resolve_the_row_somebody_is_on.rs
    - tests/tree_rows_leave_no_registry_entry.rs
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - guards/guards.toml
    - Cargo.toml
    - .planning/WINDOWS.md

key-decisions:
  - "The three differences are const predicates on Filing, not branches at the call sites, so each is written beside the other act's answer and neither can be changed without meeting the other"
  - "The copy stays inside the Copy to submenu rather than moving to the top level beside Move. The Action menu has four letters left, j, q, x and z, and none is a letter anybody would guess for copy"
  - "cannot_be_moved_into became cannot_be_filed_into taking the act, rather than a second sentence written beside it, because two sentences of the same shape are two things to keep in step"
  - "file_under and tasks_sync::a_provider_holds became public so the copy could be run end to end from tests/ rather than from a #[test] in managers.rs, which 42 records fingerprint"
  - "wx_destination::ask took a copying: bool and now takes Filing. The mail path below it still reads a flag, and rewriting that would be a change to mail rather than to this"
  - "Three guard records rather than the two the plan asked for. The second is not a duplicate: a record is also what couples an integration suite to the source it guards"

patterns-established:
  - "A RED half can be written as the wrong implementation rather than as a missing one: copy as the move by another name, with each named failure one of the ways that is wrong"
  - "A guard record whose break text a rename has moved is repaired to the code as it stands and then re-measured, not edited until it applies"

requirements-completed: []

coverage:
  - id: D1
    description: "A copy of an item leaves the original where it was and puts a second one in the chosen container"
    requirement: PIM-02
    verification:
      - kind: integration
        ref: "tests/a_copy_leaves_the_original_where_it_was.rs#test_a_task_copied_into_another_list_is_on_both_lists"
        status: pass
      - kind: integration
        ref: "tests/a_copy_leaves_the_original_where_it_was.rs#test_a_move_is_still_a_move_and_leaves_the_list_it_was_on"
        status: pass
    human_judgment: false
  - id: D2
    description: "A copy is an item made on this computer and makes no claim on anything a provider holds"
    requirement: PIM-02
    verification:
      - kind: integration
        ref: "tests/a_copy_leaves_the_original_where_it_was.rs#test_the_copy_of_a_task_a_provider_holds_is_this_computers_own"
        status: pass
      - kind: integration
        ref: "tests/a_copy_leaves_the_original_where_it_was.rs#test_the_copy_of_an_event_a_provider_holds_makes_no_claim_on_the_providers_own"
        status: pass
      - kind: integration
        ref: "tests/a_copy_leaves_the_original_where_it_was.rs#test_a_task_a_provider_holds_can_be_copied_although_it_cannot_be_moved"
        status: pass
    human_judgment: false
  - id: D3
    description: "Each act says which it was, and the refusal for a read-only destination says which act it refused"
    requirement: PIM-02
    verification:
      - kind: unit
        ref: "src/application/pim_command.rs#test_a_copy_says_it_was_copied_and_never_that_it_moved"
        status: pass
      - kind: unit
        ref: "src/application/pim_command.rs#test_a_copy_into_a_container_that_can_only_be_read_is_refused_saying_copy"
        status: pass
      - kind: unit
        ref: "src/application/pim_command.rs#test_a_command_that_failed_says_what_it_was_and_what_to_try"
        status: pass
      - kind: integration
        ref: "tests/a_copy_leaves_the_original_where_it_was.rs#test_a_copy_into_a_calendar_this_program_can_only_read_is_refused_saying_copy"
        status: pass
    human_judgment: false
  - id: D4
    description: "The chooser for a copy offers the container the item is already in and the one for a move does not"
    requirement: PIM-02
    verification:
      - kind: integration
        ref: "tests/a_copy_leaves_the_original_where_it_was.rs#test_the_chooser_for_a_copy_offers_the_container_it_is_in_and_the_one_for_a_move_does_not"
        status: pass
    human_judgment: false
  - id: D5
    description: "The menus and the command agree about which kinds accept a copy, in both directions, and the key follows the module"
    requirement: PIM-02
    verification:
      - kind: unit
        ref: "src/application/context_menu.rs#test_a_copy_is_offered_exactly_where_it_means_something"
        status: pass
      - kind: unit
        ref: "src/application/context_menu.rs#test_a_copy_sits_beside_the_move_it_is_the_twin_of"
        status: pass
      - kind: unit
        ref: "src/application/pim_command.rs#test_a_copy_means_something_exactly_where_a_move_does"
        status: pass
      - kind: integration
        ref: "tests/wired.rs#test_move_and_copy_follow_the_module_rather_than_always_meaning_a_mail_folder"
        status: pass
      - kind: integration
        ref: "tests/wired.rs#test_every_context_menu_line_has_a_handler"
        status: pass
    human_judgment: false
  - id: D6
    description: "A copy is told apart from a move by ear, and a chooser offering the container the item is in reads as the duplicate it is meant to be"
    verification: []
    human_judgment: true
    rationale: "Nobody has heard any of it. The two sentences differ by one participle in the middle. Ledger 200 and 201."
  - id: D7
    description: "A copied item is created at the provider as a second item"
    verification: []
    human_judgment: true
    rationale: "No push in this program has run against a real account. The local half is asserted; everything past it is untestable here. Ledger 203."

duration: 210min
completed: 2026-09-08
status: complete
---

# Phase 5 Plan 03: A copy that is not the move by another name

**It works. An event, a task or a note can be copied into another calendar, list
or folder, from `Ctrl+Shift+Y` and from the context menu, in whichever module is
open. The original stays where it is, the copy is this computer's own item
waiting to be sent, and each act says which one it was. Nobody has heard any of
it, and no copy has yet been made by pressing the key in the running program.**

## Performance

- **Duration:** about 210 minutes
- **Tasks:** 2
- **Files modified:** 17 (one created)
- **Commits:** 4

## What works, and what does not

**Works, and was run against a real store.** A task copied into another list is
on both lists afterwards, the original untouched and the copy carrying an
identifier of its own. A task Google holds copies fine and is refused a move,
which is the asymmetry that matters: the copy is an item made here, so
`tasks_sync::a_provider_holds` answers false about it and it is marked as
waiting to be sent. An event's copy carries no `provider_event_id` and no etag,
so a push cannot mistake it for the provider's own event, and the original is
still in the calendar it started in with its identity intact.

**Works, and was run.** The refusal for a calendar this program can only read
says "copied" where a move says "moved", on both halves of the sentence, and
that is one function taking the act rather than two sentences to keep in step.
So is what is said after a success. All five `PimCommand` variants now fail in
five distinguishable sentences.

**Works, and was read rather than run.** The routing. `tests/wired.rs` asserts
that both ids are answered by the module they were raised in, in one condition
above the mail arm, and that the mail arm is below it. That reads the source
text of the arm. Nothing presses the key. Ledger 202.

**Does not work, in the sense that nobody has checked.** Nothing here has been
heard. "Buy milk copied to Shopping" and "Buy milk moved to Shopping" differ by
one participle in the middle of a sentence, at whatever rate somebody's screen
reader is set to. Ledger 200. And the chooser for a copy now offers the
container the item is already in, which nobody has met. Ledger 201.

**Does not work, in the sense that it has never left this computer.** A copied
Google task has never been created at Google, because no push in this program
has run against a real account. Ledger 203.

**Not closed.** Criterion 2 asks for five modules and this delivers three.
Contacts are `05-04` and reminders are `05-05`. `requirements-completed` is
empty and PIM-02 is not marked, per the plan's own instruction not to correct a
requirement ahead of the code.

## The plan's premise 1 was half right, and the half that was wrong mattered

The plan says the chooser for a copy "exists and is unreached", and the prompt
that dispatched this said every production caller passes false. The first half
is what I checked, and it is right about the item path: `move_item` passed
`false` and nothing else asked for an item chooser at all.

The second half is not true of mail. `move_or_copy_message` in `wx_app.rs` takes
a `copying: bool` and passes it straight through, and `ID_COPY_TO_FOLDER` has
raised it since the message copy shipped. So the window's Copy title, its
`&Copy` button and its accessible name are reached today, every time somebody
copies a message into another folder. What was unreached was the copy chooser
*for an item*, which is a narrower claim and the one that was true.

It changed nothing about what got built and it is worth writing down anyway: a
premise that says a whole feature is dead code invites deleting or rebuilding
it, and this one would have been wrong about which.

## The RED half is the copy written as a move

The first attempt at a red half was the ordinary one: add the variant, leave
the sentences correct, take the behaviour red. It produced **one** failing test
out of the eleven I had written. The words were all green on arrival, because a
sentence function has no implementation to get wrong once the arm is typed.

That is a test that has never been red, which this project says proves nothing.
So the red half was rewritten as the mistake it exists to prevent: **`Filing`
carries both acts and the copy answers everywhere the move does.** One word for
both in `did`, one sentence for both in `did_not_happen`, both predicates
answering the move's answer, `applies_to` offering the copy nowhere.

Twelve failures, each on the thing it is about:

| what was wrong | failures |
|---|---|
| the original left the list it was on | 2 integration |
| the copy kept the provider's identity | 2 integration |
| a task Google holds was refused a copy | 1 integration |
| the chooser left out the list the thing is in | 1 integration |
| a copy said "moved" | 3 unit, 1 integration |
| a failed copy read as a failed move | 1 unit |
| the copy was offered nowhere | 1 unit |
| the key did not follow the module | 1 integration |

The pattern is worth keeping. **Where two acts share one path, the red half is
the naive sharing.** It compiles, it is honest about what is not written yet,
and every difference the plan identified becomes a named failure rather than an
assertion typed after the fact.

One test was still green on arrival and is reported rather than claimed:
`test_only_the_two_filing_commands_name_an_act`, which reads `PimCommand::filing`.
It is a pairing table, and a table's test is a spec. It could have been reddened
by having `Copy.filing()` answer `None`, and that was rejected because the
dispatcher would then have returned in silence for a copy, which is the
`file_under` trap this phase warns about.

## The five exhaustive matches, and where the fifth really is

The plan's corrected premise 7 is right that there are five and that the fifth
is in `managers.rs` inside `pub fn pim_command`. Its line number, 2641, is
stale: on this tree the match on `command` is at 2652, because `05-02` landed in
between. The structure is exactly as described.

The neutral arm question did not arise in the form the plan expected. The red
half routes `PimCommand::Move | PimCommand::Copy` to one arm that asks the
command which act it is, so there is no separate copy arm to make neutral. In
the red half `PimCommand::Copy` is constructed by nothing outside the tests, so
the running program never reaches it; the tests that would have needed a
refusal sentence are the ones that call `file_under` directly.

`-D warnings` did not force anything into the red commit. `PimCommand` is a
public enum in a library crate, so a variant nothing constructs builds clean.
That is the same correction `05-01` made about a public field, and it holds for
the same reason.

## Where the copy went on the Action menu, and why not beside Move

**It stayed inside the `Cop&y to` submenu, as its first item, reworded from
"another &Folder..." to "&Somewhere Else...".** `Ctrl+Shift+Y` did not move.

The symmetry argument for the top level is real: somebody who learns "Action,
Move to" expects "Action, Copy to" beside it. What ruled it out is measured
rather than argued. The Action menu claims twenty-two letters:

```
r a o f u n e s i k p d m v c w y l g b t h
```

leaving **j, q, x and z**. A top-level "Copy to..." would have to take one of
those, because `y` is claimed by the `Cop&y to` submenu on the same menu and
`test_no_two_items_on_one_menu_claim_the_same_letter` in `tests/wired.rs`
refuses a clash. Freeing `y` means renaming the submenu, and every sensible
name for "make a task, an event or a note out of this message" also needs a
letter from j, q, x and z.

So the choice was a guessable letter in a submenu, or Alt+X at the top level.
The key is the path that matters and it is unchanged; the mnemonic is the one
somebody would guess; and the three message-to-item commands keep their place
under a heading that still describes them. The submenu's own description
changed from "Keep this message and make something else from it", which stopped
being true when its first item started following the module.

**The three message-to-item commands were not touched and still work.**
`test_a_message_can_be_copied_into_the_other_modules` is green and the arm
answering `ID_CONTEXT_COPY_TO_TASK`, `ID_CONTEXT_COPY_TO_EVENT` and
`ID_CONTEXT_COPY_TO_NOTE` is unchanged, as is the mail arm below the module arm.

## The mnemonic on the item menus: y, and the plan's correction was right

`c` is free on all three item menus, as the plan's correction of 2026-09-08
says, and `test_no_two_entries_in_one_menu_share_a_mnemonic` was run and is
green.

I took **y** rather than `c`. It is the letter `Cop&y to folder` already claims
on the message menu and the one `Ctrl+Shift+Y` carries, so copying is one letter
everywhere in the program rather than two. Measured before choosing: events
claim n, v and d; tasks n, v, k and d; notes n, v, p and d. None claimed y.

## `moving_can_be_told` is not asked for a copy, and the reason is in the code

`Filing::needs_the_holder_told` answers false for a copy, and its doc comment
carries the reason rather than leaving it in this plan: a move of a
provider-held item means deleting it there, creating it here and writing the new
identity over the old, and none of that is built, while a copy leaves the
provider's item untouched and creates a local one the next push makes. The
comment says plainly that inheriting the refusal would refuse the one act that
is safe.

The other two differences are the same shape. `leaves_out_where_it_is` says why
a copy keeps the container it is in; `makes_a_new_row` says what the copy stops
being able to carry from the original. Three predicates, each documented beside
the other act's answer, so neither can be changed without meeting the other.

## The two sentences, quoted

A copy refused by a read-only calendar:

> "Term dates" is a calendar this program can only read, and an event copied
> into it could never be sent. Nothing has been copied. A calendar you can
> change can hold it.

The same refusal for a move:

> "Term dates" is a calendar this program can only read, and an event moved
> into it could never be sent. Nothing has been moved. A calendar you can
> change can hold it.

One function, one parameter, two words changed. The alternative was
`cannot_be_copied_into` written beside `cannot_be_moved_into`, and the reason
against it is the reason this file already gives for the pair being one shape:
two sentences of the same shape are one thing to learn rather than two, and two
sentences in two places are two things to keep in step.

## The ordering assertion, quoted

```rust
let pim_arm = squashed
    .find("(id==ID_MOVE_TO_FOLDER||id==ID_COPY_TO_FOLDER)&&showing!=PimModule::Mail")
    .expect("the arm for the other five modules");
let mail_arm = squashed
    .find("_ifid==ID_MOVE_TO_FOLDER||id==ID_COPY_TO_FOLDER=>")
    .expect("the arm for mail");
assert!(pim_arm < mail_arm, ...);
```

Both ids go in one condition rather than two, and that is what makes the test
red rather than a rename. Writing `id == ID_COPY_TO_FOLDER && showing != Mail`
as a second condition beside Move's would have left the old assertion's text
intact and the old test green, so the routing would have shipped with a check
that had never noticed anything. One condition also means one place for the
module question, which is the code reason and the one written in the source.

## What the plan promised about that test, and what actually happened

The plan's premise 5 says the existing test "will go red and it is right to",
and the dispatching prompt repeated it. **It would not have.** The mail arm's
text, `id==ID_MOVE_TO_FOLDER||id==ID_COPY_TO_FOLDER`, is unchanged by routing
copy per module; only the module arm gains text. So the old assertions would all
still have matched.

It went red because I rewrote it in the red commit to assert the new shape,
which is a red I wrote rather than one the change produced. Said plainly so the
next reader does not take the premise as evidence: **the routing change on its
own does not redden that test, and a test extended in the red half is the reason
it failed.**

This is the third document in this phase to be wrong in this direction about a
`tests/wired.rs` guard. The phase README already corrects one such claim about
`test_move_is_offered_exactly_where_it_means_something`.

## The gate found a check I had not run

The first attempt at task 2's red commit was refused:

```
red-commit: test_every_context_menu_line_has_a_handler failed and the commit did not name it.
```

It reads every id `wx_context_menu::command_for` hands out and requires
`wx_app.rs` to answer it, so mapping `Action::CopyItem` to `ID_CONTEXT_COPY_ITEM`
without routing the id is a menu line somebody could choose that does nothing.
I had run `application::context_menu::` and `house_style` and not `wired`.

It was named as a third failure and went green with the routing. Worth
recording as the gate doing its job: a scoped test run chosen by hand is a
guess about which checks a change can reach, and the gate's own list is not.

## Guard records: three, where the plan asked for two

**Task 1's two.** The plan asked for one, breaking the copy so it writes over
the row it read. That break exists, in `destinations.rs`, where
`makes_a_new_row` answers the move's answer for both. Measured with
`WIXEN_TEST_THREADS=4 cargo test --all-targets --no-fail-fast` on a clean tree,
it reddens exactly three tests, all in the new suite.

The second is not a duplicate and the reason is the one `CLAUDE.md` gives about
`guards/guards.toml` being what couples a suite to the source it guards.
`check.sh` reads a record's `file` to decide which commits run which suite. With
only the record on `destinations.rs`, a commit changing the write in
`managers.rs` would not have run the tests that watch the write. So the second
record breaks the write itself, removing `task.id = new_id("task")`, and
reddens two.

**Task 2's one.** Offering copy on the reminders menu, which is the kind the
command refuses and the menu is one line away from offering. It reddens one
test. Notably it does *not* redden `test_a_copy_sits_beside_the_move_it_is_the_twin_of`,
because a reminder is offered no move either, so that test steps over the kind
rather than finding a copy in the wrong place. That is the answer that had to be
measured rather than assumed.

**No library test goes red for either of task 1's breaks.** That is a coverage
finding rather than a red list to widen: `Filing` has no unit tests of its own,
and is measured through the real write instead. The write is the stronger of the
two places to measure it, and adding unit tests in `destinations.rs` would cost
six records a re-measurement each.

Line 80's census went 477 to 479 in task 1's green commit and 479 to 480 in task
2's. 669 records became 672, and 192 + 480 = 672 holds.

## A record the rename made unmeasurable, and how it was repaired

Renaming `Moved` to `Filed` moved the text that
`a refused move is said on the refusal channel rather than the status line`
breaks, so `test_every_guard_record_still_names_one_place_in_the_tree` went red
saying the record no longer names one place in the tree.

`CLAUDE.md` says that is the moment to measure the guard by hand again, not to
edit the record until it applies. Both were done, in that order: the `before`
and `after` were rewritten to the code as it now stands, a comment on the record
says why and when, and then `scripts/guards.sh --remeasure` measured it. It
still reddens exactly the one test it names.

It was named as a failure in the red commit rather than repaired there, because
the repair needs a tree where the code it points at has stopped moving.

## The scoped remeasure

Six records in one detached run with `WIXEN_TEST_THREADS=4`, and **every one
still reddens exactly the tests it names**. Three were flagged by the count
check because `pim_command.rs` went from 18 tests to 22; one is the repaired
record above; two are the new ones being recorded for the first time.

```
scripts/guards.sh --remeasure \
  "a refusal names the kind with the article that belongs with it" \
  "a confirmed delete that finds nothing to delete still says so" \
  "a command that failed says what it was and what to try" \
  "a refused move is said on the refusal channel rather than the status line" \
  "a copy makes a new row rather than writing over the one it read" \
  "a copied task is given an identifier of its own"
```

Task 2 added two tests to `context_menu.rs` and the count check did not fire,
because no record named that file. It names one now.

## Test counts before and after

**No `#[test]` was added to `managers.rs` (137 before, 137 after) or `wx_app.rs`
(199, 199)**, which 42 and 47 records fingerprint. That is what the new
integration file is for.

| file | records | tests before | tests after |
|---|---|---|---|
| `src/application/pim_command.rs` | 3 | 18 | 22 |
| `src/application/context_menu.rs` | 0, now 1 | 15 | 17 |
| `src/application/destinations.rs` | 5, now 6 | 37 | 37 |
| `src/presentation/wx_destination.rs` | 4 | 6 | 6 |
| `src/presentation/managers.rs` | 41, now 42 | 137 | 137 |
| `src/presentation/wx_app.rs` | 47 | 199 | 199 |
| `tests/wired.rs` | 14 | 69 | 69 |
| `tests/a_copy_leaves_the_original_where_it_was.rs` | new, 2 | none | 7 |

**The plan's premise 9 table was measured against `main` at `9611b70` and four
of its seven rows had moved.** It says `destinations.rs` 1, `wx_destination.rs`
2, `managers.rs` 40, `wx_app.rs` 42 and `tests/wired.rs` 10; on this tree they
were 5, 4, 41, 47 and 14 before anything was added. `pim_command.rs` at 3 and
`context_menu.rs` at 0 were still exactly right, and those were the two the plan
steered the tests towards. The steer held; the prices attached to the files it
steered away from had all gone up.

## Two things made public, and what that cost

`presentation::managers::file_under` and
`application::tasks_sync::a_provider_holds` were both narrower than public and
are not now.

`file_under` because the behaviour worth testing here is about rows rather than
windows, and the only other home for a test of it is a `#[test]` in
`managers.rs` at 42 records of re-measurement. Its doc comment says so.

`a_provider_holds` because the copy's whole contract is that it answers false
about the new row. Asserting the identifier prefix by hand in the test instead
would have been asserting the rule that function owns, which is the thing the
plan's key link warns about: somebody could change the prefix rule and the test
would go on passing.

Both widenings are real API surface and both are named here rather than left to
be discovered.

## `new_id` and two copies in one tick

`new_id` is a prefix and the nanoseconds since the epoch, so two rows minted
inside one clock tick collide. One copy per key press is nowhere near that, and
the same function already mints every new item this program makes, so it is not
a new risk. It becomes one the day a copy acts on more than one selected row,
which is where the collision would be a silent overwrite rather than an
impossibility.

## Deviations from plan

**1. [Deviation, deliberate] `wx_destination::ask` takes `Filing` rather than
`copying: bool`.**
- **Found during:** Task 1, designing `Filing`.
- **Issue:** The plan leaves the `bool` alone. Introducing `Filing` one call
  away from a `bool` meaning the same thing is two representations of one idea.
- **Fix:** `ask`, `build_destination_dialog` and `heading` take `Filing`. The
  button's label and its accessible name now come from one call rather than two
  literals, so it cannot say one word and be announced as another. Six call
  sites in three integration test files were updated; none added or removed a
  `#[test]`.
- **What was left alone:** the mail path below `ask` still reads a flag.
  Rewriting `spawn_folder_move` and its two sentence sites is a change to mail,
  and this plan must not change mail's behaviour. The conversion happens at the
  one call, with a comment saying so.

**2. [Deviation, deliberate] `moved` and `cannot_be_moved_into` were renamed and
given the act as a parameter rather than gaining copy twins.**
- **Issue:** The plan says "a copy needs its own wording or a parameter; decide
  which and say why". A parameter, and the why is in this summary and in the
  code.
- **Verification:** Four tests over both acts, plus the integration test that
  reaches the refusal through a real read-only calendar.

**3. [Deviation from the plan's task split] `Action::CopyItem` and
`ID_CONTEXT_COPY_ITEM` are task 2's, so task 2 touched `wx_app.rs` and
`wx_context_menu.rs`, which its file list does not name.**
- **Issue:** The action needs an id and the id mapping is an exhaustive match,
  so the three arrive together. Task 1's file list names `wx_app.rs` and task
  2's does not.
- **Fix:** They went in task 2, which is where the menus are, so task 1 is the
  Action menu and the key and task 2 is the context menus. The alternative put
  a menu action with no menu in task 1.

**4. [Rule 2 - Missing critical] The copy clears more of the provider's identity
than the plan names.**
- **Found during:** Task 1 green, reading `CalendarEventEntry`.
- **Issue:** The plan says a copy "mints a new id" and that this makes it
  locally made. True for a task, whose identity is its identifier. **Not true
  for an event**, which carries `provider_event_id` in a column of its own. A
  copy keeping it would be pushed as an update to the provider's own event, and
  the original would be the one that moved. T-05-09 exactly.
- **Fix:** An event's copy clears `provider_event_id`, `etag`,
  `last_modified_remote`, `last_synced_at`, and both columns that pair a moved
  day with its series. A task's clears `remote_updated` and `remote_status`
  beside the identifier.
- **Verification:**
  `test_the_copy_of_an_event_a_provider_holds_makes_no_claim_on_the_providers_own`,
  which also asserts the original still holds its own.

**5. [Deviation, deliberate] Three guard records rather than two.**
- **Issue:** The plan prices one per task. A record on the decision alone leaves
  the write in `managers.rs` uncoupled from the suite that watches it.
- **Fix:** A second record on the write. Both measured separately by hand.

**Total deviations:** 5. One of them, the event's provider columns, is a real
gap in the plan's own threat mitigation. The rest are design choices with the
reason written down.

## Issues encountered

**The red gate refused a commit for a check I had not thought to run**, covered
above. The recovery cost one rewritten commit message and nothing else, because
nothing had been committed.

**The plan was written against `main` at `9611b70`, version 0.75.0, with 632
records. The tree was at 0.94.0 with 669.** Its per-file record table had drifted
in four of seven rows, unlike `05-01`'s and `05-02`'s experience where only the
totals moved. The rows that drifted are the ones phases 4 and 4.2 and this
phase's own first two plans added records against.

**The commit gate's `all` path ran twice**, because both green commits bump the
version and a `Cargo.toml` change makes `which-checks.sh` answer `all`. Both
were run detached. Warm, each was around six minutes including the release
build, which is above the 311 to 353 seconds `CLAUDE.md` quotes; that figure is
warm and this tree has grown.

## Version and ledger

0.94.0 to 0.95.0 in task 1's green commit, with the copy. 0.95.0 to 0.96.0 in
task 2's, with the menu lines. Neither red commit carries a bump: neither
changes anything a person can reach.

The ledger ended at 199 and ends at 204. Five entries, one per unrun thing:

- **200** the two sentences have never been heard, and they differ by one
  participle
- **201** nobody has met a chooser that offers the container the item is in
- **202** no copy has been made by pressing a key in the running program
- **203** no copied item has reached a provider
- **204** two of the three differences have tests and no guard record

## Next plan readiness

`05-04` and `05-05` widen Move to contacts and to reminders. Both now widen Copy
at the same time or fail a test: `test_a_copy_is_offered_exactly_where_it_means_something`
and `test_a_copy_sits_beside_the_move_it_is_the_twin_of` hold the menus to
`PimCommand::Copy.applies_to`, which holds itself to `Move.applies_to`. So
widening Move alone reddens three tests rather than one. That is a stronger
coupling than either plan was written against and they should know about it.

`05.1-03` gives notes their sync columns and comes through `file_under`. The
copy branch there says so and clears nothing today because there is nothing to
clear.

Two things are owed and neither blocks a merge.
`scripts/guards.sh --touched-by 5cdbe7e` belongs to the phase-8 sweep, and the
three new records have never been through a sweep, which the census now says.

## The merge

`scripts/check.sh all` passed on the branch at `9a09576`: formatting, clippy
with `-D warnings`, the whole suite and the release build. Merged to `main` at
`40495fe` with a merge commit. `main` is at version `0.96.0` with 672 guard
records, and it is **not pushed**.

The four task commits and the docs commit:

1. `3b9f936` RED, the copy written as the move by another name, twelve named
   failures plus two house-style checks
2. `41319e3` GREEN, the three differences, the write, the routing, two guard
   records
3. `23ee544` RED, the menu agreement in both directions, three named failures
   including one the gate found
4. `8fa1a81` GREEN, the copy line on three menus, one guard record
5. `9a09576` the summary, the state and five ledger entries

## Self-Check: PASSED

Every file this summary names exists on disk, and all four commit hashes are in
`git log`. No carriage returns and no em dashes in this file.

---
*Phase: 05-the-other-five-modules-keep-up*
*Completed: 2026-09-08*

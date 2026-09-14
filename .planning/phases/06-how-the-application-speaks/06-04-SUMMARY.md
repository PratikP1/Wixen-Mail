---
phase: 06-how-the-application-speaks
plan: 04
status: complete
subsystem: ui
tags: [permissions, allow-changes, per-account, account-manager, guards, census, accessibility]

requires:
  - phase: 01-folders-and-conversations
    provides: "The mirror guard `test_every_setting_somebody_can_change_is_offered_by_a_screen` and the entry it found: `allowed_per_account`, stored, read, honoured and offered by nothing"
  - phase: 07-installing-updating-and-what-is-stored
    provides: "07-05's warning that emptying `STORED_AND_OFFERED_BY_NOTHING` disarms the guard watching it, and the hand-named companion pattern this plan's fourth companion follows"
provides:
  - "Three boxes on the account edit dialog's connection page under 'Allow Changes for this account', one per answer in `Allowed`, each built with a real label, each showing what `allowed_for` answers for this account, each unavailable where Settings has the answer off"
  - "`AppConfig::set_allowed_for`: the one writer of `allowed_per_account`, keeping what an account narrows rather than the answer as ticked, and no row for an account that narrows nothing"
  - "`allowed_per_account` in `OFFERED_BY_ANOTHER_SCREEN` pointing at the account manager; `STORED_AND_OFFERED_BY_NOTHING` empty with its guard retired and the next entry's debt written where the list is"
  - "The fourth hand-named companion, `test_what_one_account_may_change_is_offered_by_the_account_manager`, asserting built, from the real fields, showing the stored answer, unavailable where off, and read back"
  - "`docs/ALPHA_TESTING.md` describes the control; the guard that forbade a page from saying a permission could be set per account is retired"
affects: [06-06, 06-07, 08, version-2]

actuals:
  tokens: 11200
  tasks: 3
  commits: 4

tech-stack:
  added: []
  patterns:
    - "A per-account writer keeps what the account narrows, not the answer as ticked: a field that Settings has off for every account is kept as 'not narrowed here', so the account follows Settings when that is turned on later, and an entry that narrows nothing is removed rather than stored"
    - "Two labels rather than one label and a greyed box: where a control is unavailable, its own name says why and names the heading to go to, because a greyed box says 'unavailable' on every channel and 'why' on none"
    - "A note a screen cannot express with its controls is a `StaticText` on both channels, and a warning about a known gap goes into the product beside the control, not only into the changelog"
    - "A guard over an exception list is retired with the list's last entry and the retirement is written where the list is, with what the next entry owes, so a green run over nothing cannot be mistaken for a check"

key-files:
  created: []
  modified:
    - src/data/config.rs
    - src/presentation/wx_account_manager.rs
    - src/application/allowed.rs
    - src/presentation/first_run.rs
    - tests/account_edit_protocol_fields.rs
    - tests/house_style.rs
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - guards/guards.toml
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "All three answers per account, decided by Pratik on 2026-09-13: the same three the Permissions tab offers, for one account, on the account edit dialog"
  - "How phases 6 and 7 share the tree: moot, because phase 7 finished on 2026-09-12 with all nine plans merged before this plan started; there was nothing to share with"
  - "The guard watching the emptied list is retired, not given a companion, because there is nothing to record and a companion over an empty list would guard a mechanism with no subject; the debt is written on the list itself"
  - "A box that was unavailable records no narrowing, so an account follows Settings when the answer is later turned on for every account, rather than staying off with nothing saying why"
  - "The sync's refusal sentence keeps naming Settings alone, and the gap is said in the product beside the boxes; naming both places is pinned literally in tests across four modules and is a plan of its own"
  - "No replacement test was written for the retired document guard, and no test was written to keep a count level; the count moved and the remeasure was paid"

requirements-completed: [FEEDBACK-01]

coverage:
  - id: D1
    description: "A per-account answer a screen writes can only ever narrow the application-wide one, and an answer no different from it writes no row"
    requirement: FEEDBACK-01
    verification:
      - kind: unit
        ref: "cargo test --lib -- data::config::permission_tests::test_a_screens_answer_for_one_account_can_only_ever_narrow data::config::permission_tests::test_an_answer_the_same_as_the_applications_writes_no_entry data::config::permission_tests::test_a_box_that_was_unavailable_records_no_narrowing"
        status: pass
    human_judgment: false
  - id: D2
    description: "The account edit dialog builds the three boxes from the real fields, shows the stored answer, disables where Settings has it off, and reads them back into the one writer"
    requirement: FEEDBACK-01
    verification:
      - kind: unit
        ref: "cargo test --lib -- data::config::every_setting_is_acted_on::test_what_one_account_may_change_is_offered_by_the_account_manager"
        status: pass
      - kind: integration
        ref: "cargo test --test account_edit_protocol_fields"
        status: pass
    human_judgment: false
  - id: D3
    description: "Each box says which of the three it is, that it can only narrow, and where it is off for every account says so and names the heading in Settings"
    requirement: FEEDBACK-01
    verification:
      - kind: unit
        ref: "cargo test --lib -- presentation::wx_account_manager::tests::test_a_permission_box_says_which_of_the_three_it_is_and_that_it_can_only_narrow"
        status: pass
    human_judgment: false
  - id: D4
    description: "`allowed_per_account` is offered by another screen and that claim is checked; the list of settings offered by nothing is empty and nothing iterates it blind"
    requirement: FEEDBACK-01
    verification:
      - kind: unit
        ref: "cargo test --lib -- data::config::every_setting_is_acted_on::"
        status: pass
    human_judgment: false
  - id: D5
    description: "Three boxes and a note, met in order by keyboard, heard with a screen reader; whether a person tabbing past a disabled box learns why it is missing"
    requirement: FEEDBACK-01
    verification: []
    human_judgment: true
    rationale: "Windows skips a disabled control in the tab order and the note beneath is not in it either. Only a listening pass can say whether the section heading and the two remaining boxes carry it. WINDOWS.md 375. Waits for the pass after phase 8"
  - id: D6
    description: "Unticking a box holds back a real send, delete or sync for that account"
    requirement: FEEDBACK-01
    verification: []
    human_judgment: true
    rationale: "None of the paths this permission governs has ever run against a real account. WINDOWS.md 376"

duration: "about 55 minutes of work and 40 minutes of guard re-measurement, 2026-09-14 05:34 to 06:26 UTC"
completed: 2026-09-14
---

# Phase 06 Plan 04: One account can be allowed less than the others, never more, Summary

A permission the program has honoured per account for some time has its first
screen: three boxes on the account edit dialog, one per answer in `Allowed`,
each able only to narrow what Settings allows for every account, each
unavailable and saying why where Settings has that answer off. The exception
list that recorded the setting as offered by nothing is empty, the guard that
watched it is retired rather than left green over nothing, and the check that
forbade a page from describing the control is gone so the testing page does.
Version 0.122.0, merged into `main` at `06aa8765`, nothing pushed. Nobody has
heard any of it, and nothing it governs has met a real server.

## The checkpoint, answered before this plan started

Both questions the plan opens with were settled before execution and are
recorded here as the plan's acceptance criteria require.

**How much of Allow Changes one account gets: all three.** Pratik answered on
2026-09-13, the recommended option. The same three answers the Permissions tab
offers, for one account, on the account edit dialog. Whatever the shape, the
control can only narrow: `allowed_for` ends in `.and(self.allowed_changes)`, so
an account can be told to do less than the application default and never more.
The wording says that, and a box for something switched off application-wide
reads as unavailable rather than as an offer. Because the answer is three, the
criterion about what the model can hold that no screen reaches does not apply:
every field of `Allowed` has a box.

**How phases 6 and 7 share the tree: moot.** Phase 7 finished on 2026-09-12,
all nine plans merged, before this plan started. There was nothing to share
with, and no conflict in `tests/house_style.rs` or anywhere else, because
07-06 had already landed. Recorded as the question dissolving rather than as an
answer chosen.

## What landed

**Task 1, the control.** `src/presentation/wx_account_manager.rs` builds a
section headed `── Allow Changes for this account ──`, through
`SETTINGS_SECTION` rather than typed, on the connection page after the
Settings section and before the directory fields. Three boxes follow, each
built through `cb_with_description`, which carries the label on the control
itself so the name reaches UI Automation and sets the MSAA name from the same
string. The labels and their Alt keys:

| Box | Label where Settings allows it | Alt |
|---|---|---|
| `allow_mail_here` | Send and delete mail from this account, where Settings allows it | M |
| `allow_personal_information_here` | Change tasks, contacts and calendar events from this account, where Settings allows it | K |
| `allow_reading_here` | Fetch message text for this account when it is not already stored, where Settings allows it | X |

Where Settings has the answer off for every account, the box is disabled
through `enable(allowed_everywhere)` and its label becomes, for mail, "Send and
delete mail from this account: off for every account under Allow Changes in
Settings, so it cannot be turned on here", naming `SETTINGS_SECTION` for the
two changes and `READING_SECTION` for the read, because the reading box in
Settings sits under a different heading and a sentence sending somebody to a
heading has to name the one they will find. Each box shows what `allowed_for`
answers for this account today, through `what_this_account_may_change`, which
reads the stored settings the way the directory fields do; a new account with
no id yet shows the application-wide answer. A `StaticText` beneath, on both
channels, says the one thing three boxes cannot: this account can be allowed
less than Settings allows, never more, and if a sync says to turn on Allow
Changes in Settings and it is already on there, a box here is what is holding
it.

On OK, `show_edit` reads the three boxes into `Allowed { mail: w.allow_mail_here.get_value(), ... }`
and hands it to `remember_what_this_account_may_change`, which loads the
stored settings, calls `AppConfig::set_allowed_for`, saves, and says out loud
at high priority if the save failed, the way the directory fields do.

**`set_allowed_for`, the one writer.** It keeps what this account narrows, not
the answer as ticked: for each field, `answer || !everywhere`, so a box that was
unavailable, which reads as unticked, is kept as "not narrowed here". An entry
that narrows nothing is `Allowed::EVERYTHING` after that and is removed rather
than stored. Three tests specify it: an answer the same as Settings writes no
row and an account set back to the same loses its row; a screen's answer for
one account cannot even record a widening; a box that was unavailable records
no narrowing, so the account follows Settings when mail is later turned on for
every account rather than staying off with nothing saying why. That last one is
the decision worth reading, because the other choice is defensible too: keeping
the unticked box as a refusal is the safer end, and it was rejected because
nobody was asked, and an account silently off after the person turned on the
one switch they could see is the shape of defect this whole plan exists to
close. `allowed_for` is unchanged; its diff against `main` is empty, and the
one line of it in the diff is context beneath the new writer.

**Task 2, the lists.** `allowed_per_account` moved from
`STORED_AND_OFFERED_BY_NOTHING` into `OFFERED_BY_ANOTHER_SCREEN` as
`("allowed_per_account", "src/presentation/wx_account_manager.rs")`, with a
comment in the voice of its neighbours. Moved and not deleted, and the reason
was traced rather than accepted: `wx_settings.rs` names `allowed_per_account`
zero times, so deleting the entry would have failed the mirror guard, which
reads the settings screen alone. The empty list now reads:

```rust
    const STORED_AND_OFFERED_BY_NOTHING: [&str; 0] = [];
```

with its doc comment saying that the guard which watched it is retired with
its last entry and has to come back with the next one, and how to write it.

## What the guard retirement actually required

Of the two honest answers the plan offered, the test was retired. A companion
planting a name in a copy of the list would keep a mechanism armed for a
subject that does not exist, and the mechanism is three lines long: whoever
adds an entry writes the walk back, and the list's doc says so and names
`what_ships_in` as the reading to use.

Retiring it cost more than deleting one function. `what_every_screen_ships`
and `EVERY_SCREEN` existed only for that test, and under `-D warnings` a
helper with no caller in a test module is a build failure, so both went with
it. A comment stands where the test was, pointing at the list's doc. The
mirror guard's `excepted` chain still includes the empty list, so the next
entry has a slot.

It was taken red by hand first, which is the plan's "first red is free" and
which could not be in a commit, for a reason the plan did not foresee: the
guard reddens only when a screen's shipping half names the setting, which is
the green code, and the plan lands every `config.rs` change in one commit,
which is the commit that retires it. So the green `config.rs` was parked in
the scratchpad, the red commit's `config.rs` was checked out, and the guard
was run against the new control:

```
test data::config::every_setting_is_acted_on::test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing ... FAILED
1 setting(s) are recorded as offered by nothing and a screen now offers them, which is good news: take them out of STORED_AND_OFFERED_BY_NOTHING:
  allowed_per_account
test result: FAILED. 7 passed; 1 failed
```

The seven that passed include the new companion against the green control.
Then the green `config.rs` came back. The acceptance criterion asking for this
in the red commit is therefore not met as written, and could not be by any
commit; it is met by this measurement.

**The document guard.** `test_nothing_offers_a_setting_per_account_that_no_screen_writes`
is gone from `tests/house_style.rs`, with `A_CONTROL_NO_SCREEN_WRITES` and
`a_line_and_the_one_after_it`, which only it called. A comment stands where it
was. It had no companion proving its reading could see a planted phrase,
unlike most of its neighbours in that file, and that is said in the comment
and here. No replacement was written: the two pages that now describe the
control are read by the document guards that already exist, and a guard whose
only purpose was to keep a number level would be the opposite of what the
number is for. The count moved and the remeasure was paid.

## The four commits

| Commit | What |
|---|---|
| `d879b3d0` | `test(06-04)`: the red half. Six trailers: three writer tests, the companion, the wording test, and the count check with its reason. The red carried the control in its plausible wrong shape, three boxes from a fixed value, enabled, read into nothing, because the label function and its constants would otherwise have been dead code under `-D warnings` |
| `00bf22ef` | `feat(06-04)`: task 1 and the `config.rs` half of task 2, in one commit as the plan asks. Version 0.122.0, changelog, twelve records re-measured |
| `50dbba2a` | `test(06-04)`: the document guard retired, the testing page, twenty-one records re-measured |
| `06aa8765` | the merge into `main` |

Task 1 and the list half of task 2 share a commit because they cannot be
apart: the control names the setting, the old guard fails the moment it does,
and the guard lives in the file the plan says to touch once. The retirement
of the document guard and the page are their own commit, so each task has one.

## The guard records, measured not predicted

`config.rs` is named by six records, not the five the plan counted, and
`tests/house_style.rs` by twenty-one, not nineteen; `wx_account_manager.rs` by
six. Taken with a TOML reader at `62f2cfb4`, 755 records in the file. Every
record was re-measured after the tree it judges was final, and every one still
reddens exactly what it names:

| File | Records | Count moved | Reading |
|---|---|---|---|
| `src/data/config.rs` | 6 | 63 to 66 | four tests added, one retired |
| `src/presentation/wx_account_manager.rs` | 6 | 13 to 14 | the wording test |
| `tests/house_style.rs` | 21 | 71 to 70 | the count check's reading; a grep for `#[test]` says 80 to 79, and the plan said 69 |

Thirty-three records, two runs of the remedy the commit printed, about
forty minutes together with nothing else building. No record came out short.
No new record was added and the census at `guards/guards.toml` 79 and 80 is
untouched: the "cannot widen" test the plan asks for already existed as
`test_an_account_cannot_be_wider_than_the_setting`, found by reading
`permission_tests` rather than accepting the plan's absence, so the new test
proves the writer cannot record a widening instead of duplicating it.

## What the gate selected for each file

Read off the three hook runs.

| File | The gate ran |
|---|---|
| `src/data/config.rs` | `--lib data::config::` |
| `src/presentation/wx_account_manager.rs` | `--lib presentation::wx_account_manager::`; no coupled `--test` suite |
| `src/application/allowed.rs` | `--lib application::allowed::` |
| `src/presentation/first_run.rs` | `--lib presentation::first_run::` |
| `tests/account_edit_protocol_fields.rs` | `--test account_edit_protocol_fields` |
| `tests/house_style.rs` | `--test house_style` |
| `docs/changelog.md`, `docs/ALPHA_TESTING.md`, `guards/guards.toml`, `Cargo.toml`, `Cargo.lock` | nothing of their own; the four tree-reading guards ran on every commit and read the two pages |

`scripts/check.sh all` on the branch at `50dbba2a`: green, 7,594 tests, no
failures, 275 seconds, release build included, under 1.98.1. Ledger 374's
keyring race did not fire this time.

## The inherited item, read clause by clause

`ROADMAP.md` phase 6, inherited from phase 1: "A permission per account is
stored, read by `allowed_for`, honoured out to the provider clients, and
offered by no screen." Stored: still, unchanged. Read by `allowed_for`: still,
unchanged. Honoured out to the provider clients: still, and never against a
real one. Offered by no screen: no longer, and the claim that it is offered is
checked by `test_a_setting_said_to_be_offered_elsewhere_really_is`. **Closed
structurally.** The mirror guard that found it, 01-06's, still passes and
would find the next one.

## Which assertions prove structure and which only a person can settle

Structure, proved by a test: the three boxes exist with labels on the
control; each is enabled exactly when the stored Settings answer allows it;
the labels name the account and carry an Alt key; the unavailable wording says
so and names the right heading; the values shown are `allowed_for`'s; the
values are read back into `set_allowed_for`; the writer narrows and never
widens and writes no redundant row; the exception move is checked against the
screen it names.

Experience, which only a listening pass can settle, ledgered: whether the
labels read well in order; whether a person tabbing through, who never lands
on a disabled box, learns why the third one is missing; whether the note
beneath is met at all by somebody working by Tab. `WINDOWS.md` 375.

One honest bound on the live test: it proves consistency with the machine's
own stored settings, and on this machine Settings allows everything, so it
exercised the offered arm of all three boxes and never the unavailable one.
The unavailable arm is proved by the unit test on `permission_box_label` and by
no live box. `WINDOWS.md` 378 says what the fix is.

## Deviations from Plan

**1. The red commit could not name the retired guard.** Above, under what the
retirement required. Taken red by hand and quoted instead.

**2. The "cannot widen" test already existed.** `test_an_account_cannot_be_wider_than_the_setting`
in `permission_tests` does exactly what the plan's acceptance criterion
describes. Not duplicated; the new `test_a_screens_answer_for_one_account_can_only_ever_narrow`
proves the writer instead. The plan's premise that it needed writing was an
absence claim, and it was traced.

**3. The red carried the control.** The label function and its three
constants had no caller outside tests, and clippy runs with `-D warnings` on
every commit, so the red commit would have failed on dead code. On 06-03's
precedent the red carries each site in its plausible wrong shape: three boxes
built from `AppConfig::default().allowed_changes`, every one enabled, read
into nothing, which is the defect the companion was written to catch, and it
caught it.

**4. Two comments that asserted the absence were corrected.** [Rule 1, a
sentence made false.] `changes_waiting_here`'s doc and its test's comment in
`src/application/allowed.rs`, and the first-run test's comment in
`src/presentation/first_run.rs`, all said nothing writes `allowed_per_account`.
Reworded to say what is true now and why the sentence still names Settings.
Comment-only, no test count moved, in the green commit. The sentences already
in the tree saying "Allow Changes is off for this account", in
`carddav_sync.rs` and `address_book_source.rs`, pointed at nothing until now
and are left as they are.

**5. The live test gained checks in the green commit only.** Additions to
`tests/account_edit_protocol_fields.rs` do not compile against the red tree,
because the fields do not exist, and a compile failure is not a red the gate
can name. Said rather than passed off as test first.

**6. The plan's counts were wrong and were re-taken.** Six records on
`config.rs` not five, twenty-one on `house_style.rs` not nineteen, and
`house_style.rs` holds 71 tests by the count check's reading, not 69.

**7. One document-guard exception the plan did not foresee, avoided rather
than met.** The `new installation` guard in `house_style.rs` reads product
pages for claims about what a fresh installation allows. The changelog entry
and the testing page were worded not to make one, and both passed on the first
run; no allowance was added.

**8. `roadmap update-plan-progress` was not run**, on the standing note that it
is broken; `ROADMAP.md` and `STATE.md` were edited by hand and the diff read.
Reading the diff was not enough: the progress table's row for phase 6 still
said 3/8, and `test_the_roadmap_counts_the_files_that_are_on_disk` refused the
docs commit until it said 4/8. A guard nobody had to remember caught what
reading had missed, which is the argument this project keeps making for them.

**9. The docs commit went through plain `git commit`**, not the SDK's commit
wrapper, because the wrapper's own timeout has twice killed this project's
hook mid-run and left an index lock, and the docs-only gate takes minutes.

## Threat register

T-06-13, a control that appears to let one account do more: `allowed_for`
unchanged, the writer cannot record a widening, the box is disabled and says
why. T-06-14, a screen showing an answer the program does not obey: the
companion asserts shown from `allowed_for` and read back into
`set_allowed_for`. T-06-15, a settings file with an entry nobody can see: every
entry now has a box, and the move is checked against the screen. T-06-16, a
guard left green and blind: retired in the same commit, taken red first.
T-06-SC: `Cargo.toml` changed by the version line only and `Cargo.lock` by the
one line cargo rewrote for it; the diff was read.

## Ledger

`WINDOWS.md` 375 to 378, both halves at 378: the boxes unheard, the paths
unrun against a real account, the refusal sentence that still names Settings
alone, and the live test's one arm.

## No tracked file was edited by a script

Read then Edit or Write, throughout. `cargo fmt` ran on the tree, which is
the formatter the hook checks against, and `cargo update -p wixen-mail`
rewrote the one lockfile line for the version. The exception set is zero.

## Self-Check: PASSED

Files named above exist on `main`; commits `d879b3d0`, `00bf22ef`, `50dbba2a`
and `06aa8765` are in `git log`; the whole gate was green at the branch tip;
`git diff` between `main` and the branch after the merge touches only
`.planning/WINDOWS.md`, which the ledger entries added afterwards.

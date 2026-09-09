---
phase: 05-the-other-five-modules-keep-up
plan: 06
subsystem: ui
tags: [tasks, move, allow-changes, feedback, guards, requirements]

requires:
  - phase: 05-the-other-five-modules-keep-up
    provides: "05-03's Filing, its public file_under and public tasks_sync::a_provider_holds, and pim_command::filed taking the act; 05-04's file_under refusal for the three kinds with no container; 05-05's reminder filing path, which also says filed"
provides:
  - "pim_command::Waiting and what_is_waiting: three answers about what the account is still owed, decided from whether anything will be sent and whether the setting allows it"
  - "pim_command::filed takes Waiting and joins one clause onto the sentence naming where the item went"
  - "allowed::turn_the_setting_on: the way out named at the moment one change is made, with changes_waiting_here built on it so the two cannot drift"
  - "managers::will_have_to_be_sent, public: whether filing into this container leaves anything for a sync to send, asked of the destination and never of the account"
  - "tests/a_moved_task_is_in_one_list.rs: six integration tests reading a moved task back out of storage"
  - "tests/a_move_says_what_has_not_been_sent.rs: seven integration tests, one of which is the only fixture that tells the destination question from the account question"
  - "three guard records, all measured by hand, and one existing record corrected from seven tests to eight"
affects: [05-07, 05-08, 05.1-03]

actuals:
  tokens: 71000
  tasks: 3
  commits: 4

tech-stack:
  added: []
  patterns:
    - "A question that must not be asked of the wrong subject is made unaskable: will_have_to_be_sent is not given an account, so it cannot answer from one"
    - "A three-valued answer where a boolean would lose a distinction: waiting for the next sync and held by a setting differ in whether the person can do anything about it"
    - "A shared clause is extracted and the older sentence rebuilt on it, so a wording rule has one home rather than two copies to keep in step"

key-files:
  created:
    - tests/a_moved_task_is_in_one_list.rs
    - tests/a_move_says_what_has_not_been_sent.rs
  modified:
    - src/application/pim_command.rs
    - src/application/allowed.rs
    - src/presentation/managers.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/IMPLEMENTATION_STATUS.md
    - docs/ALPHA_TESTING.md
    - .planning/REQUIREMENTS.md
    - .planning/WINDOWS.md
    - Cargo.toml

key-decisions:
  - "The predicate is will_have_to_be_sent(cache, kind, into), which takes no account at all. Passing one and not reading it would leave the wrong answer one edit away; not having one makes the defect unexpressible"
  - "An event is answered by calendar::where_a_change_goes, which is already public and is the calendar sync's own question, matched arm by arm so a sixth kind of calendar is a compile error rather than a silent default"
  - "A third function in allowed.rs rather than changes_waiting_here verbatim. That sentence counts, and a count read out at the moment somebody made one change is a tally of somebody else's work. changes_waiting_here's single arm is now built on the new clause so the two cannot drift"
  - "The clause is joined with a comma, and with a colon where it names the setting, rather than being a second sentence. It is heard after every filing and guardrail 5 is about bounded as much as distinct"
  - "Two guard records for task 2 rather than the one the plan asked for, because the defect has two sites in two files and neither break applies to the other"
  - "No changelog entry and no version bump in task 1, against the plan's prose. Nothing user-visible changed there, and CLAUDE.md says a bump is owed for a behaviour change. Task 2 carries both"

patterns-established:
  - "Where a plan predicts what a break will redden, the prediction is worth less than the run: this plan predicted two library tests and nothing in the new file, and the break reddens five tests across three targets"

requirements-completed: []

coverage:
  - id: D1
    description: "A task moved to another list is returned by that list, by no other list, and once by the account"
    requirement: PIM-01
    verification:
      - kind: integration
        ref: "tests/a_moved_task_is_in_one_list.rs#test_a_moved_task_is_returned_by_the_new_list_and_by_no_other"
        status: pass
      - kind: integration
        ref: "tests/a_moved_task_is_in_one_list.rs#test_a_moved_task_is_one_task_on_the_account_and_not_two"
        status: pass
      - kind: integration
        ref: "tests/a_moved_task_is_in_one_list.rs#test_a_moved_task_is_in_the_queue_to_be_sent_under_its_new_list"
        status: pass
    human_judgment: false
  - id: D2
    description: "A task a provider holds is refused a move, stays in the list it started in, and nothing is queued"
    requirement: PIM-01
    verification:
      - kind: integration
        ref: "tests/a_moved_task_is_in_one_list.rs#test_a_task_a_provider_holds_stays_in_the_list_it_started_in"
        status: pass
      - kind: integration
        ref: "tests/a_moved_task_is_in_one_list.rs#test_a_refused_move_puts_nothing_in_the_queue_to_be_sent"
        status: pass
      - kind: integration
        ref: "tests/a_moved_task_is_in_one_list.rs#test_the_fixture_for_a_provider_held_task_is_really_one_a_provider_holds"
        status: pass
    human_judgment: false
  - id: D3
    description: "Whether anything is waiting is decided by the destination and not by the account, so a list made here on a Gmail account is not announced as waiting"
    requirement: PIM-01
    verification:
      - kind: integration
        ref: "tests/a_move_says_what_has_not_been_sent.rs#test_a_list_made_here_on_a_gmail_account_has_nothing_to_be_sent"
        status: pass
      - kind: integration
        ref: "tests/a_move_says_what_has_not_been_sent.rs#test_a_list_a_provider_gave_out_has_something_to_be_sent"
        status: pass
      - kind: integration
        ref: "tests/a_move_says_what_has_not_been_sent.rs#test_a_note_has_nothing_to_be_sent_wherever_it_is_filed"
        status: pass
      - kind: integration
        ref: "tests/a_move_says_what_has_not_been_sent.rs#test_an_event_filed_into_a_calendar_an_account_holds_has_something_to_be_sent"
        status: pass
      - kind: integration
        ref: "tests/a_move_says_what_has_not_been_sent.rs#test_an_event_filed_into_a_calendar_made_here_has_nothing_to_be_sent"
        status: pass
      - kind: integration
        ref: "tests/a_move_says_what_has_not_been_sent.rs#test_an_event_filed_into_a_calendar_nobody_can_write_to_has_nothing_to_be_sent"
        status: pass
    human_judgment: false
  - id: D4
    description: "A filing says whether the account has had it yet, in one sentence, and names Allow Changes only where that setting is what is holding it"
    requirement: PIM-01
    verification:
      - kind: unit
        ref: "src/application/pim_command.rs#test_a_filing_the_account_is_owed_says_it_has_not_got_there_yet"
        status: pass
      - kind: unit
        ref: "src/application/pim_command.rs#test_a_filing_that_stays_on_this_computer_says_nothing_about_waiting"
        status: pass
      - kind: unit
        ref: "src/application/pim_command.rs#test_a_filing_the_setting_is_holding_names_the_setting_to_turn_on"
        status: pass
      - kind: unit
        ref: "src/application/pim_command.rs#test_the_setting_is_named_only_while_it_is_the_thing_holding_the_change"
        status: pass
      - kind: unit
        ref: "src/application/pim_command.rs#test_what_a_filing_says_is_one_sentence_however_much_it_carries"
        status: pass
      - kind: unit
        ref: "src/application/allowed.rs#test_one_change_held_at_the_moment_it_was_made_names_the_setting_the_sync_names"
        status: pass
    human_judgment: false
  - id: D5
    description: "The clause and the wording it uses are heard as useful rather than as noise after repeated moves"
    requirement: PIM-01
    verification:
      - kind: manual
        ref: ".planning/WINDOWS.md#214"
        status: pending
    human_judgment: true

status: complete
---

# Phase 05 Plan 06: A move says what has not left this computer, Summary

**It works, and one line of it is the honest part.** A move or copy into a list
or calendar an account holds now says "and has not reached the account yet", or
names Allow Changes where that setting is off, in one sentence joined to the one
saying where the item went. Nothing is added where nothing is waiting, and the
question deciding that is the destination's identifier rather than the account's
provider, which is the difference between mitigating the threat and shipping it.

The honest part: **"ends in exactly one list" was already structurally true and
this plan did not close it.** `TaskEntry.task_list_id` is one nullable column, so
a task being on two lists is not a state this storage can hold. The move reads
the row, writes that column and saves it. There is no second row to leave behind
and no write here can produce one. The six tests in
`tests/a_moved_task_is_in_one_list.rs` cannot fail against the code as it stands,
and saying otherwise would be reporting coverage of a risk they do not touch.

They are worth having anyway, and for one reason. The sequence the criterion is
about is the delete-there-create-here that `05-07` and `05-08` build. The first
thing anybody writing that reaches for is an insert, and an insert is exactly
what turns one column into two rows. These tests are the floor that would notice.

---

## What already existed, and where

Task 1 was written as three new assertions. Two of them already existed, in
`src/presentation/managers.rs`:

| Assertion | Already held by |
|---|---|
| The moved task is in `pending_tasks` under the new list | `test_moving_a_task_made_here_puts_it_in_the_queue_to_be_sent` |
| A provider-held task is refused, stays where it was, and nothing is queued | `test_moving_a_task_the_provider_holds_is_refused_and_writes_nothing` |

The second is the plan's third assertion verbatim. What is new is the reading:
`get_tasks_for_list` across three lists rather than the two the existing test
uses, and `get_all_tasks_for_account` counting one task rather than trusting the
lists. That last one is not redundant with the list readings, and the test says
why: a leftover row whose list column was cleared would appear on none of the
three lists and still be a second task on the account.

Three lists and not two, for the same shape of reason. With two, "in the new list
and in no other" is the same assertion as "not in the old one", and a move that
filed the task everywhere would pass it.

## The route, and what was not widened

`file_under` is public already, made so by `05-03` for exactly this reason.
Nothing here widened anything to write task 1.

Task 2 did make one function public, `managers::will_have_to_be_sent`, and the
alternative was a `#[test]` in `managers.rs`, which 44 guard records fingerprint
and which would put a build and a full library run on the critical path of the
commit that added it. Its doc comment says it is public for that reason, the way
`file_under`'s does.

## The predicate, which is the whole of the risk in this plan

`new_item::supports` answers a question about an **account**, from
`TASK_PROVIDERS = ["gmail", "outlook"]`. Whether a moved task is really waiting
is decided per **destination list**, by the identifier:

- `store_new_container` writes `new_id("tasklist")`, which is
  `tasklist-<nanoseconds>` with no provider anywhere in it.
- `service::tasks_api::google_list_to_entry` writes `format!("google:{}", ...)`,
  and the Graph side writes `ms:`.
- `push_tasks` does `if provider.is_local(&list_id) { result.local_only += 1;
  continue; }`, and `SyncResult::summary` reports those as "kept on this
  computer", not as `changes_waiting_here`.

So a locally made list sitting on a Gmail account has `supports` true and will
never be sent anything. A move into it announced as "has not reached the account"
would say the opposite of what the sync says about that very row. That is T-05-13
delivered by its own mitigation.

`will_have_to_be_sent(cache, kind, into)` takes no account, so the wrong answer
is not expressible in it rather than merely not written. The fixture that would
have caught it is
`test_a_list_made_here_on_a_gmail_account_has_nothing_to_be_sent`: a Gmail
account, a list made here, `supports` asserted true and the predicate asserted
false in the same test. Every other fixture in that file passes against the
`supports` version, which is why that one is written out on its own.

For an event the identifier says nothing, because a calendar is told apart by
where it came from. `calendar::where_a_change_goes` is already public and is the
calendar sync's own question, so it is asked rather than a second rule written
here. Its five answers are matched out arm by arm rather than negated, so a sixth
kind of calendar is a compile error: one silent default would say nothing about a
change that really is waiting, and the other would claim one that never leaves.

## The setting's answer is an argument, and here is the comment that says why

`allowed_for`'s own doc records the trap:

> Split out so it can be tested without touching the process-wide value. That
> value is set once, so a test for the unset case and a test that sets it cannot
> both live in the same process: Rust runs them in parallel and whichever went
> first decided the answer for the other. Not hypothetical, it is what happened,
> and it failed about one run in three.

And `allowed_for` itself reads `ConfigManager::load_stored()`, which is whoever's
settings file is on the machine running the tests. So the setting's answer is
fetched at the call site in `file_it` and handed to `what_is_waiting` as a bool.
Nothing under test reads a settings file.

## The words, quoted in full

| What is true | What somebody hears |
|---|---|
| Nothing is waiting | `Buy milk moved to Shopping` |
| The next sync sends it | `Buy milk moved to Shopping, and has not reached the account yet` |
| Allow Changes is holding it | `Buy milk moved to Shopping, and has not reached the account: turn on Allow Changes in Settings to send it` |

A note, side by side with the same move into a synced list:

- `Shopping list moved to Home`
- `Buy milk moved to Shopping, and has not reached the account yet`

The longest sentence this can produce is the third row above with a long title
and a long container name. `test_what_a_filing_says_is_one_sentence_however_much_it_carries`
holds it to one sentence by asserting no `". "` appears in it, which is the
assertion that would notice somebody joining the clause on with a full stop.

## The setting clause is a third function, and the plan said to expect that

The plan asked for `allowed::changes_waiting_here` and said that if its wording
did not fit a single move, that was a finding and the answer was a sibling
function. It does not fit. `changes_waiting_here(1)` is "1 change is waiting
here: turn on Allow Changes in Settings to send it", which is a **count**, said
by a sync that has just looked at everything waiting. Said at the instant
somebody moved one thing, "1 change is waiting here" reads as a tally of somebody
else's work rather than as an answer about the thing they just did. It is also a
whole second sentence.

So `allowed::turn_the_setting_on()` returns the tail,
`turn on Allow Changes in Settings to send it`, and **`changes_waiting_here`'s
single-change arm is now built on it**, so there is one copy of those words
rather than two. That module's own comment is the argument: two hand-written
copies of the waiting sentence drifted and only one of them was ever corrected.
The plural arm keeps its own whole sentence, because the three words that have to
agree are in it.

`test_one_change_held_at_the_moment_it_was_made_names_the_setting_the_sync_names`
asserts both directions. **It passed on arrival**, which is worth saying plainly:
that function is an extraction of a string the syncs already said, so there was
no behaviour for a failing test to demand, and the test is a drift guard rather
than a red I took. It is not counted as part of the red/green cycle below.

## Guard records: three added, one corrected, all measured

### The record for task 1, and the two ways the plan was wrong about it

The break is the half-fix: `file_under` skips the second `moving_can_be_told`
check while `where_it_could_go` keeps the first. That is what somebody tidying up
a duplicated question writes, and it leaves the refusal working from the menu and
gone from every other route into the write.

Measured with `cargo test --all-targets --no-fail-fast` on a clean tree. It
reddens **five tests across three targets**:

```
presentation::managers::tests::test_moving_a_task_the_provider_holds_is_refused_and_writes_nothing
presentation::managers::tests::test_moving_an_event_the_provider_holds_is_refused_and_writes_nothing
a_copy_leaves_the_original_where_it_was::test_a_task_a_provider_holds_can_be_copied_although_it_cannot_be_moved
a_moved_task_is_in_one_list::test_a_task_a_provider_holds_stays_in_the_list_it_started_in
a_moved_task_is_in_one_list::test_a_refused_move_puts_nothing_in_the_queue_to_be_sent
```

The plan first said the record should carry `suite = "a_moved_task_is_in_one_list"`,
then corrected itself to say it must not, on the grounds that no test in that file
can reach `file_under`. **Both are wrong about the same fact.** Those tests reach
it perfectly well and two of them go red. The correction reached the right
conclusion from a false premise.

The record carries no `suite`, which is right for a different reason than the one
given: `scripts/guards.py` runs `cargo test --lib` for a record without one, so a
library test is the only kind its red list can hold, and the library pair covers
the event as well as the task while the integration pair covers only the task.
The three integration failures are written into the record's comment so nobody
re-measures it and thinks it came out short.

`scripts/guards.sh "second question"` reports: *all 2 tests named went red, and
nothing else did.*

### The two records for task 2

**"the clause about the account is said only where something really is waiting."**
The break is the sentence made uniform: `Waiting::StaysHere` gets the clause too.
Four tests, three of them older than this feature:

```
application::pim_command::tests::test_a_copy_says_it_was_copied_and_never_that_it_moved
application::pim_command::tests::test_a_filing_that_stays_on_this_computer_says_nothing_about_waiting
application::pim_command::tests::test_a_move_of_an_untitled_row_still_says_where_it_went
application::pim_command::tests::test_a_move_says_what_moved_and_where_it_went
```

The shape worth noting: what catches this are the plain tests asserting what a
move says, not the new one asserting what it says when something is waiting. A
guard asking only whether the clause can appear would have stayed green through
the whole break, which is what the plan predicted and is the one prediction it
got right.

**"nothing is waiting when nothing was going to be sent."** The same defect one
step earlier, in `what_is_waiting` rather than in `filed`. A second record because
they are different edits in different functions and neither break applies to the
other. It reddens **exactly one test**, measured rather than predicted, and that
is left on the record rather than widened until it looks healthier. The reason it
is one: every other test of the sentence hands `filed` a `Waiting` directly and
never comes through the deciding, and the integration tests ask
`will_have_to_be_sent`, which is the other half of the question.

### The record that came out short, found by remeasuring

`--remeasure` on the nine records whose test counts moved reported one
disagreement:

```
-- one change waiting is not read out in the plural
   1 test went red that this record does not name:
       application::allowed::tests::test_one_change_held_at_the_moment_it_was_made_names_the_setting_the_sync_names
```

That is correct and was not predictable. Building `changes_waiting_here`'s single
arm on the extracted clause means the new drift test reddens under that record's
break as well. Corrected by hand from seven tests to eight, with the reason
written into the record, and measured again: *all 8 tests named went red, and
nothing else did.*

### Counts

| | Before | After |
|---|---|---|
| `guards/guards.toml` records | 677 | 680 |
| Census, line 79 + line 80 | 192 + 485 | 192 + 488 |
| `src/presentation/managers.rs` `#[test]` | 137 | 137 |
| `src/application/tasks_sync.rs` `#[test]` | 106 | 106 |
| `src/application/pim_command.rs` `#[test]` | 30 | 38 |
| `src/application/allowed.rs` `#[test]` | 26 | 27 |

**No `#[test]` was added to `managers.rs` or `tasks_sync.rs`.** Both counts are
unchanged.

Records fingerprinting each file, counted by parsing `tests_last_seen` blocks and
not by grepping a file name:

| File | Records | Plan said |
|---|---|---|
| `src/application/pim_command.rs` | 3, now 5 | 3 |
| `src/application/allowed.rs` | 4 | not measured; this plan was asked to measure it |
| `src/presentation/managers.rs` | 43, now 44 | 40 |
| `src/application/tasks_sync.rs` | 19 | 19 |
| `src/application/calendar.rs` | 71 | not named |

`allowed.rs` costs **4**, which is the number the plan asked for and did not have.
`managers.rs` had drifted from 40 to 43 in the five plans since this one was
written, which is why nothing was tested there.

The scoped `scripts/guards.sh --remeasure` the commit gate would print was run
before the green commit rather than after it, detached, with
`WIXEN_TEST_THREADS=4`, over all nine affected records:

```
a refusal names the kind with the article that belongs with it
a confirmed delete that finds nothing to delete still says so
a command that failed says what it was and what to try
the clause about the account is said only where something really is waiting
nothing is waiting when nothing was going to be sent
one change waiting is not read out in the plural
one removal waiting is not read out in the plural
the constant that changes nothing still reads mail
a settings file written before reading was a setting still loads
```

## Red and green

The RED commit is `3b09589`, on this branch, accepted by `scripts/red-commit.sh`
with six `Fails-until-green:` trailers. Five are the new behaviour and the sixth
is `test_every_guard_record_says_how_many_tests_the_files_it_names_held`, named
for the reason `CLAUDE.md` gives: adding a test to a file guard records name turns
the count check red, and its remedy needs the green code before a record's red
list can be corrected, so there is no ordering in which that commit has a clean
tree around it.

The red half is today's behaviour said through the new shape: `filed` takes
`Waiting` and ignores it, `what_is_waiting` answers `StaysHere` for everything.
Both failed by assertion rather than by compile error.

GREEN is `b45806a`. All four checks passed; the commit touches `Cargo.toml`, so
`which-checks.sh` answered `all` and the whole gate ran including the release
build.

## Deviations from the plan

**1. [Deliberate] No changelog entry and no version bump in task 1.** The plan's
prose asks for both. Task 1 adds tests and a guard record and changes no
behaviour, and `CLAUDE.md` says a bump is owed for a feature, schema or behaviour
change, not for a test pass. Task 1's acceptance criteria mention neither. Task 2
carries the entry and the bump, `0.98.0` to `0.99.0`, and the entry says the
limit task 1 was asked to record: a task the account holds still cannot be moved
between lists at all.

**2. [Deliberate] Two guard records for task 2 rather than one**, for the reason
above.

**3. [Deliberate] A second new integration file**, `tests/a_move_says_what_has_not_been_sent.rs`,
rather than putting the predicate's tests in task 1's file or in `pim_command.rs`.
A new file under `tests/` is named by no record, so it costs nothing, and the
fixture is a stored list and a stored calendar, which no unit test in
`pim_command.rs` can build.

**4. [Rule 2 - Missing critical] The event and reminder cases were answered too.**
The plan's behaviour list names tasks and notes. `filed` is said for events by the
same path and for reminders by `05-05`'s. Leaving events unanswered would have
been the same repudiation the plan is about, one kind over: an event moved into a
Google calendar really is waiting and would have said nothing. A reminder is never
waiting and says so where it is written, with the reason beside it.

**5. [Correction] The plan's `move_item` does not exist.** `05-03` split it into
`file_it` and `where_it_could_go`. `into.id` is in hand at the call site in
`file_it`, which is what the plan's correction was really about, so the substance
held.

**6. [Correction] The plan's `pim_command::moved` does not exist either.** `05-03`
replaced it with `filed` taking the act. The new parameter went on `filed`.

Everything else went as written.

## What is not done, and who owns it

**PIM-01's third criterion is not closed and was not reworded.** Decision 1 of
2026-09-06 says the provider move gets built, so it becomes true by being
satisfied. `05-07` makes the half-finished state a stored, visible, recoverable
object and `05-08` makes the provider calls, in that order. `.planning/REQUIREMENTS.md`
now names both under the criterion.

**PIM-01's Allow Changes criterion was reworded**, per decision 5, dated and
attributed. It asked for a refusal at move time, which nothing has ever done. The
gate is applied where each HTTP client is built, a refused push is counted, and
the sync summary names the setting; what this plan adds is saying the same thing
at the moment somebody makes the change.

**PIM-01 stays open.** One of its three criteria is now closed by measurement, one
by rewording, and one is owed to two later plans.

## Known limitations

- **Nothing here has run against a real account**, so the clause's promise, that
  the next sync sends it and says so, has never been followed by an account
  receiving anything. Ledger 216.
- **Nobody has heard the clause.** Whether one sentence carrying both facts reads
  as one answer or as a run-on, and whether it is useful or wearing after the
  twentieth move, is a judgement about hearing it. It is said after every single
  filing, which is the guardrail 5 risk. Ledger 214.
- **`05-RESEARCH.md`'s assumption A2 is not settled and cannot be settled here.**
  Whether `Ctrl+Shift+V` really reaches the handler in the non-mail modules,
  rather than only appearing in a menu label, is not something any test in this
  repository can answer; `tests/wired.rs` says so in its own header. Ledger 215.
- **A note still carries no waiting flag**, so a note is answered `StaysHere`
  unconditionally. `05.1-03` gives notes their sync columns and comes through the
  same write, so `will_have_to_be_sent`'s note arm is one of the answers it has to
  change. The arm says so.

## Known Stubs

None. Every function added here is reached from `file_it`, which is reached from
the Move and Copy commands in five modules.

## Self-Check: PASSED

Files created, checked on disk:

- `tests/a_moved_task_is_in_one_list.rs` FOUND
- `tests/a_move_says_what_has_not_been_sent.rs` FOUND
- `.planning/phases/05-the-other-five-modules-keep-up/05-06-SUMMARY.md` FOUND

Commits, checked in `git log`:

- `8e364be` FOUND: test(05-06), task 1
- `3b09589` FOUND: test(05-06), the RED half of task 2
- `b45806a` FOUND: feat(05-06), the GREEN half of task 2
- `b7810ce` FOUND: docs(05-06), task 3
- `88575ec` FOUND: docs(05-06), this summary, the ledger and the counters
- `b7f5953` FOUND: the merge into `main`

`scripts/check.sh all` was run on the branch before the merge, detached, and
reported all four checks passed. The branch was `a-move-says-what-has-not-left-this-computer`.

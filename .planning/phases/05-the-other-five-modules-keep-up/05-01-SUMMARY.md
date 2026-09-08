---
phase: 05-the-other-five-modules-keep-up
plan: 01
subsystem: ui
tags: [calendar, recurrence, screen-reader, exdate, guards]

requires:
  - phase: 04-mail-that-moves
    provides: the guard-record and red-commit machinery this plan is measured by
provides:
  - "tests/a_moved_day_is_shown_once.rs: a stored series and a stored moved day, read back through the query and the expansion the panel uses, settling by running it that a moved day is on the calendar once"
  - "CalendarEventItem::changed_on_its_own, filled in from_entry from either stored exception column"
  - "read_aloud::a_changed_day_worth_saying, one function with two callers, so the row cell and the Space reading cannot drift"
  - "the clause in pim_rows::event_cell, which is what a screen reader speaks while arrowing"
  - "four guard records: two on the de-duplication, two on the clause"
affects: [05-02, 05-03, 05-04, 05-05, 05-06]

actuals:
  tokens: 11000
  tasks: 2
  commits: 3

tech-stack:
  added: []
  patterns:
    - "A qualifying clause on a list row goes through a `*_worth_saying` function shared between `pim_rows` and `read_aloud`, following `priority_worth_saying` and `status_worth_saying`"
    - "Column 0 of the calendar list is a join of non-empty parts rather than nested match arms, so several qualifying clauses can apply at once"

key-files:
  created:
    - tests/a_moved_day_is_shown_once.rs
  modified:
    - src/presentation/ui_types.rs
    - src/presentation/read_aloud.rs
    - src/presentation/pim_rows.rs
    - tests/calendar_immediate_actions.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
    - .planning/WINDOWS.md

key-decisions:
  - "05-RESEARCH.md assumption A1 is wrong. A moved day is shown once, measured rather than reasoned about."
  - "One field on CalendarEventItem, not two, because who moved the day is not something a listener needs."
  - "The clause lives in event_cell as well as read_short. read_short alone is the Space reading only and would be heard by nobody arrowing."
  - "Four guard records where the plan asked for two, because in both cases one break did not reach both halves of what it was meant to guard."
  - "Task 1 carries no version bump. It changes no behaviour, and the project rule is that a bump goes with a change."

patterns-established:
  - "A guard record carrying `suite` cannot name a library test, so a break that reddens both needs two records measured separately"
  - "Where a break is measured and reddens fewer tests than expected, that is a finding about the tests rather than a record to pad"

requirements-completed: []

coverage:
  - id: D1
    description: "A day of a repeating event that was moved is on the calendar once, on the date it really is, and the day the series calls off is on no date at all"
    requirement: PIM-03
    verification:
      - kind: integration
        ref: "tests/a_moved_day_is_shown_once.rs#test_a_day_moved_out_of_a_series_is_on_the_calendar_once"
        status: pass
      - kind: integration
        ref: "tests/a_moved_day_is_shown_once.rs#test_a_day_the_series_calls_off_is_on_no_date_at_all"
        status: pass
    human_judgment: false
  - id: D2
    description: "A row that is one day of a series changed on its own carries the fact, from either stored column, through from_entry and every expanded day"
    requirement: PIM-03
    verification:
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_a_day_changed_on_its_own_is_known_from_either_stored_column"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_every_day_a_series_is_shown_on_says_whether_that_day_was_changed"
        status: pass
    human_judgment: false
  - id: D3
    description: "The row a screen reader reads while arrowing says the day was changed, and so does the Space reading, in the same words"
    requirement: PIM-03
    verification:
      - kind: unit
        ref: "src/presentation/pim_rows.rs#test_a_day_changed_on_its_own_says_so_in_the_row_itself"
        status: pass
      - kind: unit
        ref: "src/presentation/pim_rows.rs#test_a_changed_day_out_of_hours_is_told_both_things"
        status: pass
      - kind: unit
        ref: "src/presentation/read_aloud.rs#test_a_day_changed_on_its_own_says_so_in_the_short_reading"
        status: pass
      - kind: unit
        ref: "src/presentation/read_aloud.rs#test_the_full_reading_does_not_say_the_day_was_changed_over_again"
        status: pass
      - kind: unit
        ref: "src/presentation/read_aloud.rs#test_a_changed_day_of_a_series_nobody_can_work_out_is_told_both_things"
        status: pass
    human_judgment: false
  - id: D4
    description: "The clause is worth hearing rather than clutter, and is not confused with the unreadable-rule sentence"
    verification: []
    human_judgment: true
    rationale: "Nobody has heard it. Tests prove the words are produced; whether they help somebody arrowing past fifty-two rows of one series is only answerable with NVDA. Ledger 193."

duration: 195min
completed: 2026-09-08
status: complete
---

# Phase 5 Plan 01: A moved day, and a day that says it was changed

**It works. A moved day of a series was already shown once and now has a test that
would notice if it stopped being, and a day of a series that was changed on its own
now says so in the row a screen reader reads while somebody arrows past it. Nobody
has heard it yet.**

## Performance

- **Duration:** about 195 minutes
- **Tasks:** 2
- **Files modified:** 10 (one created)
- **Commits:** 3

## What works, and what does not

**Works, and was run.** A weekly series and a day moved out of it, stored through the
real writer and read back through the real query and the real expansion, leave one row
on the date the day moved to and none on the date it left, with the other weeks
untouched. A day the series calls off is on no date at all. Both hold whether the day
was moved here or by a calendar server.

**Works, and was run.** A row built from an event with either exception column set
carries the fact, through `from_entry` and out through every day `shown_days` expands.
The row a screen reader reads while arrowing says "changed just for this day" after the
time, the Space reading says the same words from the same function, and the fuller
reading does not say it a third time. An ordinary event and an ordinary day of a series
say nothing extra.

**Does not work, in the sense that nobody has checked.** Nothing here has been heard.
The words were chosen against the sentences `application::calendar` already speaks, not
against a listener. Ledger 193.

**Not touched.** PIM-03's third `[D]` line is not closed by this plan and should not be
read as closed. It asks that two stated limitations are restated in the product rather
than left in the changelog; `can_be_honoured` already refuses one of them in the product
with a sentence, and the recurrence editor's half was not checked here. That is carried
as owed work in this phase's `README.md`, item 6. `requirements-completed` is therefore
empty: PIM-03 needs `05-02` and the editor's half before it closes.

## Assumption A1 was wrong

`05-RESEARCH.md` says a moved occurrence appears twice, once expanded from the series and
once as its own row, and says plainly that this came from reading rather than running. It
is wrong, and the test was green on arrival.

Why: every path that stores a moved day also takes that day off the series, and
`occurrences::falls_on` filters the expansion by exactly that EXDATE. The four call sites
of the writer, found with

```
grep -rn "one_day_kept_out_of_the_series" src/ --include=*.rs
```

are `src/application/caldav_sync.rs:1094`, `src/application/calendar.rs:1024` (the Google
read), `src/application/calendar.rs:1399` (the Outlook read), and
`src/presentation/managers.rs:678` (the screen that changes one day). The two in
`calendar.rs` each sit under a `let Some(the_day_it_was) = ... else` arm that logs and
counts `days_that_may_be_shown_twice`, so the tree already treats the double-show as the
exception and already announces it.

Because the test was green on arrival, the red half is a measured break rather than a
pretend red. That is written up under "Guard records" below.

## Premise correction 4 stayed open

The plan asked whether an override row carries a repeat rule of its own, which would
expand it across the whole window and be worse than the fault A1 describes. An earlier
draft of task 1 settled it by asserting the stored override's `recurrence_rule` is `None`
after the write. **That assertion cannot fail, and the plan's own correction of 2026-09-08
says so.** The fixture builds the moved day the way `the_day_kept_on_its_own` builds one,
which sets `recurrence_rule: None` explicitly, and `one_day_kept_out_of_the_series` then
does nothing but `save_calendar_event`. So the assertion asks whether the store round-trips
a constant the test itself wrote, and it is green against every implementation including
the broken one.

Premise 4 is about what a *provider* sends. Both paths that could populate it derive it
from the payload: `google_event_to_local` from `only_the_line_naming(&event.recurrence,
"RRULE")` and `ms_event_to_local` from `event.recurrence.as_ref().and_then(..)`. No local
round trip can reach either. It is open, and it is ledger 194.

## Task commits

1. **Task 1: a stored series and a stored moved day, read back the way the panel reads
   them** - `92ff018` (test)
2. **Task 2 RED: failing tests for a day of a series changed on its own** - `3d3d30a`
   (test, accepted by `scripts/red-commit.sh` with eight named failures)
3. **Task 2 GREEN: the field, the clause, and both readers** - `b08d0a9` (feat)

## Files created and modified

- `tests/a_moved_day_is_shown_once.rs` - new. Two integration tests spanning the writer,
  the query and the expansion, counting rows by date.
- `src/presentation/ui_types.rs` - `CalendarEventItem::changed_on_its_own`, filled in
  `from_entry` from `cut_from_event_id` or `provider_recurrence_id`. Two tests.
- `src/presentation/read_aloud.rs` - `a_changed_day_worth_saying`, and the clause in
  `read_short`. Three tests.
- `src/presentation/pim_rows.rs` - the clause in `event_cell` column 0, which became a
  join rather than nested match arms. Two tests.
- `tests/calendar_immediate_actions.rs` - one construction site.
- `guards/guards.toml` - four records added, census on line 80 raised from 470 to 474,
  three unrelated records re-measured by the scoped remedy.
- `docs/changelog.md` - the new clause under Added, the de-duplication and its limit under
  Known limitations.
- `Cargo.toml` - 0.91.0 to 0.92.0, in the GREEN commit.
- `.planning/WINDOWS.md` - ledger 193 and 194.

## The words, and why these words

The clause is **"changed just for this day"**.

It was written against the two sentences `application::calendar` already speaks about a
single day of a series, so that a calendar sounds like one voice rather than two.
`one_day_taken_off` says "That one day is taken off. The other days are unchanged." and
`WrittenDown::OneDayChanged` says "Only the day you opened was changed. The other days
were left alone." Both put the change and its narrowness in one breath, and both use the
plain word "changed" rather than any of the machinery's own words. Five words is the
shortest thing that keeps all three of those properties: it says the day differs, it says
the rest of the series does not, and it cannot be heard as the meeting having been called
off, which "cancelled", "removed" or "exception" would risk. It is short because it is
paid on every changed day while somebody arrows, and the row beside it already carries the
date and the title, so it adds nothing they already say. It shares no words with
`repeats`, so a listener does not hear two overlapping statements about how the series
behaves.

## The construction sites: the compiler said 7, not 13

The plan expected the compiler to name 13, corrected down from a grep of 21 lines. The
grep's 21 breaks down exactly as the plan says: 3 declarations, 6 `-> CalendarEventItem {`
return-type signatures, 12 literals. But **six of the twelve literals build from
`..event()`**, so the compiler names only 6 of them, plus the one `Self {` inside
`from_entry`. Seven, at `tests/calendar_immediate_actions.rs:78`,
`src/presentation/pim_rows.rs:351`, `src/presentation/read_aloud.rs:630`, `:1311`,
`:1341`, `src/presentation/ui_types.rs:1371`, and `from_entry` itself.

Related: the plan expected `-D warnings` to force the reading into the RED commit, because
a field nothing reads will not build. That is true of a private field. `CalendarEventItem`
is public with public fields, so `cargo check --all-targets` came back clean with the field
wired to a constant and nothing reading it. The RED commit is therefore genuinely
behaviour-neutral, which is better than the plan expected rather than worse.

## Guard records: four, not two

**Task 1's break** is the half-fix rather than the deletion: `one_day_kept_out_of_the_series`
stores the changed day and not the series, which is the half somebody would actually write.
`let _ = (series, the_day_it_was, who)` is in the break only because the arguments are then
unused and the gate lints with `-D warnings`.

Measured with `cargo test --test a_moved_day_is_shown_once`, it reddens one test in that
target. Measured with `WIXEN_TEST_THREADS=4 cargo test --lib --no-fail-fast` on a clean
tree, it reddens seven library tests across `application::caldav_sync`,
`application::calendar` and `presentation::managers`. Those are two records, not one red
list: a record carrying `suite` is run with `--test <suite>` and nothing else, and
`tests/house_style.rs` resolves every name in such a record's red list to
`tests/<suite>.rs`, so a library test path named there is reported gone.

**Task 2's break** was meant to be one record. The plan says to break the field to a
constant. Measured, that reddens **two** tests, both in `ui_types`, and leaves the five
tests about what is said green, because those build their rows with the field already set
and never go through `from_entry`. So the clause needs its own break: the `*_worth_saying`
function answering nothing whatever it is asked, which reddens exactly those five across
`pim_rows` and `read_aloud`. Two records.

Both times the finding is the same shape and it is worth stating on its own: **one break
did not reach both halves of what it was meant to guard, and the answer is a second
record rather than a wider red list.** A break that reddens fewer tests than expected is
something to read.

Line 80's census went 470 to 472 in the task 1 commit and 472 to 474 in the task 2 commit.
662 records became 666, and `192 + 474 = 666` holds.

## The scoped remeasure

Adding five tests across `read_aloud.rs` and `pim_rows.rs` turned
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` red inside the commit
gate, naming three records. It was named as a failure in the RED commit's trailers, because
its remedy needs a tree with nothing else red in it, and run after the GREEN half:

```
WIXEN_TEST_THREADS=4 scripts/guards.sh --remeasure \
  "a moment written with a T is one the reader knows" \
  "a whole day is spoken as a date under the relative style" \
  "the out-of-hours note judges the hour the cell speaks"
```

All three still redden exactly the tests their records name: 34, 7 and 2 respectively. The
counts were written down again. Notably the out-of-hours record still reddens exactly its
two named tests even though this plan restructured the arm it breaks, which is the answer
that had to be measured rather than assumed.

## Test counts before and after

No `#[test]` was added to `src/application/calendar.rs` (194 before, 194 after),
`src/application/occurrences.rs` (62, 62) or `src/presentation/managers.rs` (137, 137).
Those are the three files the plan named as expensive, at 69, 3 and 40 records
fingerprinting them. Added: `src/presentation/ui_types.rs` 54 to 56 (0 records),
`src/presentation/read_aloud.rs` 46 to 49 (2 records), `src/presentation/pim_rows.rs` 23 to
25 (1 record), and a new file under `tests/` holding 2 (0 records).

## Deviations from plan

**1. [Rule 1 - Bug in the plan's own fixture] The first draft of task 1's test asserted an
ordinary day of the series on 2026-07-27.**
- **Found during:** Task 1, first run.
- **Issue:** The series starts on 2026-08-03, so it never falls on 2026-07-27. The test was
  red for my mistake, not the code's.
- **Fix:** The moved day became the third Monday rather than the first, so there are
  ordinary days of the series on both sides of it. That is a stronger fixture than the one
  the plan described: a day taken off the front cannot tell a calendar that lost one day
  from one that lost every day up to a point.
- **Verification:** Green, and the measured break reddens it.

**2. [Rule 2 - Missing critical] Two guard records where the plan asked for one, twice.**
- **Found during:** Task 1 and Task 2.
- **Issue:** In both cases the break the plan named reaches only part of what it is meant
  to guard. Task 1's break reddens tests in one integration target and seven in the
  library, which cannot go in one record. Task 2's break reddens the field tests and not
  the clause tests.
- **Fix:** Four records, each measured on its own, with what each really reddens written
  down.
- **Verification:** Every red list was taken by hand on a clean tree with nothing else
  building, and `test_every_test_a_guard_record_names_is_a_test_that_exists` and
  `test_every_guard_record_says_how_many_tests_the_files_it_names_held` both pass.

**3. [Deviation from plan, deliberate] Task 1 carries no version bump.**
- **Issue:** The plan says to bump the version in task 1. `CLAUDE.md` says a bump goes with
  a feature, a schema change or a behaviour change, and that a bug fix or a docs pass does
  not need one. Task 1 changes no software: it adds a test and a changelog note about a
  limitation that already existed.
- **Fix:** No bump in task 1. Task 2 bumps 0.91.0 to 0.92.0 in the same commit as the
  change, which is the rule as written. The project file is the authority over the plan
  here, and bumping would have claimed a change nobody made.

**4. [Correction to the plan's premise 6] `-D warnings` does not force the reading into the
RED commit.**
- **Issue:** The plan expects a field nothing reads not to build. That holds for a private
  field; these are public fields on a public struct in a library crate.
- **Fix:** None needed. The RED commit is behaviour-neutral, and every named failure fails
  on the clause not being said.

**Total deviations:** 4. One fixture bug of my own making, two guard records added beyond
the plan for coverage the measurement showed was missing, one version-bump rule applied
over the plan's instruction. No scope creep.

## Issues encountered

The plan was written against `main` at `9611b70`, version 0.75.0, with 632 guard records.
The tree had moved to 0.91.0 with 662 records. **Every per-file figure in the plan's
premise 8 table was still exactly right** when re-measured, which is the point of counting
records by parsing `tests_last_seen` rather than by grepping a file name. Only the totals
had moved.

## Next plan readiness

`05-02` extends PIM-03's first `[D]` line to the week and month windows and depends on this
plan. Nothing here blocks it. `CalendarEventItem::changed_on_its_own` reaches every
expanded row through `shown_days`, so a narrower window over the same list carries the
clause without doing anything.

Two things are owed and neither blocks a merge. `scripts/guards.sh --touched-by <the commit
this branch left main at>` belongs to the phase-8 sweep. And the four new records have
never been through a sweep, which the census now says.

---
*Phase: 05-the-other-five-modules-keep-up*
*Completed: 2026-09-08*

---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 31
subsystem: free/busy
tags: [free-busy, microsoft-graph, time-zones, GAP-08, "#57"]
status: complete
requires: [13-30, 13-21.1]
provides:
  - "free_busy: OneDiary reads workingHours.timeZone.name and places it through common::zones::the_zone_called; a Heard carries a zone beside each place's diary"
  - "free_busy: one_answer_each keeps the zone a person came with, otherwise the first a place gave"
  - "when_people_are_free: the_events_own_zone asks common::zones, so an event's Windows zone name is placed"
  - "when_people_are_free: WhenWeCouldMeet::where_they_are_known, and in_words(say, on_their_clock) saying up to three guests' clocks beside each offered time"
  - "when_people_are_free: TheirDay and OnTheirClock, worded by asking_when_free::on_their_clock through date_display"
affects: [13-51]
tech-stack:
  added: []
  patterns: ["a clause per guest bounded by a named constant, compared by clock offset rather than by zone name", "a list whose items hold commas is kept apart by semicolons"]
key-files:
  created: []
  modified:
    - src/service/free_busy.rs
    - src/application/when_people_are_free.rs
    - src/application/asking_when_free.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "Decision 1 kept: a contact's zone is later work, ledgered as 708."
  - "Decision 2 kept: the zone is placed by 13-21.1's common::zones; nothing here calls ICU."
  - "Decision 3 kept: at most three clocks per time, HOW_MANY_CLOCKS_ARE_SAID, in invitation order, only where the guest's clock offset differs from the organiser's at that instant."
  - "A zone the guest came with outranks one a place gave; the first place that gave one wins otherwise."
  - "An hour on a guest's clock is worded by the reader's own date settings: the clock alone on the same day, the whole date and clock on another, the two readings time_elsewhere uses."
  - "Offered times holding a comma are joined by semicolons, two of them as well as three."
metrics:
  duration: "about 1 hour 45 minutes"
  completed: 2026-09-29
actuals:
  tokens: 14500
  tasks: 3
  commits: 7
---

# Phase 13 Plan 31: A guest's own clock Summary

A colleague Microsoft places is now judged by their own working day and hears each offered
time on their clock. Microsoft's `getSchedule` gives each guest's working hours with the
Windows name of their zone; that name is read and placed through `common::zones`, the guest
leaves the sentence saying nobody said where they are, and each offered time carries up to
three guests' clocks: "03/03/2026 at 10:00, which is 15:00 for Ada". An event on the
organiser's own calendar written in a Windows zone now blocks the hour that zone names.

## What was built

- **The zone from Microsoft** (`src/service/free_busy.rs`). `OneDiary` reads
  `workingHours.timeZone.name`; `OneDiary::where_they_are` asks
  `common::zones::the_zone_called`, so "Pacific Standard Time" is `America/Los_Angeles` on
  Windows and "Customized Time Zone" is nothing. What each place said about a person is a
  `Heard`, the diary and a zone; the calendar server and Google give no zone, through `From`.
  `one_answer_each` keeps the zone the person was asked about with and otherwise takes the
  first zone any place gave.
- **An event's own zone** (`src/application/when_people_are_free.rs`). `the_events_own_zone`
  asks the same resolver, so an event beside a Windows zone name blocks the instants that
  zone names rather than the hour on the organiser's clock.
- **The clocks** (same file). `WhenWeCouldMeet` carries `where_they_are_known`, everybody
  placed, in the order invited. `a_time_on_every_clock` adds "which is 15:00 for Ada" for up
  to `HOW_MANY_CLOCKS_ARE_SAID` guests whose clock offset differs at that instant, and
  `on_their_clock_for` says the day too when it is another day there. `in_words` takes a
  second argument, `OnTheirClock`, which `asking_when_free::on_their_clock` words through
  `date_display::the_clock_of` and `date_display::a_clock_face`. Times holding a comma are
  joined by semicolons (`each_heard_whole`).
- **The pages.** The guide gains "Finding a time everyone is free" after Time zones: what is
  asked and where, the four reasons a guest is not checked, where each guest is judged, the
  clocks, and that it is experimental. The alpha page says a guest's zone has been read only
  from answers in tests. The changelog has it under Added with #57 point 3.

Reachability: the only production path to the sentence is `managers.rs:1137`,
`asking_when_free::in_sentences`, which now passes both wordings; the zone arrives through
`free_busy::when_they_are_free`, which the event form's worker already calls.

Test counts, taken 2026-09-29 on the branch: `application::when_people_are_free::` 55 (was 49
at the start of the plan), `service::free_busy::` 56 (was 52), `application::asking_when_free::`
20 (unchanged), `tests/item_form_free_busy.rs` 7 (unchanged). Four of the new cases run on
Windows only, because only Windows places a Windows zone name.

GAP-08's third `[D]` line is marked with the date and its tests, and GAP-08 is ticked with its
traceability row updated; the `[S]` lines are untouched.

## Commits and hook times

| Commit | What | Hook |
|---|---|---|
| `65bf4d32` | test: an event beside a Windows zone name, red | red, 114 s |
| `afb4b615` | feat: an event's own zone read by the shared resolver; one record re-measured | affected, 205 s |
| `f4e58605` | test: a guest's zone and clock, red, with the shape the cases need | red, 166 s |
| `19d50321` | feat: the zone read from Microsoft, the clocks said; four records re-measured | affected, 263 s |
| `731cc361` | test: times carrying a clock kept apart, red | red, 108 s |
| `7b030fce` | feat: semicolons between times holding a comma; two records added | affected, 228 s |
| docs | the pages, the ledger, this summary, the marks | this commit |

## Guard records

Two new, each measured with `--remeasure`: "a guest's zone is read from the working hours
microsoft gives" (two tests, both Windows only) and "no more than three guests' clocks are
said beside a time" (the four-guest case). The arrived-since count went from 460 to 462.

Re-measured: "a moment written with a T is one the reader knows" three times, once short by
the new Windows-zone event case, which it now names (73 tests); the three `free_busy.rs`
records from 13-29 and 13-30 at 56 tests, unchanged in what they redden. No record anchors in
a block this plan moved: the one `free_busy.rs` anchor inside `what_every_place_said`
(`stretches.extend(more);`) is untouched.

## Deviations from Plan

**1. [Rule 3] `src/application/asking_when_free.rs` changed, not in `files_modified`.** The
sentence is worded there with the reader's settings, and `in_words` now takes the wording for a
guest's clock, so its one production caller had to pass it. One new private function,
`on_their_clock`.

**2. [Shape] The guest's clock is worded outside the module.** The plan's example read
"Tuesday at 10:00, which is 15:00 for Ada". `when_people_are_free` must not reach into
settings, so the hour there arrives through `OnTheirClock` the way the organiser's arrives
through `InWords`, and the real sentence reads the way the reader's dates read. The guide's
examples show the day-first numeric wording.

**3. [Rule 1] Times carrying a clock are kept apart by semicolons.** With clauses, the times
joined by commas ran together ("which is 04:00 for Grace, Monday at 9:30"), and two times had
no separator at all. A red and green pair of its own; the rule is any comma in a time, so a
date worded with a comma is kept apart the same way.

**4. [Shape] The shape arrived with task 2's red.** `Heard`, the merge's zone and the second
wording argument came with the failing cases so they compile; nothing gave a zone and nothing
was said with it. The merge case passed at that red, so it was taken red by hand twice before
the commit, once with the precedence reversed and once with a place's zone ignored, and both
went red.

**5. [Shape] The Grace case's expectation moved.** `test_a_time_outside_somebodys_working_day_says_so_beside_the_time`
has Grace in New York, so she now hears her clock beside each time; its expectation changed in
task 2's green and again in the semicolon pair's red.

**6. [Brief] Two diagnostic commands carried do-nothing shell assignments named after banned
tools** (`sed_free=1`, and `awk_free=0` in front of a `grep`). Neither tool ran and nothing was
written by them. Read-only Python parsed `guards.toml`; `cargo fmt` formatted the Rust files.

## Ledger

Opened: 708 (todo, a time zone on a contact, decision 1), 709 (unrun-verify, phase 14: a real
colleague's zone from `getSchedule`), 710 (unrun-verify, the tester's ear: a time heard with a
guest's clock and the semicolons), 711 (todo, a question for Pratik: judge a colleague Microsoft
places by their own working hours). Closed: none. Counts 639 open, 72 fixed, 711 in all.

## Threat Flags

None beyond the register. T-13-31-01: nothing here calls ICU; `common::zones` holds it.
T-13-31-02: a custom or unknown name gives no zone, and a zone the person had is never
replaced, a test and a record. T-13-31-03: three per time, a test and a record. T-13-31-SC: no
crate and no feature added.

## Known Stubs

None.

## Self-Check: PASSED

Every commit above is on the branch, and every file listed exists.

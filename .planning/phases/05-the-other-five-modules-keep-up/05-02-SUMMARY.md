---
phase: 05-the-other-five-modules-keep-up
plan: 02
subsystem: ui
tags: [calendar, views, screen-reader, settings, guards]

requires:
  - phase: 05-the-other-five-modules-keep-up
    provides: "05-01's measurement that a moved day is shown once, which the week and month windows inherit without doing anything"
provides:
  - "CalendarView: an enum of Agenda, Week and Month with one list, one label per view, one stored word and one reading back of it"
  - "CalendarShowing: the view and the day it is anchored on, with window() and stepped() for all three views in one function each"
  - "calendar_heading: the heading names the window where one was asked for and the rows where none was, which is T-05-07's mitigation and its search exception in one place"
  - "load_module_data now takes the window, so every path that reads the calendar back says which one it wants and a path that forgets is a compile error"
  - "AppConfig::calendar_view, offered by a labelled control on the Calendar section of the Calendar and PIM tab and read back in the same commit"
  - "three guard records: one on the week step, two on the month step"
affects: [05-03, 05-04, 05-05, 05-06]

actuals:
  tokens: 128000
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A window the panel reads is a value in WxUIState handed to every reloader as an argument, the way account_id already is, rather than worked out where it is used"
    - "A heading that describes a window rather than its contents, with the no-window case naming the older rows-derived label explicitly"

key-files:
  created: []
  modified:
    - src/presentation/ui_types.rs
    - src/presentation/wx_calendar_module.rs
    - src/presentation/wx_app.rs
    - src/presentation/managers.rs
    - src/presentation/wx_settings.rs
    - src/presentation/date_display.rs
    - src/data/config.rs
    - tests/a_moved_day_is_shown_once.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/KEYBOARD_SHORTCUTS.md
    - Cargo.toml
    - .planning/WINDOWS.md

key-decisions:
  - "The week starts on Monday, fixed, not read from the locale, because the locale question is FEEDBACK-02's and half-answering it reads as a bug"
  - "A view picker went on the calendar toolbar as well as in Settings, because task 1's own done criterion says 'chosen by keyboard' and Settings alone does not reach it"
  - "The reload in managers.rs was fixed rather than recorded as a stub, and has no test of its own; the cost of one is 41 records of remeasurement for a one-line change, and the gap is ledger 197"
  - "CalendarEventItem::the_window_now was deleted, because every caller now asks CalendarShowing::window and it was reached by nothing in the running program"
  - "The month is a list over a month-wide window, not a grid. A grid is the thing PIM-06's third criterion is written against"
  - "Task 2 has no red half and could not have one: the step is one exhaustive match, so the month arm had to ship with the week arm or be a live todo!() behind a picker offering Month"

patterns-established:
  - "A break named for a property must be checked against that property's own case: a naive implementation is right on most inputs, and those are exactly the inputs it cannot guard"
  - "A commit gate judges the working tree, not the index, so a slow gate makes the tree a lock"

requirements-completed: []

coverage:
  - id: D1
    description: "A week view and a month view exist, each a narrower window over the query that already takes a window, each showing its events in date order"
    requirement: PIM-06
    verification:
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_the_week_around_a_day_is_seven_days_and_starts_on_a_monday"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_the_month_around_a_day_is_the_first_to_the_last_of_that_month"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_a_day_at_either_end_of_a_week_lands_in_the_same_week"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_a_day_in_the_middle_and_the_last_of_a_month_land_in_the_same_month"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_a_week_and_the_agenda_agree_about_a_day_that_is_in_both"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_the_month_the_week_and_the_agenda_agree_about_a_day_in_all_three"
        status: pass
    human_judgment: false
  - id: D2
    description: "Prev and Next move by exactly one period, in both directions, across a month end and a year end, and say which period they landed on"
    requirement: PIM-06
    verification:
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_the_week_before_and_after_move_by_exactly_seven_days_at_both_ends"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_a_week_across_the_end_of_a_month_or_a_year_is_still_seven_days"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_the_month_before_january_is_december_of_the_year_before"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_the_month_after_december_is_january_of_the_year_after"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_stepping_back_from_the_thirty_first_lands_in_the_shorter_month_and_forward_returns"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_the_heading_names_a_different_week_after_a_step"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_the_agenda_is_the_window_it_always_was_and_does_not_move"
        status: pass
    human_judgment: false
  - id: D3
    description: "The heading names the period that was asked for even when it holds no events, and a search, which asked for no period, keeps describing what it found"
    requirement: PIM-06
    verification:
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_the_heading_for_a_week_names_the_week_even_when_nothing_is_in_it"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_the_heading_for_a_month_names_the_month_and_the_year"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_a_search_keeps_a_heading_made_from_the_rows_it_found"
        status: pass
      - kind: unit
        ref: "src/presentation/managers.rs#test_searching_the_calendar_sends_back_the_events_it_found"
        status: pass
    human_judgment: false
  - id: D4
    description: "A repeating event falls on every date inside a narrowed window and no others, and a day moved out of its series is in the week it moved to and not the one it left"
    requirement: PIM-03
    verification:
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_a_repeating_event_in_a_one_week_window_falls_only_on_the_days_inside_it"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_a_daily_series_in_a_month_window_is_one_row_a_day_and_no_more"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_a_day_moved_out_of_a_series_is_in_the_week_it_moved_to_and_not_the_one_it_left"
        status: pass
    human_judgment: false
  - id: D5
    description: "The chosen view is stored, defaults to the agenda, falls back to the agenda for anything unrecognised, and is offered by a labelled control on the Calendar section"
    requirement: PIM-06
    verification:
      - kind: unit
        ref: "src/data/config.rs#test_a_new_installation_opens_the_calendar_on_the_agenda"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_a_settings_file_written_before_these_existed_reads_the_way_it_should"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_a_stored_view_nothing_recognises_falls_back_to_the_agenda"
        status: pass
      - kind: unit
        ref: "src/presentation/ui_types.rs#test_every_view_is_stored_under_a_word_that_reads_back_as_itself"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_every_setting_somebody_can_change_is_offered_by_a_screen"
        status: pass
    human_judgment: false
  - id: D6
    description: "Prev and Next carry a real name on both Windows accessibility channels, neither says 'not built yet', and a month of rows in one list is navigable"
    verification: []
    human_judgment: true
    rationale: "Nothing here has been heard. The names are the same on both channels by construction and neither channel has been read with a tool. Ledger 195, 196 and 199."

duration: 300min
completed: 2026-09-08
status: complete
---

# Phase 5 Plan 02: A week, a month, and two buttons that mean something

**It works, in the sense that the code runs and the tests hold it. Choosing Week
or Month narrows the calendar to that period, Previous period and Next period
move by exactly one and say which one you landed on, both buttons carry a real
name on both Windows accessibility channels, and the view you choose survives a
restart. Nobody has heard any of it, and no calendar has ever been synced from a
real account.**

## Performance

- **Duration:** about 300 minutes
- **Tasks:** 3
- **Files modified:** 13
- **Commits:** 5

## What works, and what does not

**Works, and is tested.** The window the calendar reads is whichever view is
chosen. A week is seven days starting on Monday, a month is the first to the last
of a calendar month, and the agenda is the window it has always been. Prev and
Next step by exactly one period in both directions, across a month end and a year
end, and the round trip returns where it started. The heading names the period
rather than the rows, so an empty week still says which week it is. A search keeps
describing the rows it found. A repeating event expands correctly inside a narrow
window and a moved day is in the week it moved to. The stored view defaults to the
agenda, is offered by a labelled control on the Calendar section of the Calendar
and PIM tab, is read back into the config in the same commit, and anything
unrecognised falls back to the agenda.

**Works, and is not tested.** The reload after an edit in `manage_calendar` now
asks for the window on screen. See "The reload" below; it is ledger 197.

**Works, and is not tested, second case.** The two lines at startup that put the
stored view into `WxUIState` and set the toolbar box to match. They sit inside the
frame builder, which needs a running wxWidgets window. Every piece either side of
them is tested. Ledger 198.

**Not checked at all.** Nothing has been heard. The names on both accessibility
channels are right by construction and neither channel has been read with
Axe.Windows or `scripts/msaa-names.ps1`. Whether a month in one flat list is
navigable, and whether the heading is heard once or twice when Prev is pressed,
are questions for a person with NVDA. Ledger 195, 196 and 199.

**Not closed.** PIM-06's third `[D]` line, and therefore PIM-06 itself.
`requirements-completed` is empty.

## Prev and Next, on both channels

| Button | Label, which is the UI Automation name Narrator reads | `set_accessible_name`, which is the MSAA name NVDA reads |
|---|---|---|
| Previous | `&Previous period` | `Previous period` |
| Next | `&Next period` | `Next period` |

The labels were `&< Prev` and `&Next >`, so Narrator read the punctuation, and
both accessible names ended "not built yet". Neither says that now, and both
channels carry the same words.

The View box beside them is a `Choice` with a `StaticText` label of its own,
`&View:`, and an explicit accessible name of `Calendar view`. In Settings the same
choice is built with `labelled_choice`, which every other picker on that tab uses.

**A finding while doing it, caught by the gate rather than by me.**
`test_no_two_controls_in_one_dialog_claim_the_same_alt_key` failed on
`build_calendar_panel gives Alt+N to &Next > and &Next period`. There is no such
control: the check reads the file for labels, and my comment **quoted** the old
labels with their ampersands. The comment now describes them instead and says
why. A guard that reads source for a pattern cannot tell a comment from code, and
a commit message or comment quoting the thing being removed is enough to trip it.

## The announcement, and pressing Next five times

The period is announced through `announce_topic` on the topic `calendar-period`,
which is the same mechanism `CalendarEventsLoaded` already uses for its count.
Five quick presses of Next replace one another on that topic, so what is spoken is
the fifth week and not five headings. That is the mechanism; **nobody has heard it
suppress a burst**, and it is one of the four questions in ledger 195.

`btn_today` also moved onto that topic. It previously announced at Normal priority
with no topic, so a Today pressed after four Nexts would have queued behind them.
It now re-anchors on today and reloads, rather than writing a date onto the
heading and leaving the list showing another week, which would have been the
heading naming a period the list is not showing.

## The first day of the week: Monday, and what the other answer would cost

Fixed, in `THE_WEEK_STARTS_ON`, with the reason beside it.

Windows knows which day the machine's locale starts a week on and asking it is
the answer most people expect. It would also put a second locale-shaped question
in the calendar while the first is unanswered:
`date_display::ENGLISH_ONLY` records that month names and relative wording are
English whatever the machine is set to, and FEEDBACK-02 in phase 6 is the
requirement that changes it. A week that starts where the locale says, with
"Week of 20 July 2026" written beside it in English on a French machine, is half
an answer, and half answers read as bugs rather than as limitations.

What Monday costs somebody whose week starts on Sunday: one day of offset in
which seven days a week view holds. It never hides an event, because every day is
in some week, and Prev and Next still move by exactly seven days. What the locale
answer would have cost: opening FEEDBACK-02 here, which is another phase's work.

## The month is a list, and what a grid would have cost

The panel already holds a virtual `ListCtrl` in Report mode with five columns,
and `every_day_shown` sorts by moment, so a list read from the top is in date
order by construction. A month view is that list over a month-wide window.

A grid would have cost a new control that does not exist in this panel, a
decision about what each cell says when several events fall on one day, a second
way of reading the same rows to keep in step with the list, and a reading order
that a screen reader user has to reconstruct from cell labels. PIM-06's third
`[D]` line is written against exactly that. Not building one does not answer the
criterion, it removes the grid from the question, and it is why the criterion
still needs a person.

## Which assertions were green on arrival

Three, and they should not read as new coverage.

`test_a_repeating_event_in_a_one_week_window_falls_only_on_the_days_inside_it`
and `test_a_day_moved_out_of_a_series_is_in_the_week_it_moved_to_and_not_the_one_it_left`
were red in the RED commit, and red for the **arithmetic**, not for what they are
about. `falls_on` takes a from and a to and does not care how far apart they are,
so the expansion half was already true before this plan. The moved-day half rests
on 05-01, which measured it rather than reasoning about it, and this is a narrower
restatement of it over a week window.

`test_a_week_and_the_agenda_agree_about_a_day_that_is_in_both` is the same shape.

All eight of task 2's tests were green on arrival, and that has its own section.

## Task 2 had no red half, and could not have one

The plan asks for one step function over all three views rather than a second set
of functions beside the first, and it is right. Rust then requires the match to be
total, so writing the week arm means writing the month arm. The two ways to avoid
that were a `todo!()` in the month arm, which is a live stub behind a picker that
offers Month, and a wrong answer, which is worse. So the month arithmetic shipped
in task 1's GREEN commit and task 2's eight tests were green the moment they were
written.

There is no ordering of these two tasks in which task 2 could have had a genuine
red. The substitute is the one this project already uses and 05-01 used before me:
the red half is a **measured break** on the finished code, recorded in
`guards/guards.toml`, and the commit says so. What should change is the task
boundary, not the discipline: the week and the month are one task.

## Guard records: three, and the plan's one was pointed at the wrong thing

**Task 1's record**, the week step answering the day it was already on, was
measured twice. The first measurement reddened **one** test, the one comparing
the dates by hand. Nothing asked whether the words somebody hears after pressing
Prev had changed, so a step that moved nothing would have redrawn the same week
under the same heading and only a hand comparison of two dates would have caught
it. `test_the_heading_names_a_different_week_after_a_step` was written for that,
taken red against the same break, and the break measured again. Two tests.

**Task 2's records are two, where the plan asked for one, and the reason is
sharper than 05-01's version of the same finding.** The plan said to guard "the
month step landing in the right month across a year boundary" by making the step
add and take off thirty days. Measured, that break reddens one test and it is
neither of the year-boundary ones:

- thirty days back from 14 January is 15 December, which is the right month
- thirty days on from 14 December is 13 January, which is the right month

The naive step is **correct** across a year boundary. What it gets wrong is the
day the shorter month does not have. So that record is named for what it really
guards, the thirty-first, and a second break was found for the boundary: taking
December's next month as January of the same year, in the helper that decides
which month is next door. That reddens both boundary tests.

05-01 found that a break reddening fewer tests than expected is something to
read. This is a different cause of the same symptom. There, the surviving tests
sat at another layer and built their inputs downstream of the break. Here the
break reaches the code perfectly well and happens to give the right answer.
**A wrong implementation is not wrong everywhere, and the inputs where it is
accidentally right are exactly the ones it cannot guard.**

Line 80's census went 474 to 477 across two commits, 666 records became 669, and
192 + 477 = 669 holds.

## The reload in managers.rs: fixed, without a test, and why

The plan offered two options and priced them: fix it with a test, or record it as
a stub. I did a third thing, which the plan did not offer, and it is here rather
than implied by the absence of a test.

**Fixed.** `manage_calendar` reloads with `the_calendar_on_screen(state)` instead
of the whole eighteen months, so saving an event in a week view no longer puts
somebody back in the agenda with nothing said. The change is one line plus a
named helper.

**Without a test.** The reload lives inside `manage_calendar`, which opens a
wxWidgets dialog and needs a `Frame`, so a test would first have to extract it
into a function of its own. Any test of it would live in `managers.rs`, which 41
guard records fingerprint. At roughly 119 seconds a record that is about **80
minutes** of builds and full library runs on the critical path, for a one-line
change whose arithmetic already has eleven tests one call away.

**Why not the stub branch.** Recording it as a stub means leaving the defect in:
saving an event in a week view silently reverting to the agenda is a defect this
plan would have introduced. Fixing it and recording the untested join is strictly
better than not fixing it and recording the defect. It is ledger 197.

`load_module_data` takes the window as a parameter for the same reason, and that
is the part that has teeth: every one of the twenty-one call sites had to say
which window it wanted, so a path that forgets is a compile error rather than a
calendar that springs back to the agenda. Seven of those sites are the reloads in
`managers.rs` after a command.

The pre-existing difference premise 8 warns about, that the load path filters out
hidden calendars and the reload does not, was left alone.

## Test counts before and after

| file | records fingerprinting it | before | after |
|---|---|---|---|
| `src/presentation/ui_types.rs` | 4 after this plan, 1 before | 56 | 78 |
| `src/data/config.rs` | 3 | 58 | 59 |
| `src/presentation/wx_app.rs` | 47 | 199 | **199** |
| `src/presentation/managers.rs` | 41 | 137 | **137** |
| `src/presentation/date_display.rs` | 3 | 37 | **37** |
| `src/presentation/wx_settings.rs` | 2 | 0 | 0 |

No `#[test]` was added to `wx_app.rs`, `managers.rs` or `date_display.rs`, which
are the expensive files. Three existing tests were changed rather than added:
`test_searching_the_calendar_sends_back_the_events_it_found` in `managers.rs` now
asserts the search asks for no window, `events_drawn` in `wx_app.rs` reads the new
variant, and `test_a_settings_file_written_before_these_existed_reads_the_way_it_should`
in `config.rs` covers the new key. Changing a test body does not move the count,
so none of them flagged a record.

`a_month_in_words` went into `date_display.rs` with no test of its own, tested
through `calendar_heading` in `ui_types.rs` in both wordings. Month-and-year
wording is that module's rule and belongs there; the test is where the plan asked
tests to go and where it costs three records fewer.

## The scoped remeasures, all of which were run

Three, all detached with `WIXEN_TEST_THREADS=4`, twelve records in total, and
**every one still reddens exactly the tests it names**. Not one came out short.

1. After task 1's GREEN, one record: `a row stops knowing it is a day of a series
   that was changed on its own`.
2. After task 2's tests, four: that one again plus the three new ones.
3. After task 3's GREEN, seven: `a stored setting that no screen offers is
   caught`, `a settings file written before reading was a setting still loads`,
   `the move window stops asking where the last one went`, and the four above.

## The free red half, quoted

Task 3's red came from adding the field and nothing else:

```
1 setting(s) are stored and survive a restart and no screen offers any of them,
so nobody using this can reach them:
  calendar_view
Add a labelled control, or say in one of the three lists above this test why
there is not one.
```

**Its neighbour did not fail, and that is worth reporting rather than being
pleased about.** `test_every_setting_somebody_can_change_is_read_by_something`
passed while nothing read `calendar_view`. Its second hop credits the last
`pub fn` seen above any line naming the setting, and `fn default_calendar_view`
sits below `pub fn`s in `config.rs`, so a reader was offered for a setting nobody
read. The GREEN half names the field outright in `wx_app.rs`, so the question is
moot here; the weakness in the second hop is not, and it is not this plan's to fix.

The two lines that do the showing and the reading back:

```rust
CalendarView::from_stored(&config.calendar_view).offered_at(),   // wx_settings.rs, showing
cfg.calendar_view = CalendarView::offered_at_entry(w.calendar_view.get_selection())
    .stored().to_string();                                       // wx_settings.rs, reading back
```

## The keyboard

**No key was added.** The four calendar toolbar controls sit at the top of the
calendar panel and are the first thing `Tab` reaches there, and each carries an
`Alt` letter in its label: Today, Previous period, Next period and View. A row of
four at the top of a panel does not need a global accelerator, and adding four
would take four combinations out of a keyboard that already carries a lot.

`docs/KEYBOARD_SHORTCUTS.md` gained a paragraph beside the calendar sidebar one
saying exactly that, and saying where the stored default lives, because "reachable
by Tab" is only reachable if somebody knows the row is there.

## Deviations from plan

**1. [Deviation, deliberate] A view picker went on the calendar toolbar, which the
plan did not ask for.**
- **Found during:** Task 1, reading the done criterion.
- **Issue:** Task 1's done line is "Somebody can choose a week ... by keyboard",
  and task 1's file list has no settings file. With the picker only in Settings,
  which is task 3, nothing in task 1 could choose a week at all, and Prev and Next
  would have been enabled in a view nobody could enter.
- **Fix:** A `Choice` in the calendar toolbar, reading `CalendarView::OFFERED` so
  it and the Settings picker cannot drift. Settings holds the one the calendar
  **opens** on, which is what truth 4 asks for.
- **Verification:** `test_every_view_is_offered_under_a_name_somebody_would_recognise`,
  and the accelerator and label guards in `tests/wired.rs`.

**2. [Rule 1 - Bug this plan would have introduced] `btn_today` re-anchors and
reloads, where it used to write a date onto the heading.**
- **Found during:** Task 1.
- **Issue:** With views, a Today that only writes today's date onto the heading
  leaves the heading naming a day the list is not showing, which is T-05-07
  arriving through the button this plan did not touch.
- **Fix:** Today re-anchors on today, keeping the view, and reloads through the
  same closure Prev, Next and the picker use. It also moved onto the
  `calendar-period` topic so it replaces a burst rather than queueing behind it.

**3. [Rule 2 - Missing critical] `CalendarEventItem::the_window_now` was deleted.**
- **Found during:** Task 1, after the wiring.
- **Issue:** Every caller now asks `CalendarShowing::window`, so it was reached
  only from its own test. A public function nothing in the running program reaches
  is the first guardrail's case.
- **Fix:** Deleted, its test retargeted at `CalendarShowing::agenda_now().window()`,
  and a comment left at `the_window_around` saying it went and why. One doc comment
  in `tests/a_moved_day_is_shown_once.rs` that named it was corrected.

**4. [Deviation, forced by the language] Task 2 carries no red half and no version
bump.**
- **Issue:** Covered above. The month arm shipped with the week arm because the
  match must be total.
- **Fix:** Task 2 is a test-only commit whose red is two measured breaks. No bump,
  because it changes no software. The changelog entry from task 1 named only the
  week although the month shipped in the same commit, and task 2 completes it
  rather than leaving it describing half of what landed.

**5. [Deviation from the plan's premise 11] `ui_types.rs` carried one guard record,
not zero.**
- **Issue:** The plan says `ui_types.rs` is fingerprinted by zero records and
  becomes one when task 1 adds its own. Measured on this tree it was already one,
  from 05-01, which landed after the plan was written. The plan's correction of
  2026-09-08 is right about the mechanism and one short on the count.
- **Fix:** No change to the work. It cost one extra record per remeasure.

**6. [Correction to the plan's task 2 action] The prescribed break does not guard
the property it was prescribed for.**
- Covered under "Guard records" above. Two records where the plan asked for one.

**Total deviations:** 6. One control added because the plan's own done criterion
needed it, two fixes for defects this plan would otherwise have introduced, one
forced by exhaustive matching, and two corrections to figures and pairings the
plan had not measured. No scope creep.

## Issues encountered

**The commit gate judges the working tree, not the index.** Task 1's GREEN commit
was refused because I began writing task 2's tests while its six-minute gate ran,
and eight test functions arrived in a file the count check reads. Nothing was
wrong with the index or with the commit that eventually landed. The recovery was
to copy the working file aside, `git checkout-index -f --` the indexed version
back, commit, and copy the file in again, checking the carriage-return count both
ways. Staging snapshots content for the commit and not for the hook, and a slow
gate makes the working tree a lock with one writer.

**A guard that reads source cannot tell a comment from code**, covered under
"Prev and Next" above.

**The plan was written against `main` at `9611b70` at version 0.75.0 with 632
records. The tree was at 0.92.0 with 666.** Every per-file record count in premise
11 was still exactly right when re-measured except `ui_types.rs`, which had gained
one, and `wx_app.rs` and `managers.rs`, which had gained five and one. Counting
records by parsing `tests_last_seen` rather than grepping a file name is what
makes those figures survive; only the totals move much.

## Version and ledger

0.92.0 went to 0.93.0 in task 1's GREEN commit, with the week and the month, and
to 0.94.0 in task 3's, with the stored setting. Task 2 carries no bump.

The ledger ended at 194 and ends at 199. Five entries, one per unrun thing:

- **195** the week view has never been heard, four questions
- **196** neither accessibility channel has been read with a tool
- **197** the reload in `managers.rs` is fixed and untested, with the price of a test
- **198** nobody has chosen a view, restarted, and come back to it
- **199** PIM-06's third `[D]` line, and whether a month in one list is navigable

## Next plan readiness

Nothing here blocks `05-03` onwards. `CalendarShowing` is in `WxUIState` and every
reloader takes it, so any later plan that reads the calendar back inherits the
window without doing anything.

Two things are owed and neither blocks a merge.
`scripts/guards.sh --touched-by ce3da2c` belongs to the phase-8 sweep, and the
three new records have never been through a sweep, which the census now says.

## The merge

`scripts/check.sh all` passed on the branch at `f7dcec6`: formatting, clippy with
`-D warnings`, the whole suite and the release build. Merged to `main` at
`2e06c12` with a merge commit. `main` is at version `0.94.0` with 669 guard
records, and it is **not pushed**.

The five task commits:

1. `7ec9878` RED, the week arithmetic and the heading rule, eleven tests plus the
   count check named
2. `e94b4ae` GREEN, the week, both buttons, the picker, the wiring, one guard
   record
3. `022cfa3` the month, eight tests green on arrival, two guard records
4. `1a0cc5e` RED, the stored view, four tests plus the count check named
5. `13a254c` GREEN, the settings control, the read-back, the startup wiring

## Self-Check: PASSED

Every file this summary names exists on disk, and all five commit hashes are in
`git log`. No carriage returns and no em dashes in this file.

---
*Phase: 05-the-other-five-modules-keep-up*
*Completed: 2026-09-08*

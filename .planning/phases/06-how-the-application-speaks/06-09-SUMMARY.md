---
phase: 06-how-the-application-speaks
plan: 09
status: complete
subsystem: application
tags: [due, reminders, tasks, events, kind, identity, seam, hold-table, event-alerts, one-window, details, guards, checkpoint]

requires:
  - phase: 06-how-the-application-speaks
    provides: "06-05's `raise_what_is_due` with `BetweenLooks`, `say`, the repeat rule and the typing helper, read by the names its summary records; 06-03's catalogue at `locales/en-US/dates.ftl`, where relative wording lives; 06-04's pattern of a disabled control whose label says why"
  - phase: 05.1-notes
    provides: "`docs/development/the-notes-seam.md`, the identity rule and the table of where a contract knows it has assumed one implementation, which `Kind`'s doc comment copies one level down"
provides:
  - "`due::Kind { Reminder, Task, Event }` with `ALL`, `word`, `key`, `from_key` and `can_be_done`, a total `From<Kind> for ItemKind`, and a doc comment that is the seam for a fourth kind"
  - "`due::Identity { kind, id }`, opaque, composed by a kind's own feed and taken apart by nothing; `CalendarEventItem::due_identity()` composes an event day's from the series' id and the day's start"
  - "`due::Candidate`, `due::what_is_due(candidates, now, already, held)`, `due::when_a_day_alerts`, `due::when_an_event_alerts`; `date_display::how_soon`, `how_long_ago`, `time_of_day`; `Message::{InMinutes, InHours, InDays}`"
  - "`held_alerts`: `hold_alert(kind, id, until)`, `held_alerts()`, `let_go_of_holds_that_ended_before(moment)`, one additive table keyed by the kind's word and the identity; `tasks::complete_task(id, stamp) -> usize`, which sets done and never toggles"
  - "`wx_reminder_alert`: `raise(parent, rows, now, dates, a11y, default_snooze, editors) -> Vec<(Identity, Answer)>`, `say(rows, ..)`, `sentence_for`, `Rows`, `Editors`, `NoEditors`, `DueWindow`, `build_reminder_alert_dialog(..) -> DueWindow`, `Answer::Edited`; `Spoken` is gone"
  - "`event_alerts`: `NO_ALERT`, `StoredAlert::{Lead, Off, Unknown}`, `read`, `lead_to_raise_at`, `from_google`, `from_microsoft`: the column's meaning, and off as a value of its own"
  - "`managers::change_an_opened_event`, `an_event_editor`, `change_an_event_from_a_row`: the calendar window's edit path, shared with the due window's Details"
  - "`wx_app`: three feeds, `reminders_that_might_be_due`, `tasks_that_might_be_due`, `events_that_might_be_due`, the hold read by `what_is_held`, one window per look, answers written back by kind, `TheEditors`; `WxUIState::default_event_alert_lead` and `UIUpdate::DefaultEventAlertLeadChanged`"
affects: [version-2, phase-8]

actuals:
  tokens: 64700
  tasks: 4
  commits: 15

tech-stack:
  added: []
  patterns:
    - "A kind of its own for a closed set that can grow, with the doc comment naming every place the code assumed the current members, so the next member is a variant plus the compile errors it produces"
    - "An identity of a kind and an opaque string, composed by the feed that knows the row and parsed by nobody; whoever needs the row back keeps the row beside the identity rather than reading the string"
    - "A rule that takes its hold as an argument and is tested with a map, and a table keyed by the kind's stored word so a fourth kind is a fourth word and an unknown word is kept"
    - "A window's bookkeeping as a value the handlers push into, tested without a display, with the live test reading the real controls for what only they can say"
    - "A column whose absence meant three things gets a value for the one meaning the writers know, in a module of its own with no records, and the writers call it; the readers of the old absence are counted before anything is designed"
    - "Where a test's cost depends on the file it lives in, the behaviour moves to a module that costs nothing to test and the expensive file's untested branch is written down"

key-files:
  created:
    - src/data/message_cache/held_alerts.rs
    - src/application/event_alerts.rs
    - tests/the_due_window_holds_every_kind.rs
  modified:
    - src/application/due.rs
    - src/presentation/date_display.rs
    - src/common/catalogue.rs
    - locales/en-US/dates.ftl
    - src/data/message_cache/mod.rs
    - src/data/message_cache/tasks.rs
    - src/presentation/wx_reminder_alert.rs
    - src/presentation/wx_app.rs
    - src/presentation/managers.rs
    - src/presentation/ui_types.rs
    - src/application/calendar.rs
    - src/application/mod.rs
    - src/presentation/scan_fixtures.rs
    - src/presentation/scan_target.rs
    - src/presentation/accessibility/feedback.rs
    - tests/theme_reach.rs
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - guards/guards.toml
    - .planning/REQUIREMENTS.md
    - .planning/WINDOWS.md
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "Kind is its own three-variant enum and not ItemKind, whose Contact and Note would make a contact representable as due"
  - "Decision 1, Pratik's: a date-only task and an all-day event's alert base are due at working_day_starts; a date-only reminder stays at midnight and the disagreement is ledger 436"
  - "Decision 2, Pratik's, with the guard: a stored alert as stored, an explicit off as silence, nothing stored as the default lead; off became an empty list, written by the three writers that know it and by no other"
  - "Decision 3, Pratik's: held_alerts, its own additive table, kind as a word, unknown kept, pruned at each look"
  - "Details opens an event in the calendar window's own editor, pulled out of manage_calendar; a task or a reminder has no editor anywhere in the program, so on those rows Details is disabled with the reason in its label and ledger 437 says what it takes"
  - "The count sentence and the list's name are English plurals in code, because the catalogue holds one area; ledger 440"

patterns-established:
  - "A red commit that reshapes a type says which of its new tests were green on arrival and why, and each of those earns its place by a measured break before the plan ends"
  - "The count check names distinct records, not per-file sums, and counts by its own rule: the number to write is the one it prints"
  - "Before moving a block of code, search the tests and the guard registry for readers that locate it by text; they move with it"

requirements-completed: [FEEDBACK-01]

coverage:
  - id: D1
    description: "A Due knows its kind and says it first; what_is_due obeys a hold by identity, refuses an ended event, and hands rows back earliest first; the alert instants are pure"
    requirement: FEEDBACK-01
    verification:
      - kind: unit
        ref: "cargo test --lib -- application::due:: common::catalogue:: presentation::date_display::"
        status: pass
    human_judgment: false
  - id: D2
    description: "A snoozed task or event is held in a table a restart remembers, replaced by a later snooze, let go when it ends, kept under a word this build does not know; done on a task is done and never toggles"
    requirement: FEEDBACK-01
    verification:
      - kind: unit
        ref: "cargo test --lib -- data::message_cache::held_alerts:: data::message_cache::tasks::"
        status: pass
    human_judgment: false
  - id: D3
    description: "One window holds one row per due thing in the order given, each row its sentence with the kind first, six buttons with six distinct Alt keys, Mark Done and Details disabled with the reason in the label for the kinds they are not for; the pure bookkeeping answers one row or every row and dismisses the rest at close"
    requirement: FEEDBACK-01
    verification:
      - kind: unit
        ref: "cargo test --lib -- presentation::wx_reminder_alert::"
        status: pass
      - kind: integration
        ref: "cargo test --test the_due_window_holds_every_kind"
        status: pass
    human_judgment: false
  - id: D4
    description: "Off has a stored form and the three writers that know it write it; the due window gives the default only to silence; a day of a series is its own identity; the three feeds go through one rule into one window and the answers go back by kind"
    requirement: FEEDBACK-01
    verification:
      - kind: unit
        ref: "cargo test --lib -- application::event_alerts:: presentation::ui_types::test_two_days_of_one_series"
        status: pass
      - kind: integration
        ref: "cargo test --test wired"
        status: pass
    human_judgment: false
  - id: D5
    description: "A task due today, an event fifteen minutes off and a reminder arrive together in one window under a screen reader, each row saying its kind first, and the answers do what the labels say"
    requirement: FEEDBACK-01
    verification: []
    human_judgment: true
    rationale: "Every assertion above proves structure. Nobody has heard 'Task due today' or 'Event in 15 minutes', the count sentence, the list's name, the disabled labels, or the tone over a list; WINDOWS.md 431 to 433"

duration: "task 1 on 2026-09-14 before the checkpoint; tasks 2 to 4 about 2 hours 20 minutes, 16:05 to 18:25 UTC the same day"
completed: 2026-09-14
---

# Phase 6 Plan 9: One window for every due thing, each row saying its kind first

**Tasks and calendar events come due for the first time, in the same window as reminders. One window, "Due now", holds everything that has come due at a look, earliest first, one row each, each row beginning with its kind. A row can be snoozed for a chosen length, dismissed for the session, marked done where done means something, or opened in its editor where one exists; every row can be snoozed or dismissed at once. A snoozed task or event waits in a table a restart remembers. An event's stored alert can now say off, and off is never filled with the default. Version 0.124.0. Nobody has heard any of it.**

Task 1 on branch `a-due-thing-says-what-it-is-first`, merged at `6d57a49b`, stopped at the checkpoint. Tasks 2 to 4 on branch `one-window-for-every-due-thing` from `main` at `9a0b8e63`: eleven commits, `7225f76e` to `41f96882`, and one lock-file commit `e527b6ca` for an advisory published the same day, merged into `main` at `abaa0667` with the whole gate green on the merge, 7,677 passed and none failed. Nothing pushed.

## The three answers, and how each was applied

Pratik answered the checkpoint on 2026-09-14. His words are quoted where each answer landed.

**Decision 1, the hour.** "At the beginning of the day as set in the settings. If 8:00 is the start time, then that is when the task should be due. The snooze timer should let the user postpone it should it be necessary. Allow the user to open the task/event/reminder by using the 'details' button to reassign date/time."

Applied: `tasks_that_might_be_due` at `wx_app.rs:10609` hands `when_a_day_alerts(day, hour)` the hour from `s.working_day.starts`, the setting on the Calendar and PIM tab, nine by default. `events_that_might_be_due` at `:10668` turns an all-day event's whole-day start into that day at the same hour before handing it to `when_an_event_alerts`, so a fifteen-minute lead on an all-day event fires at a quarter to nine on the day and not the night before. A date-only reminder still fires at midnight, unchanged, and the disagreement is ledger 436. Snooze is per row, with the length read when the button is pressed so two rows can take two lengths. The Details button is new and is his: it is on the window, it opens an event in its own editor, and for a task or a reminder it is disabled with its reason in the label, because nothing in this program edits an existing one; the section on Details below says what was found.

**Decision 2, the event alert.** "Option C should be used but we must make sure that the event actually has a reminder."

Applied as three states, not two: `event_alerts::lead_to_raise_at` at `event_alerts.rs:72` answers a stored lead as the lead, an explicit off as no row at all, and nothing stored as `default_reminder_minutes`. The default fills silence and never overrides an off. For that to be true the column had to be able to say off, and it now can: an empty list, `NO_ALERT`, written by the three writers that know the alert is off. The writer and reader census is below.

**Decision 3, the hold.** "Its own table so that we get the most functionality."

Applied: `held_alerts`, `CREATE TABLE IF NOT EXISTS held_alerts (kind TEXT NOT NULL, id TEXT NOT NULL, until TEXT NOT NULL, PRIMARY KEY (kind, id))`, at `mod.rs` beside the reminders table, nothing dropped or renamed. Kind stored as the word `Kind::key` gives, an unknown word kept and read back like any other, expired rows let go at each look. A test writes a row under `mail`, prunes with an earlier moment, and finds it still there.

## What landed, task 1

**`Kind`.** Three variants, `Reminder`, `Task`, `Event`; `ALL`; `word` ("Reminder", "Task", "Event"); `key` ("reminder", "task", "event"); `from_key`, which answers `None` for a word this build does not know; `can_be_done`, yes for a reminder and a task and no for an event. A total `From<Kind> for ItemKind`, for `pim_command::no_longer_there`.

**`Identity { kind, id }`**, derived `Eq` and `Hash` over both fields. Composed by a kind's own feed and taken apart by nothing.

**`Candidate`**, what a feed hands in: identity, title, `raise_at`, `when`, `ends`, `done`. **`Due { identity, title, when, late }`** and `spoken`, a match on the kind:

| Kind | On time | Late |
|---|---|---|
| Reminder | `Reminder: Call the bank, due July 26, 2026 at 9:00 AM` (unchanged) | `Reminder, overdue: Call the bank, was due July 26, 2026 at 9:00 AM` (unchanged) |
| Task | `Task due today: File the report` | `Task overdue: File the report, was due July 25, 2026` |
| Event | `Event in 15 minutes: Standup, at 3:00 PM`; `Event now: Standup` within a minute; `Event in 15 minutes: Conference` for an all-day event, which has no clock | `Event started 10 minutes ago: Standup` |

**`what_is_due(candidates, now, already, held)`** refuses a done row, anything before its `raise_at`, anything more than `STILL_WORTH_RAISING` past it, anything in `already`, anything whose hold has not run out, and an event whose end is at or before now; sorted by the moment each row is about. **`when_a_day_alerts(day, hour)`** and **`when_an_event_alerts(start, lead)`** are pure, through `common::moment`. **`date_display::how_soon`, `how_long_ago`, `time_of_day`**, and the three catalogue messages `dates-in-minutes`, `dates-in-hours`, `dates-in-days`.

## The `Kind` doc comment, quoted

> A mail message somebody asked to be told about later is the fourth kind, after version 1. It arrives by adding a variant here and answering the compile errors, which come in this order:
>
> 1. `Kind::word`, the word said first in its row, and `Kind::key`, the word it is stored under, which `Kind::from_key` reads back. A stored word this build does not know is nobody's kind and is kept rather than dropped, on `AddressBook::Other`'s reasoning.
> 2. `Kind::can_be_done`, whether done means something for it.
> 3. `Due::spoken`, its sentence, with the word first.
> 4. The `From` into `ItemKind`, for the announcement that a row has gone.
> 5. An alert-instant function of its own beside `when_a_day_alerts` and `when_an_event_alerts`, if the moment it is raised at is not the moment it is about.
>
> The identity string on a `Due` is composed by the kind's own feed and read back by the kind's own writers, and nothing else takes it apart.

The table of where the module knows it has assumed three kinds names `Due::spoken`, `Kind::can_be_done`, the two alert-instant functions, and the granularity rule in `what_is_due`. Since task 4 a fourth kind also answers `TheEditors::has_one_for` in `wx_app.rs`, the writer's two matches there, and `Kind::key` for the hold table, each of which is an exhaustive match with no wildcard and so a compile error.

## What landed, task 2: somewhere for a hold to live, and done that does not toggle

Red `7225f76e`, green `2bb52688`, records `141f8374`.

**`held_alerts.rs`**, new, on the pattern of `reminders.rs`: `hold_alert(kind, id, until)` at `:38`, an upsert that replaces the moment because the latest snooze is the one somebody meant; `held_alerts()` at `:51`, every row as written, soonest ending first, the kind a word and not read; `let_go_of_holds_that_ended_before(moment)` at `:76`, a `DELETE WHERE until < ?1` on the text's own order, which for the fixed-width form `due::stored` writes is time order. A hold ending at exactly the moment is kept one more look, so the table and `what_is_due`'s "held until exactly now is held no longer" agree about the boundary without the table knowing the rule. The module doc says why a reminder is not in the table. The schema template builds the table once per process, as it builds everything else.

**`complete_task(task_id, stamp)`** at `tasks.rs:642`, beside the toggle: `UPDATE tasks SET is_completed = 1, completed_at = ?1, updated_at = ?1, pending = 1 WHERE id = ?2 AND NOT is_completed`, returning the rows changed. Called twice it reports one then nought, and the second call does not move the stamp of the first. T-06-35 mitigated: a task the phone finished between the look and the answer is not un-finished and sent back.

Six tests red on the todo bodies, then green: five in the new file against a `tempfile` cache, one in `tasks.rs`. The count check named the four records on `tasks.rs`, 21 to 22.

## What landed, task 3: one window, every due thing a row

Red `74ddcc2c`, green `9cd1259c`, records `bf10f978`.

**The window**, `wx_reminder_alert.rs`, module name kept because 06-05's records fingerprint it and Pratik calls it the reminder window; its module doc says every kind is a row here. Title `Due now`, because "Reminders" is the name of a module this window is not. A `ListBox` of rows in the order given, each row's text that `Due`'s whole sentence, named by `what_the_list_is_called`, "1 thing due" or "3 things due"; the "Come back in" `Choice` of `Snooze::ALL`; six buttons whose labels carry six distinct Alt keys, checked by `alt_key_of` over `EVERY_BUTTON`:

| Button | Label | Key |
|---|---|---|
| Snooze | `&Snooze` | S |
| Snooze all | `Snooze &all` | A |
| Mark Done | `Mark &Done`, or `Mark &Done: not for an event` disabled | D |
| Dismiss | `D&ismiss` | I |
| Dismiss all | `Dismiss a&ll` | L |
| Details | `De&tails`, or `De&tails: not for a task yet` disabled | T |

The list has focus when the window opens with the first row selected, and Snooze is the default button, so Enter is the answer that keeps the row. Every control carries a real label or a `set_accessible_name`; no `set_name` anywhere.

**The bookkeeping**, `Rows` at `:303`, with no window: `answer(index, answer)` answers the selected row and takes it off, refusing done for a kind that cannot be done with `Answered::NotForThisKind` and the row kept; `answer_every_row` for snooze all and dismiss all, refusing done; `now_stands(index, row)` for after the editor, replacing the row or answering it `Edited` and taking it off; `closing()` dismisses what is still listed. The handlers push into an `Rc<RefCell<Rows>>`; the window closes with `ID_EVERY_ROW_ANSWERED` when the last row goes and Escape or the title bar dismiss the rest.

**`Editors`**, a trait at `:55`: `has_one_for(kind)` and `open(parent, row) -> Option<Due>`. The window asks the first once at build for each kind and relabels Details on every selection change; the Details handler drops its borrow of the rows before calling `open`, because the editor runs the event loop inside itself. `NoEditors` is what the theme check and the scan fixture hand in.

**The sentence for several rows**, `sentence_for` at `:139`: one row is its sentence exactly as before; several are "3 things due." then each row's sentence up to `MOST_ROWS_SAID`, which is three, then "And 2 more." T-06-40's ceiling. `say(rows, ..)` takes a slice; `Spoken` is gone, because `raise` no longer says anything: the caller says the unsaid rows and `raise` shows them. That is the one name 06-05 recorded that this plan changed, and `raise`'s signature with it.

**The live test**, `tests/the_due_window_holds_every_kind.rs`, builds the real dialog with one row of each kind and reads back: three rows in the order given, each the row's sentence and each beginning with its kind's word; Mark Done disabled with "not for an event" in its label and Details enabled on the event row, Mark Done enabled with its plain label and Details disabled with "not for a task" on the task row, through `DueWindow::select`, which is the selection handler's own function; six buttons each with an Alt key, all six distinct. Red against the todo bodies, green on the first run against the real window. `tests/theme_reach.rs` builds the window with one row and `NoEditors`; the theme module paints no `ListBox`, so the list is left to Windows like every `Choice` and the dialog is the only site, and its comment says so.

**The scan target** `reminder` opens on three rows through `scan_fixtures::due_rows()`, so the next Accessibility run reaches the list and all six buttons; `scan_target.rs`'s doc says why the name stayed. **`docs/KEYBOARD_SHORTCUTS.md`** gains "The Due Now Window" in the same commit as the labels.

**One thing the plan's reading did not expect.** `tests/wired.rs` reads each `with_id(ID_X)` to know something raises the id its handler answers, and a loop over an array of ids hid every one of them; the six buttons are built one by one, with a comment saying why.

## What landed, task 4: three feeds, the answers by kind, off as a value, and the seam

Red `eba042b5`, green `731ed211`, wiring `952a0f6d`, records and ledger `41f96882`.

**The writers and readers of `reminders_json`, counted before anything was designed.** Writers outside tests: Google's pull at `calendar.rs:2580`, Microsoft's pull at `:3061`, the editor's edit path through `alerts_with_the_first_at` at `managers.rs`, a new event through `event_entry`, a new event through the item form, and the CalDAV carry-over which copies the local copy; and fifteen sites across `caldav_sync.rs`, `answered_meetings.rs`, `outlook_data_file.rs` and others that write `None` for rows they build, none of which knows anything about an alert. Readers: Google's push at `calendar.rs:2909`, Microsoft's push through `reminder_lead_minutes` at `:3363`, `alerts_with_the_first_at` again, and `ui_types::first_reminder_minutes` for the editor's box; the feed is the fifth. What each reader does with an empty list, checked: Google's push filters it to no reminders field, so off written here is not sent to Google and Google keeps what it held, ledger 435; Microsoft's push reads no lead and sends `isReminderOn: false`, which is right; `alerts_with_the_first_at` had answered an empty list by the box alone, which would have turned off back into nothing, and now keeps off under nought in the box; `first_reminder_minutes` shows no alert, which is right.

**What off became.** `event_alerts.rs`, new, at zero records: `NO_ALERT = "[]"`, `StoredAlert::{Lead, Off, Unknown}`, `read` at `:49`, `lead_to_raise_at` at `:72`, `from_google` at `:87`, `from_microsoft` at `:105`. Google's pull calls `from_google`: overrides as sent, `useDefault` false with no overrides as off, `useDefault` true as nothing. Microsoft's pull calls `from_microsoft`: on with a lead as one alert, off as off, on at nought and nothing said as nothing, as before. The editor writes off when the last alert is taken away and when a new event is saved with nought in a box that opened filled from Settings, through `an_alert_or_off`; nothing stored with nought in the box stays nothing stored, because the box showed nought for want of anything to show. **This did not grow into three sync paths and their tests.** Two pulls changed by one call each, one editor function by two arms; no test was added to `calendar.rs` or `managers.rs`, whose 72 and 50 records were the boundary the brief drew: two existing tests in `managers.rs` were flipped and renamed, one table in `calendar.rs` was rewritten, and one Google pull test gained an assertion. CalDAV stays unknown and gets the default; ledger 434.

**The feeds**, three small functions in `wx_app.rs`, each reading and returning candidates: `reminders_that_might_be_due` from the state as before; `tasks_that_might_be_due` at `:10609`, every source including the local account, not done, due yesterday or today, raised at the working-day hour; `events_that_might_be_due` at `:10668`, `events_that_could_fall_between` over yesterday to tomorrow per source, hidden calendars left out through `CalendarContainerItem::hidden_among` and `is_showing` as the calendar leaves them out, each stored event's lead from `lead_to_raise_at` and no lead meaning no row, each day from `shown_days` its own candidate with `day.due_identity()` at `:10734` as its identity and the row kept beside it for Details, an all-day day's alert base at the hour and its end at the end of its last day. `what_is_held` at `:10756` lets go of what has ended and reads the rest into the map, leaving a hold under an unknown word in the table. All three go through `what_is_due` into one window.

**The look.** `raise_what_is_due` at `:10871` takes the turn first as before; reads the state once for the reminders, the sources, the hour and the default lead; builds the candidates with one `Instant` around the reads; asks the moment once; says the unsaid rows through `say` whatever the moment is; holds the window only while every row is still within `LOOKS_TO_HOLD_A_REMINDER_WINDOW_WHILE_SOMEBODY_TYPES`, counting looks per identity in `said_and_waiting`; otherwise marks every identity in `already`, `for item in &due`, before `wx_reminder_alert::raise(`. The readings in `tests/wired.rs` see the turn before the loop and the identity written down before the window, unchanged; neither reading was edited.

**The answers, by kind.** `grep -n 'due::Kind::' src/presentation/wx_app.rs` shows the writer's two matches, no wildcard: `Done` at `:11062-11067`, `Reminder => complete_reminder`, `Task => complete_task`, `Event => continue` with the comment that the window refuses it; `Snoozed` at `:11076-11085`, `Reminder => snooze_reminder`, `Task | Event => hold_alert(kind.key(), id, until)`, which is T-06-36: no arm writes a task's date or an event's start. `Dismissed` writes nothing and stays in `already`; `Edited` writes nothing and comes out of `already`, so a thing moved to later can come round this session. `Ok(0)` says `no_longer_there(ItemKind::from(kind), title)`. The refresh of `s.reminders` is kept, and `s.tasks` is marked done in place so an open Tasks panel agrees.

**Details.** `TheEditors` at `:10789` answers `has_one_for` with an exhaustive match, `Event => true`, `Task | Reminder => false`, because the search found one editor: the calendar window's Edit Event opens the item form on `filled_from_calendar_item` and writes back through an arm inside `manage_calendar`; nothing in the program opens an existing task or reminder, `PimCommand` has no Edit, and no writer takes a `Filled` onto a `TaskEntry` or `ReminderEntry`. That arm is now `managers::change_an_opened_event`, with `NotChanged::{Refused, Failed}` where the arm used `send_refusal` and `failures.push`, and the editor closure is `an_event_editor`; `manage_calendar` calls both, so the calendar window and the due window cannot disagree about what a change to a series means. `change_an_event_from_a_row` asks `which_days_are_meant` nested under the due window, refuses what the calendar does not allow, opens the editor, writes, and returns the sentence, which `TheEditors::open` announces; then it runs the event feed again from the cache and hands the window the row as it now stands, or nothing if it is no longer due. Ledger 437 says what a task or reminder editor takes.

**The default lead reaches the look** through `WxUIState::default_event_alert_lead`, set at startup from `default_reminder_minutes` and on a settings save through a new `UIUpdate::DefaultEventAlertLeadChanged`, the way `WorkingDayChanged` reaches the working day.

**One look, timed.** With the log at debug: "One look at what is due read 3 candidates in 1 ms", then 0 ms a minute later. This profile has four reminders, no accounts and an almost empty calendar, so that is the figure with its conditions and not a figure at scale; the calendar's own read cost 68 ms bounded on a six year calendar and the feed reads three days through the same seeking queries. The reads stay on the poll, and ledger 438 says no real-sized calendar has been under them and where the fallback is.

**The seam.** `.planning/REQUIREMENTS.md`'s `## v2 Requirements` table gains the row, with Pratik's words and no requirement id, pointing at the README's "Version 2's second seam" and at `Kind`'s doc comment; 06-07 edited that file in place, so the row went there rather than to the README.

**Version 0.124.0**, minor, with the changelog entry under Changed saying what changes, what hour a task is due at and where to change it, which events are silent and why, that a snooze on a task or an event is held here and sent nowhere, that a second thing arrives in a window of its own, that an all-day alert counts from the hour, and that nobody has heard any of it.

## Names 06-05 recorded that this plan changed

| 06-05's name | Now | Why |
|---|---|---|
| `wx_reminder_alert::say(item, ..)` | `say(rows: &[Due], ..)` | one sentence for every row found at a look |
| `wx_reminder_alert::Spoken::{NotYet, Already}` | gone | `raise` no longer says anything; the caller says the unsaid rows before it |
| `raise(parent, item, now, dates, a11y, default_snooze, spoken) -> Answer` | `raise(parent, rows, now, dates, a11y, default_snooze, editors) -> Vec<(Identity, Answer)>` | one answer per identity |
| `build_reminder_alert_dialog(parent, said, default_snooze, palette) -> (Dialog, Choice)` | `build_reminder_alert_dialog(parent, rows, now, dates, default_snooze, palette, editors) -> DueWindow` | the rows, and the bookkeeping to read back |
| `BetweenLooks::said_and_waiting` per item | the same map, but the window is held only while every row is within its hold | one window, not one per row |

Everything else 06-05 recorded, `whether_a_window_may_open`, `whether_somebody_is_typing`, `RepeatingTone`, `BETWEEN_TONES`, `MOST_TONES`, `HOW_OFTEN_TO_ASK_THE_TONE`, `LOOKS_TO_HOLD_A_REMINDER_WINDOW_WHILE_SOMEBODY_TYPES`, is read by the name it shipped under and is unchanged. Ledger 383's two-minute first gap after a hold is unchanged too.

## Red and green, honestly

**Task 2.** Red `7225f76e`: seven trailers, six tests and the count check, the bodies `todo!()` so the crate built and the schema in so the red was the bodies and not a missing table. Green `2bb52688`: 27 tests across the two modules.

**Task 3.** Red `74ddcc2c`: thirteen trailers, ten in `wx_reminder_alert.rs` including 06-05's `say` test which now goes through `sentence_for`, the live test, `theme_reach`'s live test which builds this dialog, and the count check for the one record on the module. Green `9cd1259c`: 15 in the module, the live test on its first run, `theme_reach` 7. The gate refused the first green commit on `tests/wired.rs`'s reading of `with_id`, above, and the buttons were built one by one.

**Task 4.** Red `eba042b5`: ten trailers, four in `event_alerts.rs`, one in `ui_types.rs`, two flipped and renamed in `managers.rs`, one rewritten table and one grown assertion in `calendar.rs`, and the count check for the five records on `ui_types.rs`. Green `731ed211`: 414 across the four modules. The wiring, `952a0f6d`, is glue over tested parts, as the plan says and `CLAUDE.md` allows, with one exception said here: the gate refused it once because two source-reading tests in `managers.rs` and two guard records located the moved arm by its text; both readings now read `change_an_opened_event`'s body, where the refusal is a `return` and not a `continue`, one marker line was kept contiguous so a reading that keys on it still finds it, and both records were re-pointed and re-measured. Observation 572 in the skill log.

**Red fixtures re-read once the code existed.** The task 2 prune test's boundary was written as "held until exactly the moment is let go" and re-read as kept, because `what_is_due` already treats that boundary as held no longer and the table should not have to know the rule; the test's comment says so. The task 3 live test asserted the dialog's title through a `get_title` the toolkit does not have on a `Dialog`; the title is pinned by the constant in the unit tests instead. The task 4 Microsoft table lost its `(true, 0)` case from the off list and gained it as a separate assertion, because on at nought is not off.

## Guard records: sixteen written, twelve re-measured, every one measured

Records 767 to 783 by a TOML reader; census `guards/guards.toml:79-80` 192 + 591 = 783. Every break applied by hand through `scripts/guards.sh --remeasure`, the whole library run at eight threads, and the run's own report quoted:

| Task | Record | Break | Predicted | Found |
|---|---|---|---|---|
| 2 | a hold held again replaces its moment rather than keeping the first | `DO UPDATE` to `DO NOTHING` | 1 | 1 |
| 2 | a hold that ended is let go and one that has not is kept | `<` to `<=` | 1 | 1 |
| 2 | a hold under a kind word this build does not know survives a read | the read narrowed to three words | 1 | 1 |
| 2 | the same id under two kinds is two holds in the table | `PRIMARY KEY (id)` | 5 | 5 |
| 2 | completing a task already done changes nothing | `AND NOT is_completed` dropped | 1 | 1 |
| 3 | a due row's text is its sentence and not its title | the title appended, on the live target | 1 | 1 |
| 3 | mark done is unavailable on an event row and its label says so | `if true` | 1 | 1 |
| 3 | snooze all answers for every row still listed and not only one | drain the first row alone | 1 | 1 |
| 3 | closing the due window dismisses whatever is still listed | `Dismissed` to `Edited` | 1 | 1 |
| 3 | done on an event row is refused by the bookkeeping itself | the refusal dropped | 1 | 1 |
| 4 | a day of a series is its own due thing and not the series | the id alone | 1 | 1 |
| 4 | an event stored as off is never given the default lead | `Off => Some(default)` | 1 | 1 |
| 4 | an event nobody said anything about is given the default lead | `Unknown => None` | 1 | 1 |
| 4 | microsofts pull stores off when the reminder is off | `false => None` | 2 | 2 |
| 4 | googles pull stores off only when google said the event never alerts | `return None` | 2 | 2 |
| 4 | the editor stores off when the last alert is taken away | the old `then` | 1 | 1 |

No break reddened less than predicted, so no second candidate was needed; none reddened nothing. The schema break reddens all five hold tests because the conflict clause names a key that no longer exists, which is a finding about how the table is keyed and not about the test. The live target's record couples `the_due_window_holds_every_kind` to `wx_reminder_alert.rs`: `scripts/check.sh --suites-for guards/guards.toml src/presentation/wx_reminder_alert.rs` answers `the_due_window_holds_every_kind`.

**Re-measured, by the command the count check printed, detached:** the four on `tasks.rs` at 22, all still exactly what they name; the one 06-05 record on `wx_reminder_alert.rs` at 15, its two tests; the five on `ui_types.rs` at 79, "the shape the person chose decides the picture" still at 49; and the two `managers.rs` records that moved with the arm, at their new locators. Twelve runs, none corrected.

**The count check counts by its own rule.** Three of the task 4 records were first written with counts taken by `grep -c '#[test]'`, 124 for `calendar.rs` and 140 for `managers.rs`; the check holds 194 and 137. The numbers written are the ones the check prints. Observation 573.

**No record for the last-row close.** The window closing when its last row is answered is in a handler no test can press; a break there reddens nothing, so nothing was written for it and ledger 439 says so.

## What the gate selected for each file touched

From the hook's own output on each commit. `held_alerts.rs`, `tasks.rs`, `mod.rs`, `wx_reminder_alert.rs`, `scan_fixtures.rs`, `scan_target.rs`, `ui_types.rs`, `managers.rs`, `calendar.rs`, `event_alerts.rs` and `wx_app.rs` each mapped to their `--lib module::` run; `mod.rs` also pulled `a_half_finished_task_move`, `ui_types.rs` and `managers.rs` the four move suites, and `wx_app.rs` the same seven suites 06-05 listed. `tests/the_due_window_holds_every_kind.rs` and `tests/theme_reach.rs` mapped to their own targets. `guards/guards.toml`, `docs/*.md`, `locales/`, `Cargo.toml`, `.planning/*` mapped to no target, as the brief said; the count check, the one-place check and the em-dash guard live in `house_style`, a whole-tree guard, and ran on every commit anyway. `Cargo.lock` with only the package's own version did not answer `all`; `Cargo.lock` with four dependency entries did, and that run is the whole gate on the branch: 54 targets, 7,677 passed, none failed, one ignored, the release build and the advisory check, five of CI's seven jobs. Ledger 374's `keyring` race did not fire. The merge ran the whole gate again and passed.

## Deviations from Plan

### 1. [Pratik's answer, new] A Details button

Not in the plan; in decision 1's answer. Built as a sixth button, an `Editors` trait, and `Answer::Edited`. Opens an event; disabled with its reason for a task and a reminder, because no editor for an existing one exists anywhere in the program, found by searching for `Prefill`, `PimCommand`, and every `ask_for` caller. Ledger 437.

### 2. [Plan option changed by the answer] Decision 2 is option 3 with a guard, and off became representable

The plan recommended option 1. Pratik chose option C with "make sure that the event actually has a reminder", read as three states. The minimum additive change was one new module and one call in each of the three writers that know the alert is off; the count of writers and readers is above, and it did not grow into three sync paths. What stays unread, CalDAV alarms and Google's calendar default, is ledger 434; what is not sent, off to Google, is ledger 435.

### 3. [Rule 3 - Blocking] `rustls` 0.23.45, `e527b6ca`

The whole gate refused the branch at the advisory check on RUSTSEC-2026-0285, published 2026-09-14, against `rustls` 0.23.42, fixed in 0.23.45. The patched release moves `rustls-webpki` and `aws-lc-sys` with it: four lock entries, no new crate, no manifest change. Followed rather than accepted in `.cargo/audit.toml`, which would be a decision; said plainly here because it is not this plan's change.

### 4. [Rule 1 - The reading moved] Two `managers.rs` readings and two records re-pointed

Above under red and green. `test_the_one_day_answer_asks_whether_the_day_can_be_kept_before_it_writes` and `test_updating_an_occurrence_exception_in_the_calendar_window_uses_the_dedicated_merge` read `change_an_opened_event`'s body now, with comments saying why; the refusal assertion asks for `return Err(NotChanged::Refused(` rather than `continue;`.

### 5. [Rule 3 - The reading of `with_id`] Six buttons built one by one

`tests/wired.rs` reads each `with_id(ID_X)`; a loop over an array hid them. The buttons are built one by one with a comment.

### 6. [Plan detail not built] The count sentence and the list's name are plurals in code

"1 thing due", "3 things due", "And 2 more" are English plurals in `wx_reminder_alert.rs`, the shape 06-03 retired for dates. The catalogue holds one area and its loader is written for one file; a second area touches `catalogue.rs`'s six records and is the catalogue's next step. Ledger 440.

### 7. [Plan estimate corrected] The remeasures, and what a new test costs

The plan priced "whatever 06-05 left on `wx_reminder_alert.rs`" at one record and it was one; task 2's four were four; task 4's five on `ui_types.rs` were the price of ledger 404's record having a test to redden. What the plan did not price is that `calendar.rs` and `managers.rs` carry 72 and 50 records, so a single new test in either is hours; the behaviour was tested in a new module and the writers left as glue with existing assertions edited, and the one branch that has no pin, nothing stored with nought in the box, is ledger 441. Observation 574.

### 8. [Wording only a listening pass settles] "Event now" for an all-day event at the hour

Under decision 1 an all-day event raised at a quarter to nine says "Event now: Conference" at nine, because its whole-day start is not ahead of now and not late. Not widened; in ledger 431 with the rest of the unheard wording.

### 9. [The reading's shape] One window is held only while every row is within its hold

06-05 held per item; with one window, a row arriving while another waits is said and joins the window when it opens. The doc comment on `raise_what_is_due` says so. `LOOKS_TO_HOLD_A_REMINDER_WINDOW_WHILE_SOMEBODY_TYPES` is unchanged at one.

### The exception set for scripted edits is zero

Every tracked file was edited with Read then Edit or Write, including `Cargo.lock`'s package entry. `cargo fmt` reformatted files this plan had written, which is the project's formatter; `cargo update -p rustls --precise 0.23.45` wrote the four dependency entries, which is cargo's own writer for that file; `gsd-tools windows append` wrote the ledger, the sanctioned writer; `scripts/guards.sh` edited and restored the files its breaks name, byte for byte, and `git status` was clean after each run. Carriage returns measured with `tr -cd '\r' | wc -c` on every touched file: zero on every one. Em dashes in every document: zero by a byte search.

## What the tests prove and what only a person can settle

**Structure, proved:** everything in the coverage block above; the hold across a restart is a table read by a fresh `MessageCache` in the same test process, which is the same read a restart does; the identity of a series day; off never given the default; the three writers writing off; the window's rows, labels, keys and bookkeeping; the readings of the look.

**Experience, held by nobody:** whether the three kinds' sentences read well by ear in one list, whether the count sentence reads as a list or a wall, whether "Event now" for an all-day event is help, whether the disabled labels are heard as somebody tabs past, whether the tone over a list reads as before, whether NVDA and Narrator name the list and the picker, and whether "Task due today" at nine reads as help or nagging. Ledger 431 to 433 and 436. Not surfaced as something to do now, on the standing note that manual testing waits for phase 8.

## Ledger

`.planning/WINDOWS.md` 431 to 441 through `gsd-tools windows append`, 430 before and 441 after, both halves counted at 441, no backslash in any entry:

| id | kind | what |
|---|---|---|
| 431 | unrun-verify | the Due now window, three kinds in one list, the count sentence, the tone over a list, unheard |
| 432 | unrun-verify | the disabled Mark Done and Details labels, and whether a tab past them says why |
| 433 | unrun-verify | the list and the picker named on MSAA only; the scan target now opens on three rows |
| 434 | todo | the events decision 2 leaves to the default: Google's calendar default and every CalDAV alarm, unread; option 4 is the way through |
| 435 | todo | off written here is not sent to Google as off |
| 436 | todo | the day reminder still at midnight while a dated task and an all-day event are at the working-day hour |
| 437 | todo | Details opens only an event; what a task and a reminder editor take |
| 438 | unrun-verify | one look timed on an almost empty profile only, 1 ms |
| 439 | todo | the last-row close is in a handler no test can press |
| 440 | todo | the count sentence and the list's name are English plurals in code; the catalogue's second area |
| 441 | todo | Microsoft's on-at-nought is stored as nothing; the nothing-stored-with-nought branch of the editor has no pin |

The ids in the sections above were first written from memory and five were off by one; every one was re-taken from the file after the append and corrected.

## Known Stubs

None. No `todo!()` remains in any file this plan touched. Every kind is fed: `grep -n 'due::Kind::' src/presentation/wx_app.rs` shows the three arms in both writer matches, and the timing run's log shows the look reading candidates once a minute in the running program. The Details button opens an event through a non-test path and is disabled with its reason where nothing can open; that is a gate, said in the label and in the changelog, not a stub.

## Threat Flags

None new. T-06-35 mitigated by `complete_task`'s `AND NOT is_completed` and its twice-called test. T-06-36 mitigated: the writer's `Snoozed` match holds a task and an event through `hold_alert` and no arm writes either row's time. T-06-37 mitigated: `due_identity` composes id and start, the series test holds, the record is measured. T-06-38: decision 2 alerts where an alert is stored, stays silent where off is stored, and fills silence with the default by Pratik's choice; the two cases where silence is not the owner's are ledger 434. T-06-39: one look timed, on this profile only, ledger 438. T-06-40: `MOST_ROWS_SAID` is three. T-06-41 accepted as planned: an unknown word is kept in the table and left out of the map. T-06-SC: no package added; four lock entries moved for an advisory.

## Verification

- `cargo test --lib -- application::due:: common::catalogue:: presentation::date_display:: data::message_cache::held_alerts:: data::message_cache::tasks:: presentation::wx_reminder_alert::` at green: pass. One `--lib`, several filters after `--`.
- `cargo test --test wired`, `cargo test --test the_due_window_holds_every_kind`, `cargo test --test theme_reach`: 71, 1 and 7 passed.
- `scripts/check.sh` through the hook on every commit, three red commits held to exactly their named failures; refused twice on green commits, once on `with_id` and once on the moved arm, both said above.
- The whole gate on the branch, output to a file and its own exit status read: 0, 54 targets, 7,677 passed, 0 failed, 1 ignored, release build, `cargo audit` clean after `e527b6ca`. Once more on the merge, green.
- `#[test]` counts by the check's rule: `due.rs` 46 unchanged; `wx_app.rs` unchanged, no test added and its 48 records not re-measured; `mod.rs` 23 unchanged; `tasks.rs` 21 to 22; `held_alerts.rs` 0 to 5; `wx_reminder_alert.rs` 6 to 15; `ui_types.rs` 78 to 79; `event_alerts.rs` 0 to 4; `managers.rs` 137 and `calendar.rs` 194 unchanged.
- Sixteen records measured, twelve re-measured, 783 by a TOML reader, census 192 + 591.
- `roadmap update-plan-progress` not run; `ROADMAP.md` and `STATE.md` edited by hand and the diff read.
- Every document this plan wrote carries no em dash and no carriage return; `tests/house_style.rs` ran on every commit.

## Self-Check: PASSED

Files present: `src/data/message_cache/held_alerts.rs`, `src/application/event_alerts.rs`, `tests/the_due_window_holds_every_kind.rs`, `src/presentation/wx_reminder_alert.rs`, `docs/KEYBOARD_SHORTCUTS.md`, `docs/changelog.md`. Commits `7225f76e`, `2bb52688`, `141f8374`, `74ddcc2c`, `9cd1259c`, `bf10f978`, `eba042b5`, `731ed211`, `952a0f6d`, `41f96882`, `e527b6ca` and the merge `abaa0667` in `git log`. Line numbers in this file taken by `grep -n` at `abaa0667`.

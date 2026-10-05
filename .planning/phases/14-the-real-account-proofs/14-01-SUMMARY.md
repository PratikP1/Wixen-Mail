---
phase: 14-the-real-account-proofs
plan: 01
subsystem: pim-sync
status: complete
tags: [google, calendar, contacts, tasks, sync, feedback, log, "#22"]
requires: []
provides:
  - "application::who_holds_the_calendars: may_google_be_asked, a_google_token, WhyNothingWasAsked and its sentences and words, WhatWasAsked"
  - "CalendarSyncResult::absorb, and UIUpdate::CalendarSyncComplete carrying the result whole"
  - "SyncResult and TaskSyncResult carrying what_was_asked, folded by absorb and said by each summary"
  - "UIUpdate::ModuleSyncNeedsAttention for a tasks sync that asked Google nothing"
  - "SummingUp::opening_sentence"
affects: [14-02, 14-03, src/presentation/wx_app.rs spawn_calendar_sync spawn_contacts_sync spawn_tasks_sync]
tech-stack:
  added: []
  patterns: ["one answer every sync asks before a Google sign-in", "a reason carried on the result and folded by absorb"]
key-files:
  created:
    - src/application/who_holds_the_calendars.rs
  modified:
    - src/application/mod.rs
    - src/application/calendar.rs
    - src/application/contacts_sync.rs
    - src/application/tasks_sync.rs
    - src/application/summing_up.rs
    - src/application/caldav_sync.rs
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - tests/the_log_carries_what_a_report_needs.rs
    - tests/one_check_says_who_runs_the_mail.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/changelog.md
decisions:
  - "The module keeps the planned name who_holds_the_calendars: 14-02 and 14-03 name it 28 times"
  - "A browser sign-in refused with an authentication error is the run-out reason; a network failure stays the sync's error to count"
  - "No account open is a reason of its own, as spawn_notes_sync says it; the other passes still run"
  - "The log line lives once in a_google_token, which every sync asks, rather than in each spawn"
  - "The reason alone when no pass ran and nothing went wrong; after the counts otherwise (D-08)"
metrics:
  duration: "about 2 hours 30 minutes, 2026-10-05"
  completed: 2026-10-05
actuals:
  # chars/4 over added lines, git diff main..e029a185 plus the documents commit
  tokens: 17000
  tasks: 3
  commits: 5
---

# Phase 14 Plan 01: The syncs say why nothing came from Google Summary

Sync Calendar, Sync Contacts and Sync Tasks ask one answer before any Google
sign-in and, when Google cannot be asked, say why in one sentence with the cue
for an account needing attention and write the reason to the log; an account
not at Google never asks Google, and a held calendar item is counted once.

## What works now

- `who_holds_the_calendars::may_google_be_asked` answers from the account and
  the key this copy holds: not Google's to ask unless `WhoRunsTheMail::of`
  says Gmail; then no key, then an app password, in D-06's order; otherwise
  ask. `a_google_token` adds the token request, maps an authentication refusal
  to "browser sign-in missing or run out", and writes
  `Google was not asked for <module>: <reason>` at info, words only.
- Each spawn finds the account in `state.accounts` and asks `a_google_token`
  with its module; none of them spells `"gmail"` any more. A missing account
  is the "no account is open" reason.
- Each summary says the reason's sentence alone when nobody was asked and
  nothing went wrong, and after the counts when a calendar server, feed,
  address book or Microsoft was asked, or a pass failed.
- The calendar and contacts finishes, and the new `ModuleSyncNeedsAttention`
  for tasks, signal `AccountNeedsAttention` with the whole sentence when a
  reason is carried, at any level of what is said while fetching; otherwise
  `SyncComplete` as before.
- Every calendar pass folds through `CalendarSyncResult::absorb`, and the
  window receives the result boxed whole: `grep -c` of the doubled held line
  answered 5 before and 0 after.
- Reachable: Tools, Sync Contacts, Sync Calendar and Sync Tasks
  (`ID_SYNC_CONTACTS`, `ID_SYNC_CALENDAR`, `ID_SYNC_TASKS`, wx_app.rs 5885 to
  5896) and the sidebar's Sync Now (4867 to 4878) reach the three spawns, and
  `grep who_holds_the_calendars src/presentation/wx_app.rs` names a call in
  each (30879, 31014, 31224).
- Counts on 2026-10-05: `application::who_holds_the_calendars::` 18 passed,
  `application::calendar::` 196, `application::contacts_sync::` 281,
  `application::tasks_sync::` 119, `presentation::wx_app::` 199,
  `the_log_carries_what_a_report_needs` 15, `one_check_says_who_runs_the_mail`
  2, `wired` 77, `house_style` 74.

Nothing here has met a real Google account, and no NVDA case presses a Sync
command, so the pull request's NVDA run does not hear these sentences
(ledger 809).

## Commits

| Commit | What | Hook |
|---|---|---|
| `3303f8e4` | test: failing cases for a calendar sync that asks Google nothing and says nothing | red, 329 s |
| `73ecc2a5` | fix: Sync Calendar says why nothing was asked of Google; five records, two re-measured | affected, 432 s |
| `cd643634` | test: failing cases for contacts and tasks saying why nothing was asked of Google | red, 167 s |
| `e029a185` | fix: Sync Contacts and Sync Tasks say why; four records, four re-measured | affected, 430 s |
| docs | the changelog, ledgers 807 to 809, REAL-01's line and row, the four marks, this summary | see the report |

Pull request 164, pushed under Pratik's standing OK of 2026-09-23.

## Guard records

Nine new, each measured with `scripts/guards.sh` against the whole library;
the arrived-since count 640 to 649, and 1,446 records by the TOML reader.

| Record | Break | Red |
|---|---|---|
| a gmail account on an app password is not asked of google | the app-password arm removed | the Gmail rows' case |
| the calendar summary says why google was not asked after the counts | the after-the-counts sentence removed | the calendar's after case |
| the calendar sync builds no google sign-in of its own (suite) | `credentials_for("gmail")` beside the question | the reading and its companion |
| a calendar sync that asked google nothing is signalled as needing attention | the arm signals `SyncComplete` | the arm reading |
| a calendar item held for a choice is folded once | the held count added twice in `absorb` | the held case and the whole fold |
| the contacts summary says why google was not asked after the counts | the sentence removed | the contacts after case |
| the tasks summary says why google was not asked after the counts | the sentence removed | the tasks after case |
| folding contacts results keeps the reason and notes somebody answered | the fold line removed | three cases, one in contacts_sync |
| folding tasks results keeps the reason and notes somebody answered | the fold line removed | three cases, one in tasks_sync |

Re-measured, one call per commit: the readings file's two records after task
1, both agreeing; the four task 1 records naming the new module after task 2,
all agreeing. "The calendar sync builds no google sign-in of its own" came back
one short at first, the companion going red too, and was corrected by hand and
measured again. Anchors: `test_every_guard_record_still_names_one_place_in_the_tree`
green after each commit; the calendar record on the "Calendar sync" opening
still names one place because the format lines kept their indentation.

## Ledger

- 807 opened and fixed (`deviation`): the three syncs skipped Google without a
  word and said 0 created (#22).
- 808 opened and fixed (`deviation`): the held count added twice.
- 809 opened (`unrun-verify`): the three sentences heard on his Gmail account
  in sitting 1.
- Header: 693 open, 116 fixed, 809 in all.

## Deviations from Plan

1. **[Cost] The calendar fold cases live in the new module, not beside
   `absorb` in calendar.rs.** A case there would have flagged its 80 records,
   each a whole-library run, about three hours; task 2's own reasoning for
   contacts and tasks applies the same way.
2. **[Rule 3] Three readings of the old hand folds updated in task 1's
   green:** `calendar.rs`'s
   `test_a_change_that_can_never_be_saved_reaches_the_window_that_speaks_it`,
   `caldav_sync.rs`'s
   `test_a_change_sent_to_a_calendar_server_is_counted_in_what_the_person_is_told`
   (now read with spaces removed) and `tests/wired.rs`'s
   `test_the_calendar_sync_says_what_nothing_can_send`. Premise 7 read the
   guard register and not the tests that read the region as text.
3. **[Shape] One reading over the three spawns** in
   `tests/the_log_carries_what_a_report_needs.rs`, with one companion, rather
   than one per spawn; task 2 widened its list. The log line itself is read
   in `a_google_token`, where it is written once for all three.
4. **[Shape] `SummingUp::opening_sentence`** added so a summary that opens
   with the reason ends with a full stop; no test was added to
   `summing_up.rs`, its behaviour held by the new module's cases.
5. **[Red] Two red-test corrections before the green:** the calendar
   after-the-counts case asserted the whole summary passes
   `reads_as_a_persons_sentence`, which the existing "Calendar sync:" opening
   never did (the noun "sync"), so that assertion went; and the reading's
   needle lost its closing bracket, because the formatter puts a comma after
   the last argument when it wraps the call.
6. **[Scope] A tasks finish of its own,** `UIUpdate::ModuleSyncNeedsAttention`,
   because the tasks sentence travels as a string shared with notes.
7. **[Premise] Line numbers held** as premise 1 to 5 gave them at `60c5bd5b`;
   the record count was 1,437 at the start, as the README says.

## TDD Gate Compliance

Task 1 (tracer): `3303f8e4` red, accepted by `red-commit.sh`, then `73ecc2a5`.
Task 2: `cd643634` red, accepted, then `e029a185`. Task 3 is documents.

## Threat Flags

None beyond the register. T-14-01: the log line carries `module.word()` and
`why.word()` only, and `GooglesAnswer` derives no `Debug` because it carries a
token. T-14-02 and T-14-04: one sentence per module per sync, at its finish.
T-14-03: `may_google_be_asked` asks `WhoRunsTheMail::of`, held by
`one_check_says_who_runs_the_mail`. T-14-SC: nothing installed.

## Self-Check: PASSED

`src/application/who_holds_the_calendars.rs` exists; `3303f8e4`, `73ecc2a5`,
`cd643634` and `e029a185` are on the branch.

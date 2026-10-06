---
phase: 14-the-real-account-proofs
plan: 02
subsystem: pim-sync
status: complete
tags: [google, microsoft, calendar, contacts, tasks, sync, log, f5, account-manager, "#22"]
requires: [14-01]
provides:
  - "service::asked_and_answered: the_line for one request and the_reason_word out of a refusal's body"
  - "every GoogleApiClient and TasksClient helper writing the line at info, one per request sent"
  - "application::who_holds_the_calendars: may_microsoft_be_asked, a_microsoft_key, at_neither_with_nothing_of_its_own, the_finish_line, what_adding_an_account_starts, what_adding_this_account_starts, added_in_this_visit, what_refresh_does, WhyNothingWasAsked::sentence_for_every_module"
  - "WhatWasAsked::why_not_asked, renamed from why_not_google"
  - "spawn_calendar_sync, spawn_contacts_sync and spawn_tasks_sync taking the account they sync; sync_the_module; bring_what_a_new_account_holds"
affects: [14-03, 14-04, 14-09, src/presentation/wx_app.rs handle_account_mgr]
tech-stack:
  added: []
  patterns: ["a log line built from parts, never from an error's text", "one function choosing a module's sync, reached by Sync Now and F5"]
key-files:
  created:
    - src/service/asked_and_answered.rs
  modified:
    - src/service/mod.rs
    - src/service/google_api.rs
    - src/service/tasks_api.rs
    - src/application/who_holds_the_calendars.rs
    - src/application/contacts_sync.rs
    - src/application/tasks_sync.rs
    - src/presentation/wx_app.rs
    - src/presentation/managers.rs
    - tests/the_log_carries_what_a_report_needs.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
decisions:
  - "The line's method is a word, not reqwest's type, so asked_and_answered names no networking crate and stays off the outward census"
  - "A path segment is masked only when something stands before its @; Google's @me names nobody and is kept"
  - "The reason word is the first errors[].reason or error.status that is letters and underscores, 1 to 64 long"
  - "why_not_google renamed why_not_asked, since it now carries Microsoft's and neither's reasons"
  - "A refused Microsoft sign-in stays an error; only a missing Microsoft key became a reason (Microsoft is a later phase)"
  - "The stand-in cases live in asked_and_answered.rs, not beside the clients, whose files seventeen records fingerprint"
metrics:
  duration: "about 6 hours, 2026-10-05"
  completed: 2026-10-05
actuals:
  # chars/4 over added lines, git diff main..94fe66ab plus the documents commit
  tokens: 42000
  tasks: 3
  commits: 7
---

# Phase 14 Plan 02: Each request and each sync written Summary

Every request the Google and tasks clients send, and every finished
calendar, contacts and tasks sync, now writes one Info line built from
parts; each sync asks only the account's own provider; adding an account
runs the three syncs once or says the reason once; and F5 in Contacts,
Calendar, Tasks or Notes runs that module's sync.

## What works now

- `service::asked_and_answered::the_line` writes, for example,
  `Asked Google: GET www.googleapis.com/calendar/v3/calendars/primary/events, answered 200`
  or `..., answered 403 accessNotConfigured` or `..., no answer came`. The
  query is cut, a segment holding an address (or `%40`) is masked with
  `mask_email`, and a reason word is kept only when it is letters and
  underscores.
- `GoogleApiClient::api_get`, `api_post`, `api_patch` and `api_delete` go
  through `the_answer_written_down`, one line per request, so a read retried
  three times writes four lines (D-09). `TasksClient::get`, `send` and
  `delete` do the same under Google Tasks or Microsoft To Do by the path. The
  network error's text no longer carries the address: with the sync marker,
  it used to reach the log through `with_retry`'s warning.
- Each spawn writes `the_finish_line` at info just before its finish, for
  example `calendar sync finished, account at google, not asked: app_password, created 0, updated 0, deleted 0, sent 0, errors 0`.
- Microsoft is asked only for an account at Microsoft, through
  `who_holds_the_calendars::a_microsoft_key`; with no key, the reason is said
  as Google's is. An account at neither provider with no calendar server,
  feed or address book hears "This account's mail is at neither Google nor
  Microsoft, so there are no calendars there to bring."
- `handle_account_mgr` takes the window's sender; each account in
  `added_in_this_visit` goes to `bring_what_a_new_account_holds`, which runs
  the three syncs for that account, says
  `sentence_for_every_module()` once with the attention cue, or does nothing
  for an account at neither.
- The first `ID_REFRESH_FOLDER` arm asks `what_refresh_does` and, in every
  module but Mail, calls `sync_the_module`, which the sidebar's Sync Now calls
  too.

**What F5 did before, read at the merge of 14-01 (`861db2ac`):** two arms
answered `ID_REFRESH_FOLDER`, and neither asked which module was showing. With
a saved search chosen in the tree, in any module, it ran the search again.
Otherwise it read `folder_on_screen`, the mail folder the tree still had
selected, and said "Refreshed" with that folder's name, or signalled
"no folder selected". So with Contacts, Calendar, Reminders, Tasks or Notes
showing, F5 refreshed a mail folder nobody was looking at.

Reachability, read by grep after the green: Tools, Account Manager
(`ID_ACCOUNT_MGR`) and File, New, Account (`ID_NEW_ACCOUNT`) call
`handle_account_mgr(&frame, &state, &message_cache, &a11y, &runtime, &ui_tx)`,
which calls `bring_what_a_new_account_holds`, which calls the three spawns;
F5 reaches `sync_the_module(app, module)` from the first `ID_REFRESH_FOLDER`
arm; and every Google and tasks helper calls its client's
`the_answer_written_down`, which calls `the_line` twice, once for no answer
and once for an answer.

Counts on 2026-10-05, after the green: `service::asked_and_answered::` 15
passed, `service::google_api::` 39, `service::tasks_api::` 62,
`application::who_holds_the_calendars::` 30, `application::notes_backend::`
25, `presentation::wx_app::` 199, `presentation::managers::` 137,
`the_log_carries_what_a_report_needs` 19, `wired` 77,
`undoing_a_mark_names_the_message` 21,
`microsoft_people_join_the_people_found_list` 6,
`progress_is_shown_and_results_are_said` 15. Clippy with `-D warnings` clean.

The two tracer cases, as they went red and then green:

```
red  (0acc075c): one request, one line: []   left: 0  right: 1
green (97c4e1d2): test_a_google_calendar_read_writes_one_line_and_nothing_private ... ok
                  test_a_refused_google_read_writes_its_status_and_reason_and_nothing_else_of_the_body ... ok
```

Nothing here has met a real Google or Microsoft account; it is proven against
loopback stand-ins. No NVDA case adds an account or presses F5 in a module
(ledger 814).

## Commits

| Commit | What | Hook |
|---|---|---|
| `0acc075c` | test: failing cases for the line one Google request writes | red, 214 s (two refusals first: formatting, 20 s; the outward census, 264 s) |
| `97c4e1d2` | feat: a Google read writes one line; three records | affected, 337 s |
| `97314565` | test: every request, every finish and the account's own provider; field renamed | red, 367 s |
| `1c588f91` | feat: every request and finish written, own provider only; six records, fourteen re-measured | affected, 444 s |
| `1f6daa44` | test: an added account's first sync and F5 in a module | red, 220 s |
| `94fe66ab` | feat: the first sync and F5; four records, two re-anchored, ten re-measured | affected, 458 s |
| docs | pages, changelog, ledgers 810 to 816, REAL-01's line and row, the four marks, this summary | see the report |

Pull request 165, pushed under Pratik's standing OK of 2026-09-23.

## Guard records

Thirteen new, each measured with `scripts/guards.sh` against the whole
library; the arrived-since count 649 to 662.

| Record | Break | Red |
|---|---|---|
| a request's line cuts the query off the address | the path read with its query | 5 cases |
| a refusal's reason word is letters and underscores only | any printable accepted | the refusal case |
| an answered google request writes its line at info | written at debug | 5 Google stand-in cases |
| a change sent to google writes its line | the create reads its answer by hand | the POST case |
| a google request nobody answered keeps its address out of the error | the error keeps `e` | the no-answer case |
| the tasks client writes each request's line at info | written at debug | 4 tasks cases |
| a gmail account never asks microsoft | any account not at neither may ask | the Microsoft rows |
| an account at neither provider says why there is nothing to bring | the reason dropped | the neither rows |
| a finished calendar sync writes its line at info | written at debug | the finish reading and its companion |
| an account added in the account manager has its first sync started | the start removed | the reading and its companion |
| a reason known on adding an account is said once, not per module | the app password starts the syncs | the adding rows |
| f5 in a module other than mail syncs that module | Notes reads the folder | the F5 rows |
| the f5 arm syncs only the modules the answer says sync | the comparison inverted | the F5 reading |

Re-anchored and measured again: "asking for a notes sync reaches the sync
rather than reporting one" (the Notes arm moved into `sync_the_module`) and
"the sync button in the calendar window really starts a sync" (the call
gained the account). Re-measured, one call per commit: fourteen after task 2,
ten after task 3. Three came back short, each because a new case reaches the
line its break edits: the query cut (two tasks and no-answer cases), the
read's line at info (the three write cases), and the app password (the adding
rows); each red list was corrected by hand and measured again.

## Ledger

- 810 to 813 opened and fixed (`deviation`): no line per request or sync, the
  provider chosen by sign-in keys, nothing run on adding an account, F5 in a
  module reading the mail folder.
- 814 opened (`unrun-verify`): adding an account and F5 in each module heard
  in sitting 1.
- 815 opened (`todo`, for Pratik): a just-added account whose browser sign-in
  has already run out hears the reason three times, once per sync.
- 816 opened (`todo`, Microsoft phase): a refused Microsoft sign-in is still
  an error, not a reason.
- Header: 696 open, 120 fixed, 816 in all.

## Deviations from Plan

1. **[Cost] Every stand-in case lives in `asked_and_answered.rs`**, not one
   per helper in `google_api.rs` and `tasks_api.rs`, whose 7 and 10 records
   would each have been flagged; the plan's verify commands for those two
   modules still pass at 39 and 62.
2. **[Rule 3] The line takes the method as a word.** The first red was
   refused by `service::outward::completeness::test_every_module_that_names_a_way_out_is_on_one_of_these_lists`
   because the module named `reqwest::Method`.
3. **[Rule 2] The network error's text drops the address** in both clients
   (`without_url`), because `with_retry` writes it at warn and the address
   carries the sync and page markers; the no-answer case proves it.
4. **[Shape] `@me` is not masked**: a segment is masked only when something
   stands before its `@`.
5. **[Scope] Microsoft "token refused" is not a reason**; the plan's row for
   it is left out (ledger 816), on answer 11.
6. **[Shape] `why_not_google` renamed `why_not_asked`** in task 2's red,
   including two test literals in `contacts_sync.rs` and `tasks_sync.rs`.
7. **[Shape] Task 3's readings live in `who_holds_the_calendars.rs`**, which
   already reads the window, rather than in the log's readings file.
8. **[Red] The F5 reading was strengthened in the green** to ask that the arm
   compares with `SyncsTheModule`, because an inverted comparison passed it.
9. **[Rule 3] The calendar spawn lost its `state` parameter**, so the calendar
   window's Sync button in `managers.rs` passes `the_active_account(state)`.
10. **[Premise] Line numbers** held as premises 1 to 8 gave them, read at
    `861db2ac`; records were 1,446 at the start.

## TDD Gate Compliance

Task 1 (tracer): `0acc075c` red, accepted by `red-commit.sh`, then
`97c4e1d2`. Task 2: `97314565` red, then `1c588f91`. Task 3: `1f6daa44` red,
then `94fe66ab`. Every red ran against stubs that compile and answer wrongly;
no row was green against its stub.

## Threat Flags

None beyond the register. T-14-05: the line is built from host, path and
status, with cases asserting token, marker, `?` and `syncToken` absent at
info and above. T-14-06: masking case with `%40`. T-14-07: the reason word's
shape case and its record. T-14-08: the Microsoft rows and the reading that
no spawn looks up a Microsoft key of its own. T-14-10: one sentence per
module or the reason once. T-14-SC: nothing installed.

## Self-Check: PASSED

`src/service/asked_and_answered.rs` exists; `0acc075c`, `97c4e1d2`,
`97314565`, `1c588f91`, `1f6daa44` and `94fe66ab` are on the branch.

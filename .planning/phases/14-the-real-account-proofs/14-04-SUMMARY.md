---
phase: 14-the-real-account-proofs
plan: 04
subsystem: pim-sync
status: complete
tags: [google, calendar, calendar-list, read-only, hidden, free-busy, "#22", answer-3]
requires: [14-01, 14-02, 14-03]
provides:
  - "service::google_api: GoogleCalendarListEntry, GoogleAccessRole and its may_write, GoogleApiClient::list_calendars (paged at 250, showHidden=true, the whole list or an error)"
  - "application::every_google_calendar: sync, which the calendar spawn calls; the filing rules; putting away a calendar Google stopped listing; only_free_and_busy and taken_off_this_computer"
  - "data::message_cache::calendars: a_calendar_google_listed, CalendarContainer::its_id_at_google, ListedCalendar, file_a_listed_calendar, set_calendar_sync_token; ensure_provider_calendar finds the plain row"
  - "data::message_cache::calendar: OneProvidersCalendar, event_in, delete_event_in"
  - "application::calendar: sync_google_calendar reads every Google calendar filed, each from its own marker; which_calendar_at_the_provider answers Google's id; CalendarSyncResult's calendars_read, calendars_showing_only_free_and_busy, calendars_put_away"
affects: [14-08, 14-09, 14-12]
tech-stack:
  added: []
  patterns: ["a new first request put in a wrapper the production caller uses, so the function forty stand-ins script keeps its request sequence", "a row's provider identity carried in its id with the account, since the table is keyed by the id alone"]
key-files:
  created:
    - src/application/every_google_calendar.rs
  modified:
    - src/service/google_api.rs
    - src/application/calendar.rs
    - src/application/mod.rs
    - src/application/who_holds_the_calendars.rs
    - src/data/message_cache/calendars.rs
    - src/data/message_cache/calendar.rs
    - src/data/message_cache/mod.rs
    - src/presentation/wx_app.rs
    - guards/guards.toml
    - docs/PROVIDER_SETUP.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
decisions:
  - "The calendar list is read by every_google_calendar::sync, which the window calls, and calendar::sync_google_calendar reads the calendars filed; reading the list inside the old function would have handed the first scripted reply of about forty existing stand-ins to the new request"
  - "A listed calendar's id is google:<account>:<Google's id>, not google:<Google's id>, because the same calendar sits on two accounts' lists and the calendars table is keyed by the id alone"
  - "Each listed calendar's marker is kept in the calendars table's own sync_token column, so it goes when the row does; the main calendar keeps its sync_state row"
  - "The main calendar always starts shown, whatever Google's view says of it, because events made here in no calendar go there"
  - "A change waiting in a calendar Google stopped listing is moved to no calendar, where changes_nothing_can_send says on every sync that it cannot be sent"
  - "A Google read owns the rows in its calendar and any of the account's in no Google calendar, so a row somebody moved by hand into a calendar of their own is still updated where it is"
metrics:
  duration: "about 4 hours, 2026-10-05 to 2026-10-06"
  completed: 2026-10-06
actuals:
  # chars/4 over added lines in src, docs and guards, git diff main on the branch with the documents
  tokens: 17300
  tasks: 3
  commits: 5
---

# Phase 14 Plan 04: Every Google calendar, not only the main one Summary

Sync Calendar reads a Gmail account's Google calendar list, hidden calendars
included, and brings every calendar on it under a row of its own, with
read-only, free-and-busy, hidden and dropped calendars each handled and events
kept apart per calendar. Proven against loopback stand-ins only.

## What works now

- **Every calendar comes.** Sync Calendar, F5 in Calendar and an added
  account's first sync all reach `spawn_calendar_sync`
  (`wx_app.rs:5883`, `:22259`, `:30871`, and `managers.rs:1590`), whose Google
  arm calls `every_google_calendar::sync`. That reads
  `GET /users/me/calendarList?maxResults=250&showHidden=true`, pages to the
  end, files each calendar, then `calendar::sync_google_calendar` reads the
  main calendar as `primary` and every listed one by its own id.
- **Names.** The main calendar keeps its plain row and is called Google
  Calendar unless the person named it at Google; every other calendar is
  named as the person named it, else Google's name.
- **Read-only.** A `reader` calendar is filed read-only; `writer`,
  `writerWithoutPrivateAccess` and `owner` writable; a role Google has not
  named is read-only. A change made in a read-only one is said by the
  existing `changes_nothing_can_send`.
- **Free and busy.** A `freeBusyReader` calendar is not filed or read, and
  the sync says once how many were passed over.
- **Hidden.** A calendar hidden or unticked at Google starts hidden; Enter in
  the sidebar shows it, and no sync turns that back.
- **Markers.** Each listed calendar reads from its own marker on its row; a
  marker refused reads that calendar whole.
- **Kept apart.** One meeting in two calendars is two rows, and a
  cancellation in one leaves the other.
- **Put away.** After a whole list, a calendar Google no longer lists goes
  with its events and the sync says so; a change waiting in it is kept in no
  calendar and said on every sync. A list cut short puts nothing away and
  says the list could not be read.
- **Changes go to their calendar.** A change made to an event in a second
  calendar is sent to `PATCH /calendars/<that calendar>/events/<id>`, read back
  from the stand-in.
- **Said.** More than one calendar read adds "N calendars read" to the
  summary's counts; one calendar is said exactly as before.

The four verify commands, run before the green commits: task 1,
`every_google_calendar` 1, `data::message_cache::calendars` 10,
`service::google_api` 39, `application::calendar` 196, all passing; task 2,
`every_google_calendar` 12, `data::message_cache::calendar` 56,
`application::calendar` 196, `service::google_api` 39, all passing, with
`who_holds_the_calendars` 41 and `answered_meetings` 37.

## Commits

| Commit | What | Hook |
|---|---|---|
| `0f81c60a` | test: failing cases for a second calendar and the main calendar's plain row | red, 215 s |
| `c7ecab6e` | feat: the list read, the filing, every filed calendar read, the spawn wired; three records | affected, 481 s |
| `62da5928` | test: failing cases for every kind of calendar on the list | red, 206 s |
| `03a9111a` | feat: read-only, free and busy, hidden, markers, lookups within a calendar, put away; eleven records new, one re-anchored, two re-measured | affected, 355 s |
| docs | pages, changelog, ledgers 821 to 825, REAL-01's line and row, the four marks, this summary | see the report |

## Guard records

Fourteen new, each measured with `scripts/guards.sh` against the whole
library; the arrived-since count 668 to 682.

| Record | Break | Red |
|---|---|---|
| a google calendar sync files every calendar on the account's list | the list's calendars not filed | 10 cases |
| a calendar google listed is filed under an id saying so | a plain id minted | 9 cases |
| the main google calendar's row is the one with a plain id | any Google row taken | the calendars case |
| a google calendar the account may only read is filed read-only | every calendar writable | the read-only case |
| a google calendar showing only free and busy times is passed over | it is filed | the free-and-busy case |
| a read never turns back whether a listed calendar is shown | visibility set from Google each read | the hidden case |
| a google read finds an event within its own calendar | the account-wide lookup | the two-calendars case |
| a cancellation in one google calendar leaves another calendar's copy | the account-wide delete | the two-calendars case |
| a google calendar put away keeps a change waiting in it | nothing kept | the put-away case |
| a google calendar list cut short is never read as the whole list | pages read before a failure kept | the cut-short case |
| a change in a second google calendar is sent to that calendar | every row answered as the main one | 6 cases |
| each listed google calendar is read from its own marker | no marker | the marker case |
| the calendar summary counts the calendars read when there is more than one | never counted | the counted case |
| folding calendar results keeps the calendars read | the fold line removed | the counted case and the fold case |

Re-anchored: "a Google day moved and then cancelled loses its appointment as
well as its place", whose anchor held the account-wide delete, now on
`delete_event_in`, measured again and agreeing. Re-measured in the same call:
the two task 1 records the count check named after task 2's red. Three came
back short, each because task 2's cases all reach the list filing or the
address; they were corrected by hand and measured again, agreeing. Premise
6's anchors in `sync_google_calendar` and `one_day_of_a_google_series` kept
their text, the read moved into `read_one_google_calendar` at the same depth;
`test_every_guard_record_still_names_one_place_in_the_tree` passed before
each green.

## Ledger

- 821 opened and fixed (`deviation`): a Gmail account's other calendars were
  never read.
- 822 opened and fixed (`deviation`): an event found by Google's identity
  across every calendar.
- 823 opened (`unrun-verify`): the sidebar with his own calendars, heard in
  sitting 1.
- 824 opened (`todo`): the sidebar finds a calendar by its row's text, so two
  of one name and visibility are one row to Enter.
- 825 opened (`todo`): a deletion note is asked across the account, so a
  meeting deleted from one calendar leaves the other calendar's copy un-updated
  while the note stands.
- Header: 701 open, 124 fixed, 825 in all.

## Deviations from Plan

1. **[Shape] The list is read by `every_google_calendar::sync`, not inside
   `calendar::sync_google_calendar`.** About forty existing cases drive
   `sync_google_calendar` against stand-ins that answer an exact sequence of
   replies; a request in front would have handed each one's first reply to
   the list. The spawn now calls the new entry (`wx_app.rs`, Rule 3, a file
   not in the plan's list), and the tracer runs it rather than
   `sync_google_calendar`. Premise 1's flow holds with the two steps swapped
   in name.
2. **[Rule 1] A listed calendar's id carries the account:
   `google:<account>:<Google's id>`.** The module note said
   `google:<its id there>`; the calendars table is keyed by the id alone, and
   the same calendar (a shared one, Google's holidays calendar) sits on two
   accounts' lists, which sitting 4 adds. The note is corrected.
3. **[Cost] No new case in `service/google_api.rs` (10 records, not 7) or
   `data/message_cache/calendar.rs` (8).** The list read and the lookups
   within a calendar are proven end to end in the new module, as 14-02 did
   with `asked_and_answered.rs`; the lookups' own rule is held by two
   records.
4. **[Shape] The marker for a listed calendar lives in the calendars table's
   `sync_token` column**, premise 5's second option, so it goes with the row;
   the main calendar's stays in `sync_state` as before.
5. **[Shape] The main calendar always starts shown**, a narrowing of decision
   2, because events made in no calendar go there; written on `file_one`.
6. **[Shape] A kept change goes to no calendar**, where the existing
   nothing-can-send sentence says on every sync that there is no calendar to
   send to, rather than a new sentence; decision 4's "kept and said".
7. **[Red] The two new sentences gained a closing full stop in the green**,
   which the red's expected values left off and `reads_as_a_persons_sentence`
   asks for; the summary's joiner takes it off and puts it back either way.
8. **[Red] Three task 2 cases passed at the red and were not named**: every
   role's answer to `may_write` (written in task 1's red), a list cut short
   putting nothing away (nothing put anything away yet), and a change in a
   second calendar going to its address (task 1's green made it true). Each
   is held by a record.
9. **[Scope] A third summary sentence**, for a calendar put away, beyond the
   plan's two clauses, since a calendar leaving the sidebar unsaid would be
   the silence #22 was about.
10. **[Process] The stream editor ran twice in read-only pipelines and awk
    once**, against the brief; no file was written by either.
11. **[Premise] Line numbers had moved by about 34** from the plan's reading
    at `01ef4589` (14-01 to 14-03 landed between); the guard register held
    1,465 records at the start, not 1,437.

## TDD Gate Compliance

Task 1 (tracer): `0f81c60a` red, accepted by `red-commit.sh`, then
`c7ecab6e`; the tracer's verify passed end to end before expanding. Task 2:
`62da5928` red, accepted, naming the count check with the eight cases and the
fold case, then `03a9111a`. Task 3 is documents. Every red ran against stubs
that compile and answer wrongly; each named case failed for the reason it
names, read before the green.

## Threat Flags

None beyond the register. T-14-16: the push addresses the row's own id, and
the change-sent case reads `PATCH /calendars/team%40group.calendar.google.com/events/team-evt`
from the stand-in. T-14-17: put away only on a whole list, a waiting change
kept, the cut-short case and its record. T-14-18: lookups and cancellation
within a calendar, two cases' worth of records. T-14-19: the list's path holds
no address, and a calendar id in an events path is masked by 14-02's line.
T-14-20: free-and-busy not filed, said once per sync. T-14-SC: nothing
installed.

## Self-Check: PASSED

`src/application/every_google_calendar.rs` exists; `0f81c60a`, `c7ecab6e`,
`62da5928` and `03a9111a` are on the branch.

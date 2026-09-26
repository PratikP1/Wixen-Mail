---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 12
subsystem: data::message_cache::calendar, application::answered_meetings, application::reading_a_message, application::calendar, application::caldav_sync, application::invitations, service::google_api, service::microsoft_graph
status: complete
tags: [invitations, calendar, sync, google, microsoft-graph, caldav, GAP-04]
requires: [13-11]
provides:
  - calendar_events.ical_uid and calendar_events.organiser, two additive columns kept off CalendarEventEntry
  - MessageCache::remember_where_it_came_from, get_event_by_ical_uid and the_organiser_on_the_calendar
  - GoogleEvent.ical_uid and organizer, MsGraphEvent.ical_uid and organizer, each the_organisers_address; never serialised
  - invitations::the_organiser_named_in, the ORGANIZER line read the way an invitation's is
  - calendar::save_what_google_sent and save_what_outlook_sent, the save and the UID together
affects: [13-13, 13-51]
tech-stack:
  added: []
  patterns:
    - "a writer that starts finding another writer's rows keeps that writer's identity columns rather than overwriting them"
key-files:
  created: []
  modified:
    - src/data/message_cache/mod.rs
    - src/data/message_cache/calendar.rs
    - src/application/answered_meetings.rs
    - src/application/reading_a_message.rs
    - src/application/calendar.rs
    - src/application/caldav_sync.rs
    - src/application/invitations.rs
    - src/service/google_api.rs
    - src/service/microsoft_graph.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/WINDOWS.md
key-decisions:
  - "An answer landing on a provider's row keeps the provider's identifier, source, version marker and link, or the next sync finds nothing under Google's id and files the meeting a third time."
  - "The organiser a provider recorded wins over the invitation's, so a stranger's invitation carrying a real meeting's UID cannot replace the organiser 13-13 will trust."
  - "Only whole meetings are written: the days of a Google series stored as rows of their own keep no UID, so a series invitation finds the series; subscribed feeds already carry the UID as their provider identifier."
metrics:
  duration: about 6 hours, most of it two guard re-measures of 14 and 98 records
  completed: 2026-09-26
estimate:
  tokens: 90000
  tasks: 3
actuals:
  tokens: 30000
  tasks: 3
  commits: 6
---

# Phase 13 Plan 12: Synced events carry their iCalendar UID and organiser Summary

Every meeting a Google, Microsoft or calendar server read saves now keeps its iCalendar UID
and its organiser, so an invitation finds the copy a provider filed: the reader says it is
already on the calendar rather than new, and answering it writes on that row, keeping the
provider's identifier, rather than filing the meeting a second time.

## What was built

- **Two additive columns.** `ical_uid` and `organiser` on `calendar_events`, through
  `ensure_column_exists` and in the rebuilt table, kept off `CalendarEventEntry` for the reason
  the answered version already is. `test_a_rebuilt_events_table_matches_a_new_one` holds the
  rebuild to the same shape.
- **The lookups.** `get_event_by_ical_uid(account, uid)` is one query: the row whose `ical_uid`
  is the UID first, else a row with no `ical_uid` whose provider identifier is the UID, which is
  every CalDAV row, every answer filed here and every row from before the column.
  `remember_where_it_came_from` writes both columns after the save; `the_organiser_on_the_calendar`
  reads one.
- **The answer and the reader.** `file_the_answer`, `invitation_check_for` and the buttons'
  `the_answer_given_to` look the meeting up by UID. An answer on an existing row keeps that row's
  provider identifier, source, version marker and link (`still_where_it_came_from`), and records
  the UID and the organiser, the one already on the row winning over the invitation's.
- **The three sources.** Google's `iCalUID` and `organizer.email`, Graph's `iCalUId` and
  `organizer.emailAddress.address`, each `skip_serializing`, so a change sent back carries
  neither key; the two save sites of each sync write them, created or updated alike, through
  `save_what_google_sent` and `save_what_outlook_sent`. The calendar server read writes
  `remote.uid` and the ORGANIZER line through `invitations::the_organiser_named_in`.

Verification, 2026-09-26 on the branch: `cargo test --lib data::message_cache::` 799 passed;
`application::answered_meetings::` 22; `application::reading_a_message::` 15;
`service::google_api::` 39; `service::microsoft_graph::` 59; `application::calendar::` 196;
`application::caldav_sync::` 68; `application::invitations::` 49; `--test
a_copy_leaves_the_original_where_it_was` 9, `a_moved_day_is_shown_once` 2,
`an_invitation_is_said_before_the_body` 10; `house_style`'s six guard checks pass. Acceptance
greps: `ical_uid` twice in `mod.rs`; `get_event_by_provider_id` in `answered_meetings.rs` only at
lines 378, 722 and 829, all inside `mod tests` from line 313; `iCalUID` four times in
`google_api.rs`, `iCalUId` four times in `microsoft_graph.rs`; `remember_where_it_came_from`
twice in `application/calendar.rs` and once in `caldav_sync.rs`.

## Commits

| Commit | What | Hook |
|--------|------|------|
| `cd32831e` | red: nine cases by UID and organiser, stubs answering nothing | 117 s, after one refusal by rustfmt at 9 s |
| `20197c45` | green: columns, lookups, the answer and the reader | 180 s |
| `ee6f6f8e` | red: seven cases for the three sources | 176 s |
| `270167ed` | green: the three sources write where a meeting came from | 196 s, after one refusal at 205 s (deviation 8) |

## Guard records

| Record | What |
|--------|------|
| a meeting a provider filed is found by the UID its invitation names | new, `calendar.rs`, 4 red |
| answering a meeting a provider holds writes on that row and adds none | new, `answered_meetings.rs` |
| Google's iCalUID is read under the name Google gives it | new, `google_api.rs`, 2 red |
| a Google read that updates a meeting writes where it came from | new, `application/calendar.rs` |
| three anchored on `cache.save_calendar_event(&merged)?;` at the two update sites | anchors rewritten to the new save calls, re-measured |
| eight named too few | corrected by hand and measured again; four for this plan's cases, four for tests 13-09 to 13-11 had added in files those records never named |
| the other 97 the count check flagged | re-measured, each unchanged |

Four records new; the arrived-since line at the head of `guards/guards.toml` went from 346 to 350.

## Deviations from Plan

**1. [Rule 1] An answer on a provider's row overwrote its identifier.** The plan's case asked
that answering a meeting Google holds "writes the answer on that row and adds none". Finding the
row was not enough: `the_row_an_answer_leaves` sets `provider_event_id` to the invitation's UID
and `source_provider` to nothing, and the save's `ON CONFLICT(id)` arm writes both, so the next
Google read would find nothing under `google-123` and file the meeting again. The answer keeps
the row's identifier, source, version marker and link; the case asserts them.

**2. [Rule 2] The organiser is not replaced by an invitation's (T-13-12-02).** A stranger's
invitation carrying a real meeting's UID would otherwise have written its organiser over the
provider's, which 13-13 trusts. A case holds it.

**3. [Shape] Not in the fresh `CREATE TABLE`.** The plan said "in the fresh schema". The tree
adds every later column through `ensure_column_exists` and names it in the rebuilt table, and
the rebuild test compares the two; the columns follow that.

**4. [Shape] Four callers, not two.** `reading_a_message` looks a meeting up twice since 13-11,
the sentence and the buttons; both go by UID, each with a case.

**5. [Shape] Two sync cases in `application/calendar.rs`, not one.** One for Google and one for
Outlook, each with one meeting updated and one created, so neither save site is untested.

**6. [Brief] `invitations.rs` edited.** Not in the plan's files: `the_organiser_named_in` sits
beside `the_meeting_named_in` so the calendar server read reuses the invitation's ORGANIZER
reader rather than a second one.

**7. [Premise] Three guard anchors inside the edited save sites.** The premise said no anchor lay
in `application/calendar.rs:815-845` or `:1170-1200`. Three records anchor on multi-line text
ending at the update arm's save line; `test_every_guard_record_still_names_one_place_in_the_tree`
refused them and they were rewritten to the new line and re-measured.

**8. [Process] The second green's first commit was refused.** The documents, edited in the
working tree while the 98-record re-measure ran, put `STATE.md` at plan 12 and the roadmap at
12 of 53 before this summary and the body's plan line existed, and the hook reads the working
tree. Finished before committing again.

**9. [Brief] `sed` ran once, read only.** A diagnostic pipeline carried `sed -n '1,0p'`, which
prints nothing, and two commands carried do-nothing names `sed_free=1` and `awk_free=0`. No file
was written through either. Logged as observation 0857.

### Found and left

- An answer filed before the calendar check has seen the meeting still creates an event under
  the invitation's UID, and the check then files the provider's copy beside it (ledger 154).
- Whether Google and Microsoft accept a guest's change pushed to their own copy is ledger 154's
  open half now.

## Threat Flags

None beyond the register. T-13-12-01: both fields `skip_serializing` on both types, held by
`test_the_meetings_uid_and_organiser_are_never_sent_back_to_google` and its Graph sibling.
T-13-12-02: `test_an_invitation_naming_another_organiser_does_not_replace_the_one_on_the_calendar`.
T-13-SC: no crate added; `Cargo.lock` unchanged.

## Known Stubs

None. Each stub answered nothing in its red commit only.

## Ledger

Opened: 635 (`unrun-verify`, phase 14's: whether each provider's UID is its invitations' UID, the
Graph series case, and the rows not written). Updated: 154, the order that is settled and the
half still open. Closed: none. Both halves of each.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does not
move.

## Self-Check: PASSED

- `cd32831e`, `20197c45`, `ee6f6f8e`, `270167ed`: in `git log` on
  `13-12-synced-events-carry-their-uid`.
- Every file in `key-files` exists.

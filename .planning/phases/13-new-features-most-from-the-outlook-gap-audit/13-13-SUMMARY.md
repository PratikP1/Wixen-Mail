---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 13
subsystem: application::meeting_changes, application::reading_a_message, application::invitations, data::message_cache::calendar, presentation::reader_text, presentation::wx_reader, presentation::page_jumps, presentation::wx_app
status: complete
tags: [invitations, calendar, reader, accessibility, keyboard, GAP-04]
requires: [13-12]
provides:
  - meeting_changes::what_opening_it_changes, MeetingChange and Why, one pure decision for what opening an update or a cancellation changes
  - meeting_changes::the_copy_moved, the calendar's copy at the update's time, pending
  - reading_a_message::what_opening_it_in_a_reader_changed, the reader windows' application call, a move saved before the document is built
  - WhatIsSaidAboutIt.change and ReaderDocument.removal, folded after the answer buttons and before the signature
  - MessageCache::mark_the_meeting_called_off, cancelled and free and pending, never a delete
  - ReaderWindow::on_remove, remove_now and remove_button_on, one builder and one handler for both message windows
  - page_jumps::Jump::Remove on Alt+R in the formatted window
affects: [13-14, 13-15, 13-51]
tech-stack:
  added: []
  patterns:
    - "a change a message asks of the calendar is applied only for the organiser the calendar's own copy records, never the one the message names"
key-files:
  created:
    - src/application/meeting_changes.rs
    - tests/a_meeting_change_reaches_the_calendar.rs
  modified:
    - src/application/mod.rs
    - src/application/invitations.rs
    - src/application/reading_a_message.rs
    - src/data/message_cache/calendar.rs
    - src/presentation/reader_text.rs
    - src/presentation/wx_reader.rs
    - src/presentation/page_jumps.rs
    - src/presentation/scan_fixtures.rs
    - src/presentation/wx_app.rs
    - tests/theme_reach.rs
    - guards/guards.toml
    - docs/development/requirements-backlog.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/REQUIREMENTS.md
    - .planning/WINDOWS.md
key-decisions:
  - "What a message would change is asked before who may change it, so a message that would change nothing says nothing; then the organiser, then a repeating meeting, then Allow Changes, because a stranger's message is the reason that matters and switching changes on would not make it apply."
  - "A move is applied only when the time differs: a newer version at the same time moves nothing, so reopening an update after it moved the meeting says nothing more."
  - "A repeating meeting is said and not changed, because a message about one day applied to the series copy would move or call off every day (ledger 638)."
  - "A later version at the time the calendar already holds is said to be already on the calendar, not a change 'which was' that same time."
  - "The preview asks nothing about the calendar; only open_single_message calls the seam, and a reading holds it."
metrics:
  duration: about 4 hours on 2026-09-26, about 70 minutes of it three guard re-measures of 4, 31 and 2 records
  completed: 2026-09-26
estimate:
  tokens: 120000
  tasks: 3
actuals:
  tokens: 30000
  tasks: 3
  commits: 5
---

# Phase 13 Plan 13: An organiser's update moves the meeting and a cancellation is removed with one button Summary

Opening an update from the meeting's organiser in the text reader or the formatted window now
moves the meeting on the calendar, and the bar says "Moved on your calendar from 05/03/2026 at
09:00 to 06/03/2026 at 14:00." A cancellation from the organiser gets one native button, &Remove
from Calendar, where the answer buttons go in both windows, on Alt+R, which marks the meeting
cancelled and free, keeps the row, and says "Removed from your calendar." once. A change from
anybody else, for a meeting that records no organiser, for a repeating meeting, or with calendar
changes switched off is said with its reason and not applied. The preview changes nothing.
GAP-04 is ticked.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards` and `docs`
against `main` at `95a9f508`, rounded.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `42b9aa0b` | red | the decision's 12 cases, the invitation's already-there case and the count check; decision and move stubbed | 141 s, after a rustfmt refusal at 12 s and a house-style refusal at 170 s |
| `0a4293ac` | green | `what_opening_it_changes`, `the_copy_moved`, the standing fix; 2 records new, 3 re-measured, 1 corrected | 203 s |
| `dc8312c5` | red | the store, fold, page and scan cases, the new target's 16 readings and the count check; four fold-chain anchors rewritten | 276 s, after a refusal at 213 s (deviation 6) |
| `76dff944` | green | the store call, the reader's application call, the fold, the button in both windows, Alt+R, the handler, the seam, the scan fixture; 4 records new, 5 rewritten, 26 re-measured, 1 of 13-12's corrected | 266 s |
| this commit | docs | the pages, the backlog line, the changelog, the ledger, this summary, the four marks | |

**Test counts, taken on 2026-09-26.** `cargo test --lib application::meeting_changes::` 18 (the
plan asked for at least 9). `application::invitations::` 50 (37 at the plan's premise, 49 after
13-12). `data::message_cache::calendar::` 47. `presentation::page_jumps::` 8.
`presentation::reader_text::` 139. `presentation::wx_reader::` 14 (no case added: the windows
are read by the target). `presentation::scan_fixtures::` 13. `application::reading_a_message::`
15. `cargo test --test a_meeting_change_reaches_the_calendar` 19 (the plan asked for at least 6).
`the_invitation_is_answered_from_the_reader` 12, `the_page_window_keeps_the_document_focused_when_it_comes_back`
4, `print_is_on_the_file_menu` 9, `wired` 77, `an_invitation_is_said_before_the_body` 10,
`theme_reach` 7, `attachments_are_reached_with_alt_a_in_both_views` 10, all passing.

**Acceptance readings.** `grep -c '&Remove from Calendar'` finds 1 in `wx_reader.rs`, where
`THE_REMOVE_BUTTON` holds the label, and 0 in `wx_app.rs`: the formatted window builds the
button through `ReaderWindow::remove_button_on`, the one builder, as 13-11's buttons are built
(deviation 3). `grep -c "kind: 'remove'" src/presentation/page_jumps.rs` 2: one in the script,
one in the case that reads it, as 13-11 found for `answer`. `grep -c 'Invitations are read,
answered and filed | Done' docs/development/requirements-backlog.md` 0. GAP-04's line begins
`- [x] **GAP-04**`. Carriage returns 0 on every document touched, by `tr -cd '\r' < FILE | wc -c`.

**The letters, checked as one set per window.** The reader's tab after 13-11: A&ccept (C),
&Tentative (T), &Decline (D), the attachments' Alt+A, the menu titles &File and &Go; the page's
listener F7, Alt+A, Ctrl+P, Alt+C, Alt+T and Alt+D; `page_links.rs` Backspace and Alt+Left. The
Reader Window section of `docs/KEYBOARD_SHORTCUTS.md`, read with Grep for `Alt+`, held Alt+A,
Alt+C, Alt+T and Alt+D. R was free in both windows and is Remove from Calendar's.

**Where the keyboard lands after Alt+R.** In the text reader the panel's dialog manager presses
the button from the message's text; the target posts `WM_SYSKEYDOWN` for R with Alt and reads
one press carrying `evt-1`. The window a test builds is not in front, so where the keyboard ends
up is the ear's (ledger 636), as it was for Alt+C.

**GAP-04's clauses**, each on a named test, are written into its `[D]` line in
`.planning/REQUIREMENTS.md`: 13-10's two raw-message readings, 13-11's two button readings and
two answering cases, 13-12's sync case, and this plan's update, button, removal and stranger
readings.

## Guard records

| Record | What |
|--------|------|
| a meeting change from anybody but the organiser recorded is said and not applied | new, `meeting_changes.rs`, 2 red |
| a meeting change while calendar changes are switched off is said and not applied | new, `meeting_changes.rs`, 1 red |
| a meeting called off is marked and kept rather than deleted | new, `calendar.rs`, 1 red |
| what opening a message changed is folded in before the signature, so it is spoken | new, `reader_text.rs`, 1 red |
| the preview never applies what a message changes on the calendar | new, suite the target, 1 red |
| the page's script posts remove for alt+r, not nothing | new, `page_jumps.rs`, 2 red |
| the envelope, the meeting (two) and why a meeting cannot be answered folded before the signature | anchors rewritten for the chain's new link, re-measured |
| a message tab in the reader is handed what it prints | anchor rewritten to the tab's new two lines, re-measured |
| a moment written with a T is one the reader knows | corrected by hand, two of this plan's cases added, re-measured |
| a meeting a provider filed is found by the UID its invitation names | 13-12's, corrected by hand: two of 13-12's own sync cases it reddened and did not name |
| 22 more flagged by the count check | re-measured, each unchanged |

Six records new; the arrived-since line at the head of `guards/guards.toml` went from 350 to 356.
The plan named three records for task 2; the fold-order record is a fourth, because a new link
in that chain is the thing this project has had to hold four times already. The anchors premise
4 named: `page_jumps.rs:50` and `:62` are not duplicated; the answer-button row's record ends at
the row's closing line and the button is built after it; 13-11's answer arm is untouched and the
remove arm sits after it.

## Deviations from Plan

**1. [Rule 1] The invitation's own sentence described a move already made as one to come.**
After an update moved the meeting on opening, 13-10's sentence, asked against the moved copy
and a version answered earlier, said "a change to the meeting on your calendar, which was" the
very time it had just named. A later version at the time the calendar holds now reads "and it
is already on your calendar", which is also true when a provider applied the update itself. A
case in `invitations.rs` holds it.

**2. [Rule 2] A repeating meeting is said and not changed.** Not in the plan. A message about
one day of a series, applied to the series' copy, would move or call off every day. `Why` has
`ARepeatingMeeting` and a case; the recommendation is ledger 638.

**3. [Shape] The formatted window's button is built by the reader's builder.** The acceptance
grep asked for `&Remove from Calendar` in `wx_app.rs` too. The label lives once, in
`wx_reader::THE_REMOVE_BUTTON`, and both windows build the button through
`ReaderWindow::remove_button_on`, the way 13-11 built the answer buttons, so the two cannot
come to letter it differently. The target reads the label on both windows' buttons.

**4. [Shape] `Why::NotOnTheCalendar` left out; `Why::CouldNotBeSaved` added.** A cancellation
or update for a meeting the calendar does not hold already says so in the invitation's own
sentence, so a second sentence would be the same fact twice; it is `Nothing`. A move that could
not be saved says so rather than failing quietly.

**5. [Shape] The decision's order and its signature.** The plan put Allow Changes "before
anything else". A message that would change nothing now says nothing, then the organiser is
asked, then a repeating meeting, then Allow Changes: with Allow Changes first, every invitation
on an account with calendar changes off would have carried a sentence, and a stranger's message
would have been told to switch changes on. The signature takes the copy, its organiser and the
answered version together in `TheCalendarsCopy`, and a `DateSettings`, because the sentence
words the times.

**6. [Rule 3] The second red's first commit was refused** by
`test_every_guard_record_still_names_one_place_in_the_tree`: four records anchor on the fold
chain, and the red added a link to it. The anchors were rewritten in the same red commit and
re-measured in the green.

**7. [Rule 1] A move is applied only when the time differs.** The plan moved the copy "when its
version is above the version answered here". A newer version at the same time moves nothing, so
"moved from nine to nine" is never said and reopening an update after it moved the meeting says
nothing more. Where only the end moved, the sentence says both whole times.

**8. [Rule 1] A test comment claimed a new installation has calendar changes off.** The first
red was refused by `test_nothing_says_a_new_installation_changes_nothing_while_it_changes_contacts`:
a new installation allows personal information. The comment was corrected. The same test refused
a changelog sentence later; it reads "Opening a message left your calendar as it was" now.

**9. [Brief] `awk` ran once and two do-nothing names were written.** A pipeline finding which
function encloses a line carried `awk -F: '$1<20485'`; read only, nothing written. Two commands
carried `sed_free=;` and `sed_x=0;`, which run nothing. Logged as observation 0858. The new target,
untracked and half-written, was moved out of the tree with `mv` before the first green's commit
so the hook's clippy over every target would not compile it, and copied back with `cp` before
the second red; no tracked file was moved or written that way.

**10. [Brief] Documents in the working tree before the second green.** The documents were
written while the 31-record re-measure ran, and the hook reads the working tree, so this summary
was written before the second green rather than after it, as 13-12's deviation 8 found.

### Found and left

- A record of 13-12's named two tests short; corrected and re-measured here.
- Where the keyboard lands after Alt+R in a window in front (ledger 636).

## Threat Flags

None beyond the register. T-13-13-01: `the_organiser_sent_it` compares the sender with the
organiser the calendar's copy records, and a copy recording nobody is changed by nobody; a guard
record and `test_a_cancellation_from_somebody_other_than_the_organiser_changes_nothing`.
T-13-13-02: `mark_the_meeting_called_off` updates and never deletes, only on a press; a guard
record and the target's reading that the handler calls no delete. T-13-13-03: accepted and
ledgered for phase 14 (637). T-13-13-04: `what_opening_it_in_a_reader_changed` reads only stored
parts. T-13-SC: no crate added; `Cargo.lock` unchanged.

## Known Stubs

None. Each stub answered nothing in its red commit only.

## Ledger

Opened: 636 (`unrun-verify`, the new sentences, the button and Alt+R heard), 637
(`unrun-verify`, phase 14's: a real organiser's update and cancellation on each provider, a
change pushed after the provider applied it, a forged From), 638 (`todo`, a repeating meeting's
update or cancellation, Pratik's to schedule). Updated: 154, moves and removals pushed the same
way as an answer. Closed: none. Both halves of each; 578 open of 638.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does
not move.

## Self-Check: PASSED

- `src/application/meeting_changes.rs`, `tests/a_meeting_change_reaches_the_calendar.rs`:
  present.
- `42b9aa0b`, `0a4293ac`, `dc8312c5`, `76dff944`: in `git log` on
  `13-13-a-meeting-change-reaches-the-calendar`.

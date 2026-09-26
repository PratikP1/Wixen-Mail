---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 11
subsystem: application::answering, application::answered_meetings, application::reading_a_message, application::context_menu, data::message_cache::calendar, presentation::reader_text, presentation::wx_reader, presentation::page_jumps, presentation::wx_app
status: complete
tags: [invitations, calendar, reader, accessibility, keyboard, context-menu, GAP-04]
requires: [13-10]
provides:
  - answering::the_answer_buttons, AnswerButtons and TheButtons, one decision for whether a message's invitation is offered Accept, Tentative and Decline and what each says
  - reading_a_message::answer_buttons_for, AnsweringAs and carries_a_calendar_part; WhatIsSaidAboutIt.answering, folded after the meeting and before the signature
  - answered_meetings::the_invitation_on, the answer path from a message row with the headers a reply threads by
  - calendar_events.answered_with, remember_the_answer and the_answer_given_here, the answer given kept beside the version
  - ReaderWindow::on_answer, answer_now and answer_buttons_on, one builder and one handler for both message windows
  - page_jumps::Jump::Answer on Alt+C, Alt+T and Alt+D in the formatted window
  - context_menu::entries_for_a_message_carrying_an_invitation, three answers on I, E and L
affects: [13-12, 13-13, 13-51]
tech-stack:
  added: []
  patterns:
    - "a control a person presses is offered only when pressing it can work, and its absence is said where the person reads"
key-files:
  created:
    - tests/the_invitation_is_answered_from_the_reader.rs
  modified:
    - src/application/answering.rs
    - src/application/answered_meetings.rs
    - src/application/invitations.rs
    - src/application/reading_a_message.rs
    - src/application/context_menu.rs
    - src/application/attaching.rs
    - src/data/message_cache/calendar.rs
    - src/data/message_cache/messages.rs
    - src/data/message_cache/mod.rs
    - src/presentation/reader_text.rs
    - src/presentation/wx_reader.rs
    - src/presentation/page_jumps.rs
    - src/presentation/wx_context_menu.rs
    - src/presentation/scan_fixtures.rs
    - src/presentation/wx_app.rs
    - tests/a_conversation_row_stands_for_one_message.rs
    - tests/every_command_acts_on_the_selection.rs
    - tests/an_invitation_is_said_before_the_body.rs
    - tests/theme_reach.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "The answer given is stored as invitations::Answer converted at the SQL boundary, not a second AnswerGiven enum with the same three variants."
  - "No key is bound for the reader's buttons: the panel's dialog manager presses a button by its letter from any control in the tab, and a binding as well would press twice."
  - "The buttons' sentences are decided in application and carried in WhatIsSaidAboutIt, so every surface that folds what is said about a message gets them in the one order that keeps the reason spoken."
  - "The context menu offers the three answers on a recorded calendar part without parsing it; the path they reach refuses a cancellation with its reason."
  - "Every outcome of answering is said once, through the status line, which speaks what it shows."
metrics:
  duration: "about 5 hours on 2026-09-26, most of it guard re-measurement"
  completed: 2026-09-26
actuals:
  tokens: 43000
  tasks: 4
  commits: 7
---

# Phase 13 Plan 11: Accept, Tentative and Decline where an invitation is said Summary

A message whose invitation can be answered now has three native buttons, A&ccept, &Tentative
and &Decline: in the text reader's tab after the bar and before the message, in the formatted
window after the page and before the attachments. Each is named for its answer and described by
what pressing it will do and who will be told. Alt+C, Alt+T and Alt+D press them in both
windows. An invitation that cannot be answered has no buttons and says why in the bar, above
the signature's account. The message list's context menu offers the three answers on a message
with a calendar part. Answering takes the message it was pressed on, keeps the answer beside the
version, threads the reply under the invitation, and says what happened once.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards` and `docs`
against `main` at `a2958da4`, rounded.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `98a68b5f` | red | the answer kept, the reply threaded, the path from a row; 7 cases and the count check | 228 s, after one rustfmt refusal at 12 s |
| `a191f3ad` | green | `answered_with`, `remember_the_answer`, threading, `the_invitation_on`, `answer_the_invitation` from a row and said once; 2 records new, 3 corrected, 7 re-measured | 253 s |
| `92e63552` | red | the buttons decided, folded and built in the reader's tab; the new target; 9 cases and the count check | 232 s |
| `b95c13da` | green | `the_answer_buttons`, `answer_buttons_for`, the fold, the reader's row, the main window's handler, the scan fixture, the reader's keys on the shortcuts page; 3 records new, 3 rewritten, 18 re-measured | 279 s |
| `1b5d8cbc` | red | the page's answer keys, the formatted window's buttons, the context menu's answers; 7 cases and the count check | 208 s, after one refusal at 204 s |
| `96af6bc1` | green | the script, the page window's buttons and arm, the context menu, the said-once reading; 4 records new, 3 corrected, 7 re-measured | 241 s |
| this commit | docs | the guide, the changelog, the ledger, this summary, the four marks | |

**Test counts, taken on 2026-09-26 against the plan's of 2026-09-24.**
`cargo test --lib data::message_cache::calendar::` 42 (39 at the start). `application::answered_meetings::`
19 (17). `application::answering::` 39 (38). `application::invitations::` 49 (48).
`application::reading_a_message::` 13 (9). `presentation::reader_text::` 135 (131).
`presentation::wx_reader::` 14 (14, no case added: the window is read by the target).
`presentation::page_jumps::` 7 (5). `application::context_menu::` 19 (18).
`presentation::wx_context_menu::` 2 (2). `presentation::scan_fixtures::` 12 (11).
`cargo test --test the_invitation_is_answered_from_the_reader` 12 (the plan asked for at least
5). `attachments_are_reached_with_alt_a_in_both_views` 10, `the_page_window_keeps_the_document_focused_when_it_comes_back`
4, `wired` 77, `an_invitation_is_said_before_the_body` 10, `theme_reach` 7, all passing.

**Acceptance readings.** `grep -c 'answered_with' src/data/message_cache/mod.rs` 2.
`grep -c 'selected_message_index' src/presentation/wx_app.rs` 29, 30 at `a2958da4`.
`grep -c 'A&ccept' src/presentation/wx_reader.rs` 1. `grep -c 'Enable(false)\|enable(false)'
src/presentation/wx_reader.rs` 0 before and after. `grep -c "kind: 'answer'"
src/presentation/page_jumps.rs` 2: one in the script, one in the case that reads it.
`grep -c 'Accept &invitation' src/application/context_menu.rs` 3: two menus and the case. The
Reader Window section of `docs/KEYBOARD_SHORTCUTS.md` holds a row each for Alt+C, Alt+T and
Alt+D. Carriage returns 0 on every document touched, by `tr -cd '\r' < FILE | wc -c`.

**GAP-04's clauses.** "With Accept, Tentative and Decline":
`test_an_answerable_invitation_has_three_named_described_buttons` and
`test_the_formatted_window_has_the_three_buttons_after_the_page`. "The answer sent through the
outbox and the gate": `application::answering::invitations_from_strangers::test_nothing_is_ever_answerable_with_sending_switched_off`
and `application::answering::tests::test_the_answer_goes_out_threaded_under_the_invitation`. The
box waits for 13-13.

**Where Alt+C arrives.** In the text reader, read from wx's source before anything was bound:
`wxWindowMSW::MSWProcessMessage` hands a `wxPanel`'s key messages to `IsDialogMessage`, which
finds a `WM_SYSCHAR`'s letter among the panel's children wherever the keyboard is. The target
posts `WM_SYSKEYDOWN` for C with Alt to the message's text through the window's loop, and Accept
is pressed once with the tab's row; nothing is bound on the text, and a binding as well would
have pressed twice. In the formatted window it was not measured with a real key: the browser
keeps every key once it has focus (#84) and a key typed there arrives from another process, so
measuring would have meant typing into this machine's foreground window. The page's listener is
the route, as the plan said it would be either way.

**Where the keyboard lands after Alt+C.** The plan said the button pressed. In a window the
target builds, which is not in front, it stays in the message; the reading requires one of the
two and prints which. The ear settles it (ledger 634).

**The mnemonics, checked as one set per window.** The reader's tab: C, T and D against the
tab's Alt+A and the menu bar's F and G, all distinct; the File menu's R, S, P, C, W and the Go
menu's N, P, A, F are its own. The formatted window: C, T and D against the page's Alt+A and F7
and no menu bar. The context menu: I, E and L against R, A, F, M, S, D, P, V, Y, T, C and N,
held by `test_no_two_entries_in_one_menu_share_a_mnemonic`, which now walks both of the new
menus; J stays free for 13-22.

**The anchors premise 5 named.** `wx_app.rs`'s flush block inside `answer_the_invitation` still
names one place and was not re-indented. The page window's browser-first and keyboard-back
anchors are untouched; the buttons are built after the page. `page_jumps.rs:50` and `:62` are
duplicated by nothing. `answered_meetings.rs`'s save and identity anchors are untouched; the
version write became the answer write beside them. The context menu's three anchors still name
one place; the new entries are separate statics beside the old ones. `mod.rs`'s twelve anchors
are untouched.

## Guard records

| Record | What |
|--------|------|
| the answer given is remembered beside the version | new, `calendar.rs` |
| the answer to a meeting goes out threaded under the invitation | new, `answering.rs` |
| why a meeting cannot be answered is folded in before the signature, so it is spoken | new, `reader_text.rs` |
| a message that cannot be answered is offered no buttons | new, suite the target |
| each answer button is described by what pressing it will do | new, suite the target, both windows red |
| the page's script posts the answer for alt+c, alt+t and alt+d, not nothing | new, `page_jumps.rs` |
| the formatted window's answer keys press the buttons' handler with its own message | new, suite the target |
| the formatted window offers the three buttons for a message that can be answered | new, suite the target |
| the three answers are offered only on a message carrying an invitation | new, `context_menu.rs` |
| the envelope, the meeting and the raw signed invitation folded before the signature | three rewritten for the chain's new link and measured again |
| three on `answered_meetings.rs` and one on the read form of the message menu | corrected by hand for cases this plan added, measured again |
| thirty more flagged by the count check | re-measured, each unchanged |

Nine records new; the arrived-since line at the head of `guards/guards.toml` went from 337 to 346.

## Deviations from Plan

**1. [Shape] One enum, not two.** The plan named `AnswerGiven { Accepted, Tentative, Declined }`.
`invitations::Answer` already has those three; it gained `as_stored` and `from_stored`, and
the data layer converts at the SQL boundary with it. A word nothing here wrote reads as no
answer.

**2. [Shape] No date beside the answer.** The must-haves quoted "you accepted this version on
...". The behaviour the plan specified keeps the version and the answer, not a moment, so the
sentence is "and you accepted this version". Storing when would be another column.

**3. [Rule 1] Two source readings named `answer_the_invitation` as reading the selection.**
`tests/every_command_acts_on_the_selection.rs` and `tests/a_conversation_row_stands_for_one_message.rs`
listed it among the commands acting on the cursor; the plan's premises did not list either
file. Both now leave it out and say why (observation 0847).

**4. [Rule 1] Accepting was heard twice.** Every outcome went to the status line, which speaks
what it shows, and was announced beside it. It goes through the status line alone now, a
refusal through the refusal line; the target holds it with a companion. This settles ledger
155's first question by structure.

**5. [Shape] One source for who answers.** `answer_the_invitation` read the address from the
window's state and the buttons would have read it from the store; both read
`reading_a_message::AnsweringAs::on` now, the account's own row and `allowed_for`, and the
state-based helper is gone.

**6. [Rule 3] The target waits for its browsers.** The first attempt at the third red ended the
loop while WebView2 was still making the formatted window's browser, and the target died with
0xc000041d. The session now waits for each browser to exist before it ends.

**7. [Test] The press in the formatted window is a click's command.** The plan asked for a
posted `{kind:'answer',answer:'decline'}` to reach the handler. A script message cannot be
posted into the page from a test without the browser's own loop, so the page's half is held by
`page_jumps`' cases and a reading of the arm with a companion, and the window's half by Decline
pressed through the command a click sends, which `BM_CLICK` would refuse in a window that is not
in front.

**8. [Test] The scan fixture's case was written with the fixture.** `scan_fixtures`' file asks one
case of each fixture; `test_the_invitation_is_offered_its_three_buttons` arrived in the green,
not ahead of it.

**9. [Brief] No `awk`.** The plan's acceptance line for the shortcuts page uses `awk`; the
section was read with Read instead.

### Found and left

- Where the keyboard lands after Alt+C in a window in front (ledger 634, and a question for
  Pratik if the ear disagrees with keeping it in the message).
- The formatted window's buttons are built for one message only; a conversation of several
  shows none, as the plan said, and each message's sentence still says what it is.

## Threat Flags

None beyond the register. T-13-11-01: the arm acts only on a window whose one message was
offered the buttons, and the answer still goes through `whether_it_can_be_answered`, the
account's Allow Changes answer and the outbox's hold. T-13-11-02: the row travels with the
press; the target asserts the row carried in both windows. T-13-11-03:
`test_a_stored_answer_this_program_never_wrote_reads_as_no_answer`. T-13-SC: no crate added;
`Cargo.lock` unchanged.

## Known Stubs

None. Each stub answered nothing in its red commit only.

## Ledger

Opened: 634 (`unrun-verify`, the buttons, their descriptions, the keys and the context menu
heard in both windows, and where the keyboard lands). Updated: 155, its first question settled
by structure. Closed: none. Both halves of each.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does
not move.

## Self-Check: PASSED

- `tests/the_invitation_is_answered_from_the_reader.rs`: present.
- `98a68b5f`, `a191f3ad`, `92e63552`, `b95c13da`, `1b5d8cbc`, `96af6bc1`: in `git log` on
  `13-11-answer-buttons-in-the-reader`.

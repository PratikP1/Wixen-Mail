---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 10
subsystem: application::invitations, application::answering, application::reading_a_message, application::mail_sync, presentation::reader_text
status: complete
tags: [invitations, calendar, reader, accessibility, GAP-04]
requires: [13-09]
provides:
  - invitations::WhatTheInvitationSays and what_the_invitation_says, one pure decision for an invitation, a cancellation, an answer or a calendar file
  - answering::is_a_calendar_part, taking text/calendar and application/ics, shared by the finder and the attachment row
  - reading_a_message::WhatIsSaidAboutIt.invitation, asked by for_message from the stored parts and the calendar of the message's own account
  - reading_a_message::keep_what_a_download_carried, the download of everything keeping each part's name and the calendar document
  - ReaderDocument::with_invitation, folded pgp, envelope, invitation, signature
affects: [13-11, 13-12, 13-13, 13-14, 13-15, 13-51]
tech-stack:
  added: []
  patterns:
    - "what is said before a message is decided in application and folded by the one composition every surface calls"
key-files:
  created:
    - tests/an_invitation_is_said_before_the_body.rs
  modified:
    - src/application/invitations.rs
    - src/application/answering.rs
    - src/application/reading_a_message.rs
    - src/application/mail_sync.rs
    - src/presentation/reader_text.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_reader.rs
    - tests/theme_reach.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "The fold is pgp, envelope, invitation, signature, so the meeting is above 'More about this signature:' and spoken."
  - "An invitation's date is always written in full through the reader's date settings, never as '3 days ago'."
  - "A fifth standing, already on your calendar, for an invitation a calendar server filed itself (Google does), beside new, changed and already answered."
  - "The download of everything keeps every part's name, type and size and the calendar document's bytes, so a message it brought lists its attachments and says its meeting; other files stay on the server until opened."
  - "The row table says 'calendar file' for text/calendar and application/ics; the message's decision names the row where it has one."
metrics:
  duration: "about 80 minutes of code on 2026-09-25 by the first executor, then a stall, then the documents and the merge on 2026-09-26 by a second"
  completed: 2026-09-26
actuals:
  tokens: 23573
  tasks: 3
  commits: 5
---

# Phase 13 Plan 10: The invitation said before the body Summary

A message carrying a meeting now says what it is before a word of the message, at the top
of the bar and again at the top of the message, in the text reader, the formatted window
and the preview: "Meeting invitation: Quarterly review, 05/03/2026 at 09:00 to 10:00, in
Room 4, from Ada Lovelace, and it is new to your calendar." It ends by saying whether the
meeting is new, a change to the one on the calendar and when that was, a version answered
here already, or one already on the calendar. A cancellation says so and whether the
meeting is here, an answer to your meeting says who answered and what, and a calendar
document that asks nothing says it is a calendar file. The attachment row says the same in
a word or two. An invitation sent as `application/ics` is found as one sent as
`text/calendar`, here and by Answer Invitation. On a signed message the meeting comes
before the account of the signature, so it is spoken.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards` and `docs`
against `main` at `aeb0c0c1`.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `e7f15b8e` | red | the types, the field and the wiring with the decision and the finder stubbed; 14 cases and the count check | not recorded |
| `9f94755a` | green | the decision, the finder taking `application/ics`, the fourth field; two records new, six re-measured | not recorded |
| `d4e95844` | red | the fold, the row's words and the new target, `with_invitation` and `keep_what_a_download_carried` stubbed; 13 cases and the count check | not recorded |
| `8c21e045` | green | the fold, the row, the download keeping parts; the envelope record rewritten, five new, two corrected | not recorded |
| this commit | docs | the guide, the changelog, the ledger, this summary, the four marks | |

The first executor made the four code commits between 15:38 and 16:54 on 2026-09-25 and
did not record the hook times; the commit timestamps are all that remains of them.

**Test counts, taken again on 2026-09-26.** `cargo test --lib application::invitations::`
48 (37 on 2026-09-24). `application::answering::` 38 (37). `application::reading_a_message::`
9 (7). `presentation::reader_text::` 131 (125). `presentation::html_renderer::` 92,
unchanged, not edited. `cargo test --test an_invitation_is_said_before_the_body` 10 (the
plan asked for at least 4). `an_encrypted_message_is_not_left_unexplained` 10, passing.
`house_style` 74, `the_words_that_say_nothing` 10,
`the_planning_files_agree_with_themselves` run before this commit.

**Acceptance readings.** `grep -c 'application/ics' src/application/answering.rs` 2.
`grep -c 'pub invitation' src/application/reading_a_message.rs` 1.
`grep -c 'with_invitation' src/presentation/reader_text.rs` 3.
`grep -c '"calendar invitation"' src/presentation/reader_text.rs` 0.
`grep -c '^### Meeting invitations' docs/USER_GUIDE.md` 1. Carriage returns 0 on
`docs/USER_GUIDE.md`, `docs/changelog.md` and `.planning/WINDOWS.md`, by
`tr -cd '\r' < FILE | wc -c`; `house_style` holds the em dash and the six words.

**GAP-04's first `[D]` clause, "the invitation's part shown and said before the body",**
is held by `test_a_raw_invitation_is_said_at_the_top_of_the_bar_and_the_body` and
`test_the_formatted_window_and_the_preview_say_the_invitation_before_the_message` in the new
target. The box waits for 13-13.

**Where the parts come from when a message opens.** The reader reads the stored parts
(`cache.attachments_with_content`). They are stored when the reader fetches a whole
message it does not hold. A message whose text the download of everything brought was
never fetched again, and the text pass kept only the text, so such a message listed no
attachments and its meeting could not have been said. The second green makes the text pass
keep every part's name and the calendar document through
`reading_a_message::keep_what_a_download_carried`, called from `mail_sync.rs`. Messages
brought that way before this build still have nothing recorded (ledger 633).

**The page's section.** The plan put the sentence into `html_renderer.rs`'s security
section. It was not needed: the formatted window draws the bar the composer answers above
its page, and `reader_text::preview_html` renders the bar into the preview before the
title, so both carry the sentence from the one fold. `html_renderer.rs` is not edited and
its ten anchored records are untouched.

## Guard records

| Record | What |
|--------|------|
| a cancellation is said as one and never as an invitation | new, `invitations.rs` |
| an invitation sent as application/ics is found | new, `answering.rs` |
| six the count check flagged on `answering.rs` and `reading_a_message.rs` | re-measured in one call with the two above, each unchanged |
| the envelope is folded in before the signature, so it is spoken | anchor rewritten to the four-line chain, re-measured |
| the meeting is folded in before the signature, so it is spoken | new, library cases |
| a raw signed invitation says the meeting above the signature's account | new, suite the target |
| a calendar part's row says what the message decided it asks | new |
| a calendar part named like a program is still called a program | new |
| a download leaves the files the reader kept where they are | new |
| the attachment row's record | corrected by hand, two of 13-10's whole-row cases added |
| the date picture's record | corrected by hand, nine printing cases from 13-02 and 13-04 added |

Seven records new; the arrived-since line at the head of `guards/guards.toml` went from 330
to 337. `test_every_guard_record_still_names_one_place_in_the_tree` and
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` pass on the branch.
The other `reader_text.rs` records the plan named by line (`:336-338` twice, `:777`,
`:1009`, `:1025-1028`, `:1331`, `:1357-1359`, `:1803`, `:1826`, `:1854-1858`) are
untouched apart from the two corrected above.

## Deviations from Plan

**1. [Process] Finished by a second executor after the first stalled.** The first executor
made the four code commits on 2026-09-25 and stopped responding with the documents commit in
progress: the guide, the changelog and four ledger rows written and uncommitted. A second
executor read the four diffs against the plan, found tasks 1 and 2 done, re-ran their
verification, checked the uncommitted edits, and wrote this summary, the marks and the
documents commit. The hook times of the code commits were not recorded.

**2. [Rule 1] The guide's "What this does not do yet" tripped `house_style`.** "Nothing on
your calendar changes when a message is opened" read to
`test_nothing_says_a_new_installation_changes_nothing_while_it_changes_contacts` as a claim
that a new installation changes nothing. It reads "opening a message leaves your calendar
as it was" now.

**3. [Rule 1] The ledger's frontmatter counts were eight behind before this plan.** At
`aeb0c0c1` the frontmatter said 621 entries and 561 open against 629 rows, 569 open, in
both halves; 13-05, 13-06, 13-07 and 13-09 added rows without moving the counts. With this
plan's four they read 633 total, 573 open, 60 fixed, 0 waived, counted from the table and
the JSON alike.

**4. [Rule 2] The download of everything keeps parts.** Not in the plan: found by the
premise check before the second red, above.

**5. [Shape] A fifth standing.** "Already on your calendar", for an invitation a calendar
server filed itself, beside new, changed and already answered.

**6. [Shape] The date is the reader's full date**, "05/03/2026 at 09:00 to 10:00" in the
default settings, rather than the plan's "Thursday 5 March 2026 from 09:00 to 10:00".

**7. [Shape] `html_renderer.rs` not edited**, above.

### Found and left

- A TZID time is read as that hour on this clock, as the whole calendar reads one (ledger 632).
- Messages the download brought before this build have no parts recorded (ledger 633).
- Shift+Space reads the sentence twice, from the bar and the body, as it already does an
  S/MIME envelope's sentence (in ledger 630).

## Threat Flags

None beyond the register. T-13-10-01: the sentence says "from" the organiser the document
names and the sender line stays; nothing on the calendar changes. T-13-10-02: a document
that does not parse reads as a calendar file (`test_an_invitation_that_does_not_read_is_a_calendar_file_and_not_silence`),
and a stranger's words are put on one line and cut at 200 characters so a title cannot
start a line of the bar. T-13-SC: no crate added; `Cargo.lock` unchanged. The download now
writes calendar documents to the attachment store, the same store and budget the reader
already writes to.

## Known Stubs

None. `with_invitation` and `keep_what_a_download_carried` were stubs in the second red
only.

## Ledger

Opened: 630 (`unrun-verify`, the sentence heard in the three surfaces), 631
(`unrun-verify`, a real organiser's invitation, cancellation and answer, phase 14's), 632
(`todo`, time zones), 633 (`todo`, messages downloaded before this build). Closed: none.
Both halves of each.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does
not move.

## Self-Check: PASSED

- `tests/an_invitation_is_said_before_the_body.rs`, `src/application/invitations.rs`: present.
- `e7f15b8e`, `9f94755a`, `d4e95844`, `8c21e045`: in `git log` on
  `13-10-invitation-said-before-body`.

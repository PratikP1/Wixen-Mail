---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 30
subsystem: free/busy
tags: [free-busy, caldav, microsoft-graph, google-calendar, GAP-08, "#57"]
status: complete
requires: [13-29]
provides:
  - "asking_when_free: every_place_to_ask(calendars, sign_in, microsoft, google) -> Vec<WhereToAsk>, every calendar server once per sign-in on one server, then Microsoft, then Google"
  - "asking_when_free: questions_for_every_place(places, people) -> Vec<AskHere>, everybody at every place, Nowhere when there is no place, nothing when there is nobody"
  - "free_busy: one_answer_each and what_every_place_said, one Invited per address in the order first asked, answered anywhere wins, busy stretches united, coverings joined only where they meet"
  - "free_busy: the_same_server public"
affects: [13-31, 13-51]
tech-stack:
  added: []
  patterns: ["answers from several providers merged per person by normalised address before the address is dropped"]
key-files:
  created: []
  modified:
    - src/application/asking_when_free.rs
    - src/service/free_busy.rs
    - src/presentation/managers.rs
    - tests/item_form_free_busy.rs
    - guards/guards.toml
    - docs/privacy.md
    - docs/changelog.md
    - docs/ALPHA_TESTING.md
    - docs/plans/20260924-pro-licence.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "Decision 1 kept: a person answered by any place is answered, every place's busy time kept; unknown only where no place answered, with the first place's reason."
  - "Decision 2 kept: what Microsoft answers a personal account's getSchedule is ledgered (706), not guessed at."
  - "A calendar server is asked once per sign-in on one server, judged by scheme, host and port through free_busy::the_same_server."
  - "Two answers' coverings are joined only where they meet; where they do not, the longer is kept, so a gap nobody spoke about is never read as free."
metrics:
  duration: "about 1 hour 50 minutes"
  completed: 2026-09-28
actuals:
  tokens: 16500
  tasks: 2
  commits: 5
---

# Phase 13 Plan 30: Free/busy asks every source Summary

Find when everyone is free now asks every place an account keeps a calendar, each calendar
server it signs in to, Microsoft and Google, all at once, and builds one answer per guest
from everything those places said. A guest any place answered for is checked, with the busy
time from every place kept; a guest no place answered for is said once, with the first
place's reason, and never counted free.

## What was built

- **Every place** (`src/application/asking_when_free.rs`). `every_place_to_ask` returns every
  calendar server with a stored sign-in, one per sign-in on one server, then Microsoft where
  a token is held, then Google where the account keeps a Google calendar and a token is held.
  `questions_for_every_place` asks everybody at each place, asks nowhere when there is no
  place so everybody still comes back unknown, and asks nothing when there is nobody.
- **The merge** (`src/service/free_busy.rs`). `everybody_asked` hands every place's answers,
  each still beside its `AskAbout`, to `one_answer_each`, which groups them by the normalised
  address and folds each person's list through `what_every_place_said`. Two guests sharing a
  name stay two people; one guest spelled two ways is one.
- **The call site** (`src/presentation/managers.rs`). The worker builds `every_place_to_ask`
  with the same sign-ins and tokens it handed `where_to_ask`, then
  `questions_for_every_place(places, people)`. `where_to_ask` and `one_question` had no caller
  left and are gone.
- **The pages.** The privacy page says every place is asked the same question and how the
  answers are put together; the changelog under Changed, #57 point 2; the alpha page asks a
  tester with calendars in two places to say if a guest one of them knows is still not checked.

Test counts, taken 2026-09-28 on the branch: `application::asking_when_free::` 20 (was 16),
`service::free_busy::` 52 (was 44), `presentation::managers::` 137 (unchanged),
`tests/item_form_free_busy.rs` 7 (was 5).

GAP-08's second `[D]` line is marked with the date and the tests that hold it; the box waits
for 13-31.

## Commits and hook times

| Commit | What | Hook |
|---|---|---|
| `78af384d` | test: every place, everybody at each, one answer per person, red | red, 137 s |
| `4c2e1181` | feat: every_place_to_ask, questions_for_every_place, the merge; four records re-measured, two added | affected, 212 s |
| `9cc31cc8` | test: the event form asks every place, red | red, 83 s |
| `d9369a37` | feat: the worker asks every place; where_to_ask and one_question retired; one record renamed, one added | affected, 223 s |
| docs | the pages, the ledger, the changelog, this summary, the marks | this commit |

## Guard records

Three new, each measured with `--remeasure`: "busy time from every place a person was asked
about is united, not replaced" (two tests), "every place an account keeps a calendar at is
asked, not only the first" (two tests), and "find when everyone is free asks every place the
account has, not the first" (suite `item_form_free_busy`, the reading). The arrived-since count
went from 457 to 460.

13-29's records rewritten or renamed: "an account whose calendar is at google asks google" and
"google is never asked for an account with no google calendar" re-anchored onto
`every_place_to_ask`'s Google filter; the first came back short, its break also reddening
`test_every_place_the_account_keeps_a_calendar_is_asked`, and names it now. "find when everyone
is free hands the google token to every_place_to_ask" renamed from `where_to_ask` and
re-measured at seven tests. The two `free_busy.rs` records from 13-29 re-measured at 52 tests,
unchanged in what they redden. The six-record call ran in the background, about eleven
minutes.

## Deviations from Plan

**1. [Shape] `where_to_ask` stayed one commit longer.** Task 1 could not retire it while
`managers.rs` still called it, so it became the first of `every_place_to_ask` in task 1's green
and went, with `one_question`, in task 2's green.

**2. [Shape] "Each server once" is decided by sign-in and server.** Two calendars on one
server under one user name post to the same outbox, so the second is dropped. The comparison
is `free_busy::the_same_server`, made public for it. The plan did not say how a server is
known to be the same.

**3. [Shape] The merged covering.** The plan said "covering the window". Every source answers
for the window asked today, but a covering is a span, so two are joined where they meet and
otherwise the longer is kept; a covering stretched across a gap would read the gap as free.
Held by `test_what_two_places_cover_is_joined_only_where_the_two_meet`.

**4. [Shape] A guest typed twice with one address comes back once.** The merge is by address,
so a duplicate on the guest list is one person. It used to be two identical answers.

**5. [Rule 2] Two pages the plan did not list.** `docs/ALPHA_TESTING.md` gained a sentence for
a tester with calendars in two places, and `docs/plans/20260924-pro-licence.md`, whose
proposal cited `where_to_ask` by a command that now finds nothing, gained a dated line saying
so.

**6. [Brief] Task 2's test edits were set aside for task 1's green.** Written while the
re-measure ran, they would have made the hook's count check fail on the working tree. The
file was restored with `git checkout --` on that path, its diff kept in the scratchpad, and
the edits made again with Edit before task 2's red; the new diff compared equal to the kept
one. One diagnostic command carried a do-nothing shell assignment whose name contained
`sed`; nothing ran it. One hook log was first written beside the repository rather than in
the scratchpad and deleted at once.

## Ledger

Opened: 706 (todo, what Microsoft answers a personal account's getSchedule, decision 2, for
phase 14 to read and a later plan to word), 707 (unrun-verify, phase 14: an account with
calendars in two places asked for real). Closed: none. Counts 635 open, 72 fixed, 707 in all.

## Threat Flags

None beyond the register. T-13-30-01: busy time united, two tests and a record. T-13-30-02:
merged by address inside `everybody_asked`, a test. T-13-30-03 accepted and said on the
privacy page. T-13-30-SC: no crate added.

## Known Stubs

None.

## Self-Check: PASSED

The four code commits are on the branch; every modified file exists.

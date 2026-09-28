---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 29
subsystem: free/busy
tags: [google-calendar, free-busy, oauth, GAP-08, "#57"]
status: complete
requires: [13-28]
provides:
  - "free_busy: WhereToAsk::Google { base, token }, ask_google posting {base}/freeBusy through outward.asking_when_people_are_free, the pure what_google_said(reply, about), GOOGLE_CALENDAR_BASE"
  - "when_people_are_free: WhyNot::NotSharedWithYou, said \"their calendar is not shared with you\", fourth in EVERY_REASON_A_CALENDAR_IS_NOT_KNOWN"
  - "asking_when_free: where_to_ask(calendars, sign_in, microsoft, google) and holds_a_google_calendar(calendars)"
  - "oauth: a_google_token_for(account) -> Option<String>"
  - "calendar: GOOGLE made public"
affects: [13-30, 13-31, 13-51]
tech-stack:
  added: []
  patterns: ["a provider asked only where the account keeps a calendar there, checked before the token is fetched and again before it is used"]
key-files:
  created: []
  modified:
    - src/service/free_busy.rs
    - src/application/when_people_are_free.rs
    - src/application/asking_when_free.rs
    - src/application/calendar.rs
    - src/service/oauth.rs
    - src/presentation/managers.rs
    - tests/item_form_free_busy.rs
    - guards/guards.toml
    - docs/privacy.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "Decisions 1 to 3 kept: no new Google permission, notFound its own reason, one place per account in the order calendar server, Microsoft, Google."
  - "Any refusal in a calendar's errors wins over its busy list, because Google sends an empty busy list beside a refusal."
  - "A reply with no calendars object is refused as unreadable rather than read as naming nobody."
  - "The Google check runs twice, before the token is fetched (managers) and inside where_to_ask, so neither alone decides whether a guest list goes to Google."
metrics:
  duration: "about 2 hours 30 minutes"
  completed: 2026-09-28
actuals:
  tokens: 21000
  tasks: 3
  commits: 7
---

# Phase 13 Plan 29: Google as a free/busy source Summary

Find when everyone is free on an account whose calendar is at Google now asks Google's
`POST /freeBusy` with the token the account's Google sign-in already holds, fetched inside
the worker. Google's answer becomes the same busy stretches a calendar server's does, and a
guest Google cannot find is said as "their calendar is not shared with you" and never
counted free.

## What was built

- **The source and its parse** (`src/service/free_busy.rs`). `WhereToAsk::Google` posts
  `timeMin`, `timeMax` and `items[].id` with each address bare and lower case, bearer
  token, through the free/busy door. `what_google_said` reads `calendars.(id).busy[]` as
  Busy stretches over the window asked; any `errors[]` entry makes the diary unknown,
  `notFound` as `NotSharedWithYou` and any other reason as `TheServerWouldNotSay`; one
  stretch that is not an RFC 3339 instant makes that diary unreadable; a reply with no
  `calendars` object, or one that is not JSON, is refused. An address Google passed over
  stays unknown, as from every source.
- **The reason** (`src/application/when_people_are_free.rs`). "their calendar is not
  shared with you", said last among the four.
- **Where to ask** (`src/application/asking_when_free.rs`). A calendar server first, then
  Microsoft, then Google, and Google only where `holds_a_google_calendar` finds a calendar
  whose `source_provider` is `calendar::GOOGLE`.
- **The token and the call site.** `oauth::a_google_token_for` is one expression over
  `AuthManager::new(account, "gmail", ..).get_valid_token()`. The worker in
  `managers::asking_when_people_are_free` asks for it only for an account holding a Google
  calendar, inside `rt.spawn`, and hands it to `where_to_ask` with
  `free_busy::GOOGLE_CALENDAR_BASE`.
- **The waiting sentence.** "Asking where your calendar is kept about everybody on the guest
  list. This is experimental and can take a few seconds." See deviation 1.

Test counts, taken 2026-09-28 on the branch: `service::free_busy::` 44 (was 37),
`application::when_people_are_free::` 49 (was 47), `application::asking_when_free::` 16
(was 11), `presentation::managers::` 137 (unchanged), `service::oauth::` 56 (unchanged),
`tests/item_form_free_busy.rs` 5 (was 1).

GAP-08's first `[D]` line is marked with the date and the tests that hold it; the box waits
for 13-31.

## Commits and hook times

| Commit | What | Hook |
|---|---|---|
| `c0238941` | test: Google's answer and the not-shared reason, red (a first try refused by clippy at 21 s: the stub parse was unused) | red, 142 s |
| `3d655562` | feat: the parse, the sender, the reason; two records, CLOCK_FACES corrected | affected, 211 s |
| `4cbbc98c` | test: asking Google on a Google account, red | red, 144 s |
| `9050f5d3` | feat: where_to_ask, the token, the worker; three records | affected, 241 s |
| `1a746efc` | test: the waiting sentence, red | red, 119 s |
| `25e0e390` | fix: the waiting sentence; the managers record re-measured | affected, 439 s |
| docs | the pages, the ledger, the changelog, this summary, the marks | this commit |

## Guard records

Five new, each measured with `--remeasure`: "a guest google cannot find is not shared with
you and never free" (two tests), "google is asked when people are free with the account's
own sign-in", "an account whose calendar is at google asks google", "google is never asked
for an account with no google calendar", and "find when everyone is free hands the google
token to where_to_ask" (suite `item_form_free_busy`, re-measured again at five tests).
The arrived-since count went from 452 to 457. 1,254 records by the TOML reader.

"a moment written with a T is one the reader knows", flagged because
`when_people_are_free.rs` gained tests, came back short: its break reddened fourteen
`presentation::event_times` tests it did not name, a file 13-21.2 added and no record
listed. Corrected by hand and re-measured: 72 red, nothing else.

## Deviations from Plan

**1. [Rule 1/2] The waiting window's sentence changed.** `ASKING_THE_CALENDARS` in
`managers.rs` said "Asking your calendar server" on every account, untrue for Microsoft
and now Google, and nothing the person sees said the asking is experimental although no
source has answered for a real account. Red and green of their own, the test a reading in
`tests/item_form_free_busy.rs` with a companion, so `managers.rs` still holds 137 tests.

**2. [Rule 3] `calendar::GOOGLE` made public.** The plan named "gmail" as the source word
and put no public constant for it in reach; `calendar.rs` was not in `files_modified`. One
word and a doc line, no anchor moved.

**3. [Shape] Three records on `asking_when_free.rs` and `managers.rs` rather than two.** The
extra one holds T-13-29-02, a Google token used for an account with no Google calendar.

**4. [Shape] `holds_a_google_calendar` is public and tested.** The plan had the worker
check the calendars itself; the predicate is shared by the worker and `where_to_ask` so
the two cannot disagree.

**5. [Premise] Counts re-taken.** `managers.rs` records were 56 by `tests_last_seen` and
41 by `file`, not 54 and 39; no record anchors in the function edited.

**6. [Brief] Shell variables named after banned tools.** Three diagnostic commands carried
do-nothing assignments whose names contained `sed` or `awk`; neither tool ran and nothing
was written by them. Read-only Python parsed `guards.toml` and printed planning lines.
`cargo fmt` formatted the Rust files.

## Ledger

Opened: 704 (unrun-verify, phase 14: Google's freeBusy from a real account, a colleague and
a guest outside the domain), 705 (unrun-verify, the tester's ear: the not-shared sentence and
the waiting sentence). Closed: none. Counts 633 open, 72 fixed, 705 in all.

## Threat Flags

None beyond the register. T-13-29-01: a refusal wins over the busy list, two tests and a
record. T-13-29-02: checked before the token is fetched and in `where_to_ask`, a test and a
record. T-13-29-03: the token fetched inside `rt.spawn`, a reading with a companion and a
record. T-13-29-SC: no crate added.

## Known Stubs

None.

## Self-Check: PASSED

The six code commits are on the branch; every modified file exists.

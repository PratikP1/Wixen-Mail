---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 28
subsystem: directory lookup
tags: [microsoft-graph, people-search, oauth, tasks, GAP-07, "#55"]
status: complete
requires: [13-26, 13-27]
provides:
  - "oauth: People.Read on the outlook consent list; AuthManager::a_graph_token_carrying(scopes) under get_valid_graph_token, a_people_token and a_tasks_token; free functions a_people_token_for(account) -> Option<String> and a_tasks_token_for(account) -> Result<String>"
  - "microsoft_graph: MsGraphClient::people_matching(token, typed, at_most) and the pure the_people_microsoft_found(json) -> Result<Vec<Somebody>>"
  - "looking_people_up: Whose::Microsoft said \"from Microsoft\", everybody_found over three lists, microsoft_is_asked_for(account), SIGN_IN_AGAIN_FOR_PEOPLE_SEARCH"
  - "finding_people: who_matches opens the address book once and asks Microsoft through the_people_microsoft_knows"
affects: [13-29, 13-30, 13-51]
tech-stack:
  added: []
  patterns: ["a new provider permission asked in a token of its own, never added to a list older sign-ins refresh with"]
key-files:
  created:
    - tests/microsoft_people_join_the_people_found_list.rs
  modified:
    - src/service/oauth.rs
    - src/service/microsoft_graph.rs
    - src/application/looking_people_up.rs
    - src/presentation/finding_people.rs
    - src/presentation/wx_app.rs
    - guards/guards.toml
    - docs/privacy.md
    - docs/PROVIDER_SETUP.md
    - docs/USER_GUIDE.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "Token per new permission (decision 2) kept: Pratik's answer (c) confirmed the two permissions and did not choose the shared list."
  - "The people request selects displayName and scoredEmailAddresses only; personType is not asked for because nothing reads it."
  - "Microsoft's refusal is said in a sentence of our own, not Microsoft's words, which can echo the typed name; a failure to reach Microsoft drops the address, which carries it."
  - "The provider check is held by behaviour tests in finding_people rather than by a source reading."
metrics:
  duration: "about 3 hours"
  completed: 2026-09-28
actuals:
  tokens: 25500
  tasks: 4
  commits: 5
---

# Phase 13 Plan 28: Microsoft's people search in People found Summary

Typing a name on an Outlook or Office 365 account signed in through the browser now asks
Microsoft's `/me/people` with a token carrying `People.Read` alone, and the people it finds
join People found once per address, each row ending "from Microsoft"; the tasks sync asks
for a token carrying `Tasks.ReadWrite` alone, closing ledger 282.

## Pratik's answer (task 1)

Answer (c) of 2026-09-24, recorded in the phase README as decision 54 from his words "Yes to
all questions. Before you file an issue with wxDragon, give me more details.": Microsoft
Graph's People.Read and the tasks permission are confirmed. The executor's brief carried it,
so task 1 did not stop. His words did not choose the longer shared list, so the planner's
token-per-permission shape (decision 2) was built.

Microsoft's sentence behind that shape, from "Microsoft identity platform and OAuth 2.0
authorization code flow" as quoted in the plan's premise 2 (read by the planner on
2026-09-24): "The scopes requested in this leg must be equivalent to or a subset of the
scopes requested in the original `authorization_code` request leg." A reading of the
documentation, not a measurement: nothing here can ask Microsoft.

## What was built

- **The permission and the tokens** (`src/service/oauth.rs`). People.Read is on the consent
  list. `a_graph_token_carrying(scopes)` is the one refresh; `get_valid_graph_token` calls it
  with `THE_SCOPES_A_GRAPH_TOKEN_CARRIES`, unchanged at three entries, so no account's
  contacts, calendar, free/busy or notes change. `THE_PEOPLE_PERMISSION` and
  `THE_TASKS_PERMISSION` are each asked alone. The shared list's doc comment says where
  Tasks.ReadWrite is asked now.
- **The tasks sync** (`spawn_tasks_sync`) asks `a_tasks_token_for`, keeping the sign-in
  error it already reported.
- **The people call** (`src/service/microsoft_graph.rs`): `GET {base}/me/people?$search="<typed>"&$top=<n>&$select=displayName,scoredEmailAddresses`,
  the quoted text through `in_a_query` with any typed quote left out, the header
  `X-PeopleQuery-QuerySources: Mailbox,Directory`, no retry. The parse keeps the first
  scored address with text, drops a person with none, ignores unknown fields and refuses a
  malformed answer with a sentence.
- **The list** (`src/application/looking_people_up.rs`): `Whose::Microsoft` said "from
  Microsoft"; `everybody_found(contacts, directory, microsoft)` keeps each address once,
  compared without case, the contact first, then the directory, then Microsoft.
- **The ask** (`src/presentation/finding_people.rs`): `who_matches` opens the address book
  once, for the contacts and the account. `the_people_microsoft_knows` asks only where
  `microsoft_is_asked_for` says the account uses OAuth and its provider is `outlook`; a
  missing people token adds "Sign in again from the Account Manager to let Microsoft find
  people for this account." to the one trouble line; a failed call adds its sentence. The
  typed name is never logged.

Test counts, taken 2026-09-28 on the branch: `service::oauth::` 56 (was 51),
`service::microsoft_graph::` 65 (was 59, not the plan's 57), `application::looking_people_up::`
43 (was 37), `presentation::finding_people::` 7 (was 5), the new target 6.

## Commits and hook times

| Commit | What | Hook |
|---|---|---|
| `da086c71` | test: a token per new Microsoft permission, red | red, 179 s |
| `068e4969` | feat: People.Read, the people and tasks tokens, two records | affected, 486 s |
| `55d27abf` | test: people search, red (a first try refused by clippy at 57 s) | red, 156 s |
| `df2e7b90` | feat: people search joins People found, three records | affected, 269 s |
| docs | the pages, the ledger, the changelog, this summary, the marks | this commit |

## Guard records

Five new, measured: "the shared graph token asks for nothing an older sign-in never
granted", "the tasks sync asks for the token carrying the tasks permission" (suite, the new
target), "a people search names both places microsoft looks", "a person microsoft also
found keeps the row already in the list" (two tests), "microsoft is asked about a typed name
only for a microsoft browser sign-in". Re-measured: the three `oauth.rs` records at 56, the
three `microsoft_graph.rs` records at 65, "a name written back out is quoted when it needs
to be" (20 tests red, unchanged). The arrived-since count went from 447 to 452. 1,249
records by the TOML reader.

## Deviations from Plan

**1. [Rule 1] The shared-list test first compared arrays, so its break did not compile.**
The record adding People.Read makes the constant `[&str; 4]`, and `assert_eq!` against a
three-entry array is a type error, so the first re-measure reported the record wrong. The
test compares slices now; re-measured alone after the five-record call.

**2. [Shape] The provider check is held by behaviour, not a reading.** The plan asked for a
reading that `who_matches` asks Microsoft behind the check. `the_people_microsoft_knows`
takes the account, so two unit tests call it: every other account comes back silent, and an
outlook browser sign-in with no token is told to sign in again, which is also the proof the
path is reachable. The guard record's red is that lib test, with no suite. The target holds
two readings instead: the people call is asked only with `a_people_token_for`, and
`who_matches` hands Microsoft's people to the fold.

**3. [Shape] No `personType` in `$select`.** Nothing reads it, and least data was the
threat register's line.

**4. [Rule 2] Microsoft's own error text is not carried.** A refusal becomes our sentence
with the status and, for 401 and 403, both remedies; a network failure's address is
dropped with `without_url()`, because the address holds the typed name.

**5. [Shape] `a_tasks_token_for` returns `Result`, `a_people_token_for` `Option`.** The tasks
sync already reported why a sign-in failed and keeps doing so; people search has one
sentence for every missing token.

**6. [Premise] Ledger 645 was not measured.** The plan named 13-28 as where a real token's
length is measured. No real account is reachable here, and the new tokens are used at once
and never stored, so the stored sign-in is still the only one that meets the limit. Noted in
645's both halves and in ledger 702.

**7. [Brief] Two shell variables named after a banned tool.** Two diagnostic commands
carried do-nothing assignments whose names contained the word `sed`; nothing ran it and
nothing was written by them. Read-only Python printed long `STATE.md` lines, parsed
`guards.toml` and checked the ledger's halves agree. `cargo fmt` formatted the Rust files.

### Found and left

- Ledger 703, a design question for Pratik: an account signed in before tasks synced both
  ways is expected to be refused the tasks token and sync no Microsoft tasks until it signs
  in again. Recommendation: keep it; the alternative is a read-only fallback to the shared
  token.
- `docs/PROVIDER_SETUP.md` and `docs/privacy.md` still say nothing uses Notes.ReadWrite,
  while `notes_backend` names OneNote as the backend for an outlook account since 5.2. Not
  this plan's subject; not ledgered here because it was not verified whether the OneNote
  sync reaches a real notebook.

## Ledger

Closed: 282, both halves. Opened: 700 (todo, the People API's maintenance mode and the move
to `/search/query`), 701 (unrun-verify, the tester's ear), 702 (unrun-verify, phase 14:
consent, a real tenant, a task change, an older sign-in's tasks token), 703 (todo, the design
question above). 645 annotated. Counts 631 open, 72 fixed, 703 in all.

## Threat Flags

None beyond the register. T-13-28-01: `microsoft_is_asked_for`, two behaviour tests and a
record. T-13-28-02: the shared list unchanged, a test comparing it exactly and a record that
adds People.Read to it. T-13-28-03: serde parse, unknown fields ignored, an address required.
T-13-28-04: People.Read only. T-13-28-SC: no crate added.

## Known Stubs

None.

## Self-Check: PASSED

The four code commits are on the branch; the created files exist.

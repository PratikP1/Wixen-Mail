---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 27
subsystem: directory lookup
tags: [directory, ldap, credential-store, account-manager, GAP-07, "#55"]
status: complete
requires: [13-26]
provides:
  - "application::directory_sign_in::what_the_window_keeps(address, look_in, sign_in_as, typed_password, a_password_is_saved) -> Result<WhatIsKept { directory, password: PasswordChange }>, with A_PASSWORD_IS_SAVED and NOT_TRIED_YET"
  - "wx_account_manager::build_directory_sign_in_dialog and wire_the_directory_sign_in, Look People Up at Work on the Account Manager's Alt+L"
  - "service::directory::WHERE_A_DIRECTORY_IS_SET_UP and no_password_is_saved_for(named, sign_in_as), one sentence for the lookup and the window"
  - "ScanTarget::DirectorySignIn, directory-sign-in, on the Accessibility workflow's list"
affects: [13-28, 13-33, 13-51]
tech-stack:
  added: []
  patterns: ["a window's save decided by one pure function returning the refusal as a sentence, the window writing only what it answers"]
key-files:
  created:
    - src/application/directory_sign_in.rs
    - tests/the_directory_sign_in_has_a_window_of_its_own.rs
  modified:
    - src/application/mod.rs
    - src/service/directory.rs
    - src/presentation/wx_account_manager.rs
    - src/presentation/scan_target.rs
    - src/presentation/wx_app.rs
    - .github/workflows/accessibility.yml
    - tests/account_edit_protocol_fields.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/ALPHA_TESTING.md
    - docs/privacy.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "The window's OK has an id of its own rather than ID_OK, so a refused save keeps the window open rather than wxWidgets closing it."
  - "Settings are written before the password: a store that refuses leaves the address and the name kept, and the window says the password was not."
  - "The builder takes only whether a password is saved, never the password, so nothing but the credential store and the box ever holds one."
  - "The window is not resizable: a resize border put a nameless size grip into the MSAA tree."
  - "Five lookup sentences that said \"the account's settings\" now name Look People Up at Work on the Account Manager, since the boxes left the account editor."
metrics:
  duration: "about 75 minutes to the documents commit"
  completed: 2026-09-28
actuals:
  tokens: 20000
  tasks: 3
  commits: 5
---

# Phase 13 Plan 27: Look People Up at Work, the directory sign-in in a window of its own

The Account Manager has **Look People Up at Work** on `Alt+L`, beside Edit. It opens a window
for the chosen account holding the directory address (`Alt+D`), where in it to look (`Alt+W`),
the sign-in name (`Alt+N`) and the password (`Alt+P`); with none chosen it says what Edit says.
OK writes the address, the place and the name to the settings and the password to the Windows
credential store through `service::directory`, decided in one pure function. The password box
opens empty; when a password is saved, its description and a line under it say so. A sign-in
name with an empty box and nothing saved is refused with the lookup's own sentence, the window
stays open and focus goes to the password box. The two directory boxes left the account
editor's second page, so Y and H are free there again. Ledger 693 is closed: the sign-in name
and the password now have a writer outside tests.

## What works, and how it is known

- **The save decision.** `application::directory_sign_in::tests`, 11 rows: both places empty
  removes the directory and forgets the password; a name with a typed password replaces it; a
  name with an empty box keeps the saved one; a name with nothing saved is refused with
  `directory::no_password_is_saved_for`; no name forgets; values trimmed, the password kept as
  typed; a box of spaces is empty; one place filled is still a directory; the refusal names
  the address as typed when it is not one; a replaced password never reaches `Debug`; the two
  sentences.
- **The window, read in a built process.** `tests/the_directory_sign_in_has_a_window_of_its_own.rs`,
  13 readings on a desktop of the process's own with the profile pointed at a temporary
  directory: the Account Manager's `&Look People Up at Work...` holds L alone, with a
  companion planting a second L; the window's letters are D, N, P and W once each, with a
  companion planting a second D; focus opens on the directory address, named on MSAA at its
  own handle; the four fields and two buttons named in tab order; the boxes open on the stored
  directory with the password box empty and `ES_PASSWORD`; with a password saved the box's MSAA
  description and a shown line are `A_PASSWORD_IS_SAVED`, and with none neither is; the
  untried line comes first; a refused save pressed with `BM_CLICK` leaves the window shown,
  the sentence on its line and focus in the password box; the save calls
  `directory::keep_the_password` and `forget_the_password` and `Directory` has no field for a
  password, with a companion planting both faults.
- **Page two.** `tests/account_edit_protocol_fields.rs` rewritten in place: no showing label
  on page two reads `Director&y address:` or `W&here in it to look:`, and every letter on each
  page is still one control's.
- **The sentences.** `service::directory::where_the_sentences_send_somebody`, one test over
  the five sentences that send somebody to set a directory up: each names
  `WHERE_A_DIRECTORY_IS_SET_UP` and none says "account's settings".
- **Counts, re-taken 2026-09-28:** `application::directory_sign_in::` 11 (the plan asked at
  least 7); the new target 13 (at least 8); `data::config::` 68 with
  `test_a_setting_said_to_be_offered_elsewhere_really_is` green; `presentation::scan_target::`
  11; `service::directory::` 54 (53 before); `presentation::wx_account_manager::` 14;
  `tests/account_edit_protocol_fields.rs` 1; `ScanTarget::ALL` 43 (42 before).
- **Reachability.** Tools, Account Manager (`Ctrl+Shift+A`) -> `build_account_manager_dialog`'s
  `look_people_up` button -> `wire_account_manager_actions` -> `look_people_up_for` ->
  `show_the_directory_sign_in` (reads `directory::the_saved_password` once, for whether one is
  saved) -> `build_directory_sign_in_dialog` -> `wire_the_directory_sign_in` -> OK ->
  `keep_what_the_directory_window_holds` -> `what_the_window_keeps` ->
  `remember_where_to_look_people_up` and `directory::keep_the_password` or
  `forget_the_password`. The next lookup reads both through `finding_people::the_organisation`.
- **Acceptance greps:** `&Look People Up at Work...` 1 in `wx_account_manager.rs`;
  `Director&y address` 0 outside comments; `directory-sign-in` 1 in the workflow.
- **Not proved:** a real directory with a sign-in saved here (ledger 698, beside 695), and the
  window heard (ledger 697). The Accessibility scan of the new target is read on the pull
  request.

## Commits

| Commit | What | Hook |
|---|---|---|
| `ecd95b6c` | red: 11 rows, the sentence test, the count check | red, 163 s |
| `2d45a182` | feat: `what_the_window_keeps`, the five sentences; 1 record new, 2 re-measured | affected, 198 s |
| `6b2fde9f` | red: 13 readings on a bare stub, page two's rewritten case | red, 160 s |
| `13f3494a` | feat: the window, the button, page two, the scan target, the shortcuts page; 2 records new, 1 re-measured | all (the workflow line), 782 s |

The documents commit follows, then the pull request, then the merge.

## Guard records

| Record | Break | Red |
|---|---|---|
| clearing the directory sign-in name forgets the saved password (new) | the no-name arm keeps | `test_no_sign_in_name_forgets_any_saved_password` |
| a password typed into the directory window goes to the credential store (new, suite the new target) | the Replace arm made `Ok(())` | the source reading |
| the directory window's password box says when a password is saved (new, suite the new target) | `if a_password_is_saved` made `if false` | the saved-password reading |

Re-measured: the two records on `directory.rs` (53 to 54 tests) and "a page of the account
editor gives each letter to one control" for its rewritten case. Two `--remeasure` calls, one
per green commit: 3 records in 375 s and 3 in 61 s, every one reddening exactly what it names.
Anchors inside the files this plan edits were read before editing: the eight
`wx_account_manager.rs` records anchor on lines no edit touched, the `scan_target.rs` record
on the `SignatureEditor` name line, the workflow records on the `which-days` and
`contact-editor` lines, and the `directory.rs` encryption record on the `ldap://` arm, which
the refactor of the arm above it left alone. The arrived-since count went from 444 to 447.

## Documents

- Shortcuts page: the Account Manager's `Alt+L`, the window's four letters, page two without
  `Alt+Y` and `Alt+H`, in the commit that changed them.
- User guide: "### Looking people up" under Composing Email, the People found list and the
  directory set-up in five steps, where the password lives, the `ldaps://` rule, the saved
  password's three answers, the refusal, experimental; Managing accounts names the button.
- Alpha page, privacy page (the row and two paragraphs), changelog Added and Changed.

## Ledger

Closed, both halves: 693. Opened, both halves: 697 (`unrun-verify`, the tester's ear), 698
(`unrun-verify`, phase 14: a sign-in saved here reaching a real directory and leaving
Credential Manager when cleared), 699 (`todo`, a design question: refuse a password for an
`ldap://` address at OK rather than at every lookup). Counts 699 total, 628 open, 71 fixed,
0 waived.

## Deviations from Plan

**1. [Rule 1] Five lookup sentences named the page the boxes left.** `service::directory`
said "Add one in the account's settings" and four more like it, which meant the account
editor's second page. Moving the boxes made each one send somebody to a page with nothing to
change. Each now names `WHERE_A_DIRECTORY_IS_SET_UP`, and `no_password_is_saved_for` is the
one sentence for the lookup and the window's refusal, as the plan's "the sentence 13-26's
lookup says" asked. `directory.rs` was not in `files_modified`; its test was red first.

**2. [Shape] The guard record "writes the password into the settings" is the store write
going missing.** `Directory` has no field a password could go in, which the reading now
holds, so the only way a typed password goes astray is the save not reaching the store. The
record breaks that, with the new target as its suite.

**3. [Expectations] The red's names lacked the comma.** `name_from_label` turns a label's
colon into a trailing comma, the pause before the role, and every labelled field here is
named that way. The red expected "Directory address"; the green expects "Directory address,"
and the password box is named through the same function. The red was red for its real reason,
empty names. The refusal line wraps, so it is read with its breaks as spaces.

**4. [Rule 2] The window is not resizable.** With a resize border the MSAA tree held a
nameless `ScrollBar`, the size grip, after Cancel. The window fits itself to what it holds.

**5. [Rule 3] `wx_app.rs` gained the scan target's arm,** which the exhaustive match needs; it
opens the window on the scan-only account with a made-up directory and a password said to be
saved, and wires no OK.

**6. [Rule 2] A store that cannot be read is said, not only shown.** The account manager's
own test holds every answer on that screen to `said_and_shown`, so the sentence is said
through `call_after` once the window is up.

**7. [Docs] The user guide said `Ctrl+A` for the Account Manager** twice; it has been
`Ctrl+Shift+A` since `Ctrl+A` became Select All. Both lines in Account Setup are corrected.

**8. [TDD] The scan target and its workflow line landed in the green** without a red of their
own; `test_the_workflow_asks_for_every_target` and the names list would have been red for
it. It is a list entry and a match arm.

**9. [Brief] Read-only Python** printed long lines of the planning files and parsed
`guards.toml` and the ledger's JSON; nothing was written by it. `cargo fmt` formatted the
Rust files.

### Found and left

- Ledger 699: a password for an `ldap://` address is kept and refused at each lookup; the
  recommendation is to refuse it at OK.

## Threat Flags

None beyond the register. T-13-27-01: written only through `directory::keep_the_password`,
`Directory` has no password field, a reading and a record. T-13-27-02: the builder never
receives a saved password, the box opens empty, `ES_PASSWORD` read. T-13-27-03: refused
before anything is written, a row and a reading. T-13-27-SC: no crate added.

## Known Stubs

None.

## Self-Check: PASSED

The four commits above are on the branch; the created files exist.

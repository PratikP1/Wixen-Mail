---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 46
subsystem: mail export
status: complete
tags: [export, eml, file-menu, GAP-13, issue-53]
requires:
  - 13-45 (the folder question, the shared store reading, the File item on F)
provides:
  - service::mailbox_archive::message_files_written_under, MessageFilesBeingWritten, WhereItWent
  - application::importing_messages::a_subject_as_a_file_stem (Save As now calls it)
  - export_tree::a_message_file_named, numbered_names, one_message_written_out_and_counted, what_the_message_files_export_did, message_files_that_broke_off
  - application::exporting_mail::one_folder_as_message_files
  - File, Export Folder as Message Files (ID_EXPORT_A_FOLDER_AS_MESSAGE_FILES) and a_folder_written_out_on_a_worker
affects:
  - src/presentation/wx_app.rs (13-45's handler now hands its worker to the shared function)
  - src/application/exporting_mail.rs (the store reading is one struct both loops use)
tech-stack:
  added: []
  patterns:
    - every message file opened with create_new, the next offered name taken when one is there
    - one worker for both exports of one folder, the writer passed in as a closure
key-files:
  created:
    - .planning/phases/13-new-features-most-from-the-outlook-gap-audit/13-46-SUMMARY.md
  modified:
    - src/service/mailbox_archive.rs
    - src/application/export_tree.rs
    - src/application/exporting_mail.rs
    - src/application/importing_messages.rs
    - src/presentation/wx_app.rs
    - tests/mail_goes_out_in_every_shape.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - The name is cleaned twice, the subject and then the day with it, so a long subject is shortened behind the day rather than the day lost
  - The count of files and signatures left behind comes from export_tree::one_message_written_out_and_counted, one walk over the files, rather than a second count beside one_message_written_out
  - A partial message file left by a failed write is taken away; what went out before a failure stays, and the sentence says how many
  - The writer refuses a name that is not one plain step down from the folder, though the names reaching it are already cleaned
metrics:
  duration: about 2 hours 40 minutes
  completed: 2026-10-03
actuals:
  tokens: 30000
  tasks: 3
  commits: 5
---

# Phase 13 Plan 46: One folder out as loose message files Summary

File, Export Folder as Message Files (`Alt+F`, then `X`) writes the folder the cursor is on, and
every folder inside it, as one saved message per file into a folder the person chooses, named
`2026-09-24 Hello.eml`, never over a file already there, and this program's own folder import
reads it back as the same folders and messages.

## What works now

- **The item.** File holds "E&xport Folder as Message Files..." after Export Folder as a
  Mailbox File, no shortcut key, help string "Write this folder's mail as one saved message per
  file, with a folder for each folder inside it". Letters on File re-taken on the day from the
  builder with Read: N (the prepended New submenu), S, A, M, D, I, O, E, F, K, P, Q; X was free
  and the item took it.
- **The handler.** `export_a_folder_as_message_files` asks `a_folder_to_write_out`, opens a
  `DirDialog` titled "Choose the folder the message files go into", refuses "No folder was
  chosen." through `send_refusal`, says "Writing this folder's mail as message files." on the
  status line and at normal priority, and hands `a_folder_written_out_on_a_worker` the writer.
  That worker is now the one 13-45's handler uses too.
- **The names.** `a_message_file_named` is the day the message's own RFC 3339 date gives, in
  its own offset (`chrono::DateTime::parse_from_rfc3339`, the parser `message_files` and
  `mail_across_accounts` already use), then the subject through
  `importing_messages::a_subject_as_a_file_stem`, the function extracted from `saving_as`.
  No readable date, the subject alone. `one_nothing_else_has_taken`, now `pub(crate)`, tells
  names apart in one folder ignoring case; `numbered_names` offers `stem.eml`, `stem (2).eml`
  and on.
- **The writer.** `MessageFilesBeingWritten::a_message_file` opens each offered name with
  `create_new`, passes over one already there, writes, syncs the data, and answers the name used
  and whether it was the first. A failed write takes away the file it made. A root that is not a
  folder is refused at the start; a name that is not one plain step down is refused.
- **The loop and the sentence.** `one_folder_as_message_files` lays the folders out with
  `where_each_folder_goes`, makes each one even when nothing goes into it, counts left-out
  messages, missing files and signatures in the shared words, and adds "N messages were saved
  under a numbered name, because a file with that name was already in the folder".

What a person has not done: opened these files in another mail program, or heard the item and
its sentences (ledger 792).

## The deep path

A message twelve folders down with a 120-character name, past 260 characters in all, was written
and read back by `std::fs` as it is. The writer does not use the verbatim form; nothing needed it
(RESEARCH-5's A4, settled by
`service::mailbox_archive::tests::test_a_message_file_deep_past_the_old_path_limit_is_written_and_read_back`).

## Commits

| Commit | What | Hook |
|---|---|---|
| `4177e4f7` | test: failing cases for the writer, the names, the sentence and the loop | red, 169 s |
| `5c62752c` | feat: the writer, the names, the loop, the shared stem | 285 s |
| `c75f84f9` | test: failing readings for the item and its handler | red, 109 s |
| `339f6ccb` | feat: the item, the handler, the shared worker, the shortcuts row, the changelog | 413 s |
| docs | the guide, the ledger, this summary, the four marks | this commit |

## Counts

Taken 2026-10-03 with `cargo test --lib <path>` and `cargo test --test <target>`:

| Target | Before (13-45's summary) | After |
|---|---|---|
| `service::mailbox_archive::` | 36 | 41 |
| `application::export_tree::` | 43 | 55 |
| `application::exporting_mail::` | 4 | 6 |
| `application::importing_messages::` | 36 | 36 |
| `mail_goes_out_in_every_shape` | 4 | 6 |
| `wired` | 77 | 77 |

`grep -cF "=> '_'," src/application/importing_messages.rs` is 1, and
`test_every_guard_record_still_names_one_place_in_the_tree` passes.

## Guard records

Four new; the arrived-since count 613 to 617; 1,414 records by the TOML reader.

| Record | Break | Red |
|---|---|---|
| a message file is never written over a file already in the folder | `create_new` replaced by `File::create` | 2: the writer's case and the loop's |
| names in one folder are told apart ignoring case | `named.to_lowercase()` made `named.clone()` | 3: the case-only names and two folder-layout cases |
| a message file is named by its day and then its subject | the loop names by subject alone | 2: the round trip and the never-written-over case |
| Export Folder as Message Files writes where the folder picker said | the picker's answer replaced by `"."`, suite `mail_goes_out_in_every_shape` | 1: the reaches-its-writer reading |

Remeasured, one call per commit that flagged them:

- `5c62752c`: seven records, `--log` in the background. Four held: the two new loop records,
  "a name written back out is quoted when it needs to be" and "a mailbox file nothing went into
  leaves the name chosen as it was". Three did not, and were corrected by hand and measured
  again: the case record also reddens two folder-layout cases in `export_tree`; "Save As takes
  the path out of a subject before offering it as a name" now also reddens
  `test_a_message_file_whose_subject_holds_a_slash_keeps_one_name`, since the export shares the
  stem; and "a mailbox file of one folder counts only the folders inside it as left out" named
  two places, because the new loop spells `let inside = format!("{folder}/");` too, so its
  anchor gained the line above. The three then held, 341 s.
- `339f6ccb`: the new picker record and "Export Folder as a Mailbox File asks whether what is
  chosen is a folder", each exactly as named, 40 s.

## Ledger

Opened 792 (`unrun-verify`, the tester: the files in Thunderbird and his own mail program, the
round trip on his machine, the item and its sentences heard). 792 in all, 688 open, 104 fixed,
0 waived, both halves written. Nothing touches a server, so nothing went to phase 14.

## Version and pull request

No version bump: no build has been cut since 1.0.0-alpha.1. Nothing added to `Cargo.toml` or
`Cargo.lock` (T-13-46-SC). The plan changes what is shown and spoken, so the branch is pushed
with a pull request; its number and the three runs are in the merge report.

## Deviations from Plan

**1. [Rule 2] The writer refuses a name that climbs out of the folder chosen, with a case.** The
names reaching it are cleaned already; this is the last place a mistake there could still write
elsewhere (T-13-46-01). `test_a_name_that_climbs_out_of_the_folder_chosen_is_refused`.

**2. [Shape] The loop counts through `export_tree::one_message_written_out_and_counted`, not
`one_message_written_out` alone.** The plan's step gives bytes and no count of the files left
behind or the signatures not kept; a second count beside it would be a second walk over the
files. `one_message_written_out` now answers through it, so Save As writes what it wrote before.

**3. [Shape] A broke-off sentence of its own, `export_tree::message_files_that_broke_off`.** The
plan named none. Unlike the mailbox file, what went out before a failure stays, so the sentence
says how many message files are in the folder chosen. It is in the module's every-sentence check
and has its own case.

**4. [Shape] One worker for both exports of one folder.** 13-45's handler carried the store
opening, the progress lines and the failure lines inline; the new handler would have copied
them, so both hand `a_folder_written_out_on_a_worker` their writer. No record anchored in those
lines.

**5. [Shape] The store reading is a struct, `WhatTheStoreHolds`,** which both loops read, rather
than 13-45's three reads inside `built_from_the_store`.

**6. [Anchor] "a mailbox file of one folder counts only the folders inside it as left out" was
lengthened by one line** (see Guard records).

**7. [Order] The shortcuts row and the changelog entry went in with the item in `339f6ccb`,** as
13-45 did, because CLAUDE.md puts both in the commit that makes the change; the guide, the ledger
and this summary are in the documents commit.

**8. [Brief] No commit on `main` after the merge to add a mode line to this summary,** which the
plan's verification asks; the brief allows one documents commit before the merge and no other,
and 13-45 made none.

**9. [Test] The case-only case asserted inequality at first and passed against the stub;** it
was rewritten to exact names before the red commit, and went red.

## TDD Gate Compliance

Task 1: `4177e4f7` (test, red: five writer cases, thirteen export_tree cases, two loop cases and
the count check, accepted by `red-commit.sh`) then `5c62752c` (feat). Task 2: `c75f84f9` (test,
red: two readings, the census and the count check, accepted) then `339f6ccb` (feat). Task 3 is
documentation.

## Known Stubs

None. The item reaches the handler, the handler the worker and the loop, the loop the writer,
the names and the sentence; Save As reaches the shared stem.

## Threat Flags

None beyond the register. T-13-46-01: the separator rule and `safe_file_name` through one stem
function, cases for `CON`, `/` and a 200-character subject, and the writer's own refusal.
T-13-46-02: `create_new` on every file, no check-then-write, a guard record. T-13-46-03: the
shared sentences and the numbered-name count. T-13-46-SC: nothing added to `Cargo.toml`.

## Self-Check: PASSED

The four code commits are in `git log --oneline main..HEAD`; the four marks are made in the
documents commit.

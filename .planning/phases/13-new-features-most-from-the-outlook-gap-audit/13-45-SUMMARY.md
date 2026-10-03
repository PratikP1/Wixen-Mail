---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 45
subsystem: mail export
status: complete
tags: [export, mbox, file-menu, GAP-13, issue-53]
requires:
  - 13-03 (File, Print on P)
  - 13-17 (File, PGP Keys on K)
  - 13-44.9
provides:
  - service::mailbox_archive::one_mailbox_file_written_to, MailboxFileBeingWritten, MailboxFileOutcome
  - application::exporting_mail (one_stored_message_added, InTheFile, one_folder_as_a_mailbox_file)
  - export_tree::what_the_mailbox_file_export_did and a_mailbox_file_that_broke_off
  - message_files::between_two_messages
  - File, Export Folder as a Mailbox File (ID_EXPORT_A_FOLDER_AS_A_MAILBOX_FILE) and a_folder_to_write_out
affects:
  - src/presentation/wx_app.rs (Export Mailbox's loop and refusals)
  - src/application/importing_messages.rs (seams removed)
  - tests/mail_taken_off_leaves_no_words_behind.rs (census fix)
tech-stack:
  added: []
  patterns:
    - a file written under the chosen name plus .partial with create_new, renamed over the name only when something went in
    - one folder question every export handler calls, held by a census with a planted companion
key-files:
  created:
    - src/application/exporting_mail.rs
    - tests/mail_goes_out_in_every_shape.rs
    - .planning/phases/13-new-features-most-from-the-outlook-gap-audit/13-45-SUMMARY.md
  modified:
    - src/service/mailbox_archive.rs
    - src/application/export_tree.rs
    - src/application/importing_messages.rs
    - src/application/message_files.rs
    - src/application/mod.rs
    - src/presentation/wx_app.rs
    - tests/wired.rs
    - tests/mail_taken_off_leaves_no_words_behind.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - The shared step takes where the message lands (InTheFile), because a message after another needs the empty line the writer adds only when its buffer already holds one, and both exports build each message in a fresh buffer
  - The broke-off sentence for the bare file is its own, in export_tree, because the archive's says the file is unfinished and the bare file leaves none
  - The folders-inside count is what the separator guard holds, since the folder's mail is read by its exact path and the filter only counts
metrics:
  duration: about 6 hours
  completed: 2026-10-03
actuals:
  tokens: 90000
  tasks: 4
  commits: 6
---

# Phase 13 Plan 45: One folder out as a bare mailbox file Summary

File, Export Folder as a Mailbox File (`Alt+F`, then `F`) writes the folder the cursor is on,
without the folders inside it, into one mailbox file put at the chosen name only when a message
went in; the round trip that proves it found Export Mailbox had been writing every folder so
that it read back as its first message since 2026-08-29, and that is fixed too.

## What works now

- **The item.** File holds "Export Folder as a Mailbox &File..." directly after Export Mailbox,
  no shortcut key. Letters on File re-taken on the day from the builder: N (the prepended New
  submenu), S, A, M, D, I, O, E, K, P, Q, so F was free and the item took it, as the plan said.
  13-46's X is untouched.
- **The handler.** `export_a_folder_as_a_mailbox_file` asks `a_folder_to_write_out`, offers the
  folder's last part made safe through `export_tree::where_each_folder_goes` with `.mbox`, and
  hands a worker `exporting_mail::one_folder_as_a_mailbox_file`, which says "N messages written
  out so far." every hundred and the closing sentence on the status line.
- **The writer.** `MailboxFileBeingWritten` writes `<name>.partial` with `create_new` and
  renames it over the name at `finish` only when something went in; `abandon` and a finish
  nothing went into take it away. A file already called `<name>.partial` is refused by name.
- **The sentence.** `what_the_mailbox_file_export_did` shares the left-out, files-not-here and
  signatures-not-kept sentences with `what_the_folder_export_did` through one private
  function, `what_was_left_behind`, and adds the folders not included and Export Mailbox's name.
- **One folder question.** Export Mailbox's three refusals moved into `a_folder_to_write_out`
  with their sentences unchanged; both export handlers call it.
- **Export Mailbox's folders read back whole again** (ledger 790), through the same step.
- **Removed** (decision 6): `writing_out`, `CHOOSE_THE_MESSAGES_TO_EXPORT`,
  `what_the_mail_export_did`, `WritingOut::Refused`. `the_file_ends_with` answers a plain
  `&'static str`.

What a person has not done: opened a file this writes in another mail program, or heard the
item and its sentences (ledger 789).

## Commits

| Commit | What | Hook |
|---|---|---|
| `e8f3d404` | test: failing cases for the writer, the loop and the sentence | red, 216 s |
| `51fbc763` | feat: the writer, the loop, the sentence, the shared step, the census fix | 402 s (a first try refused at 445 s, deviation 2) |
| `24ab8f8f` | refactor: the export seams nothing reached removed | 287 s |
| `bdd551f6` | test: failing readings for the item and the folder question | red, 99 s |
| `23a5ad18` | feat: the item, the helper, the handler, the arm, the page row, the changelog | 381 s (a first try refused at 403 s, deviation 4) |
| docs | the guide, the ledger, this summary, the four marks | this commit |

## Counts

Taken 2026-10-03 with `cargo test --lib <path>` and `cargo test --test <target>`:

| Target | Before | After |
|---|---|---|
| `service::mailbox_archive::` (lib) | 31 | 36 |
| `application::exporting_mail::` | new | 4 |
| `application::export_tree::` | 38 | 43 |
| `application::importing_messages::` (module run) | 41 by the count check | 36 |
| `mail_goes_out_in_every_shape` | new | 4 |
| `wired` | 77 | 77 |
| `mail_taken_off_leaves_no_words_behind` | 3 | 4 |

The plan's acceptance asked at least 36, 3 and 40 for the first three; all hold.

## Tests removed and rewritten (task 2)

Removed, each about a removed item only, all in `application::importing_messages::tests`:
`test_exporting_with_nothing_chosen_says_so_rather_than_writing_an_empty_file`,
`test_one_message_is_written_as_a_single_message_and_several_as_an_archive`,
`test_an_export_opens_with_the_same_count_the_writer_says`,
`test_messages_whose_text_was_never_downloaded_are_said_rather_than_written_out_empty`,
`test_one_message_left_out_of_an_export_is_said_in_the_singular`.

Rewritten in place: `test_each_kind_of_export_names_the_ending_the_file_it_writes_should_have`
(through `the_file_ends_with` alone); `importing_messages`'
`test_everything_this_module_says_is_a_sentence_and_names_no_machinery` (three removed items
dropped); `export_tree`'s
`test_messages_left_out_are_said_in_the_words_the_one_folder_export_already_uses` (the folder
export against the mailbox file export); `test_a_folders_mail_is_named_the_way_an_archive_written_on_its_own_is`
(its `expect` gone with the `Option`).

`dead-code-hunter` over the two files, 2026-10-02, `grep -rn --include='*.rs' -F` per name over
`src tests`, with `saving_as(` as the probe that the search sees (10 hits): `writing_out` 8 hits,
the definition and 7 test calls; `CHOOSE_THE_MESSAGES_TO_EXPORT` 4, the definition, `writing_out`
and two tests; `what_the_mail_export_did` 8, the definition and test calls only. No macro
generates any of them. All three reached only by tests: a capability nothing reaches, deleted
by decision 6 rather than wired, since the new items cover it.

The header at `export_tree.rs`'s "What cannot come back" now reads: "The files a message came
with, where this computer never kept them. The files a message carries are kept when it is
read, and those go with it. One this computer does not have is described by name, type and
size and nothing more, so the message goes out without it, and the export counts it and says
so."

## Guard records

Three new; the arrived-since count 610 to 613; 1,410 records by the TOML reader.

| Record | Break | Red |
|---|---|---|
| a mailbox file nothing went into leaves the name chosen as it was | `if !anything_went_in {` made `if false {` | 3: the writer's nothing-written case and two `exporting_mail` cases |
| a mailbox file of one folder counts only the folders inside it as left out | the separator dropped from `inside` | 2: the round trip and the kept-bytes case |
| Export Folder as a Mailbox File asks whether what is chosen is a folder | the helper call replaced, suite `mail_goes_out_in_every_shape` | 2: the census and the reaches-its-writer reading |

The last record's `before` is the handler's signature down to the call, because the call line
alone is in Export Mailbox too; `test_every_guard_record_still_names_one_place_in_the_tree`
passed with it. Probes before editing `wx_app.rs`: no record's `before` lay in Export Mailbox's
loop, and in the File builder only "a PGP private key somebody can import" and "File, Print
carries Ctrl+P in its label", neither duplicated by the insertion.

Remeasured, one call per commit that flagged them:

- `51fbc763`: "a name written back out is quoted when it needs to be" (all 20 red, nothing
  else) with the two new ones (3 and 2, nothing else), 464 s; then "every check lets the search
  index forget what was taken off" (1, nothing else), 26 s.
- `24ab8f8f`: the five naming `importing_messages.rs`, `--log` in the background. Four held.
  "which end of a folder's numbering a filed row takes" also reddened
  `data::message_cache::in_the_trash::tests::test_what_has_been_in_the_trash_leaves_out_deleted_filed_here_and_waiting_rows`,
  a test 13-44.6 added in a file the record did not name; added to its red list and
  `tests_last_seen`, measured again: all 12 red, nothing else, 212 s.
- `23a5ad18`: the new record, 25 s.

## Ledger

Opened 789 (`unrun-verify`, the tester: the file in Thunderbird and his own mail program, the
item and its sentences heard). Fixed 790 (Export Mailbox's missing empty line) and 791 (the
compaction census red on main). 791 in all, 687 open, 104 fixed, 0 waived, read with Python
from both halves, agreeing. Nothing here touches a server, so nothing went to phase 14.

## Version and pull request

No version bump: no build has been cut since 1.0.0-alpha.1. Nothing added to `Cargo.toml` or
`Cargo.lock` (T-13-45-SC). The plan changes what is shown and spoken, so the branch is pushed
with a pull request; its number and the three runs are in the merge report.

## Deviations from Plan

**1. [Rule 1] Export Mailbox lost every message after the first in each folder, fixed through
the shared step (ledger 790).** The round trip red-then-green showed the bare file reading back
as one message. `added_to_the_archive` writes the empty line before a separator only when its
buffer already holds a message, and since `1fbe263b` (2026-08-29) Export Mailbox built each
message in a fresh buffer, as the plan's `one_stored_message_added(cache, message, into)` would
have. So the step takes `InTheFile` (`First` or `AfterAnother`) as a fourth argument,
`message_files::between_two_messages` names the line, and
`test_messages_built_a_buffer_at_a_time_read_back_as_as_many_messages` holds the zip export's
pattern; taken red by hand by ignoring the landing, which reddened it and the round trip. A
changelog Fixed entry went in the same commit.

**2. [Rule 3] The compaction census had been red on main since 13-44.9 (ledger 791).** The first
green commit was refused by `test_nothing_else_compacts_the_search_index`, which read
`what_forgetting_costs.rs`, declared under `#[cfg(test)]` in its parent, as a shipping caller.
13-44.9's merge never ran that target. The census now skips a module its parent declares that
way, with `test_a_module_declared_only_for_tests_is_told_from_one_that_ships`; it went into
`51fbc763` rather than a commit of its own, because `guards/guards.toml` held both changes and
nothing may stage part of a file here.

**3. [Shape] The bare file's broke-off sentence is its own,
`export_tree::a_mailbox_file_that_broke_off`.** The plan named `an_export_that_broke_off`'s,
which says the file "is unfinished and should not be kept as a backup"; the bare file leaves
none, so its sentence says no file was written and a file already at that name is as it was.
A write failure also abandons; a failure to take the partial file away is said too.

**4. [Rule 1] "No file name was chosen." goes through `send_refusal`.** The first try at
`23a5ad18` was refused by `test_a_refusal_is_not_written_to_the_status_line`, which reads
refusals sent on the status channel; Save As already used `send_refusal` for that sentence.

**5. [Order] The shortcuts row and the changelog entry went in with the item in `23a5ad18`,
not in task 4,** because CLAUDE.md puts both in the commit that makes the change. The Fixed
entry for deviation 1 went in `51fbc763` for the same reason. The guide, the ledger and
this summary are in the documents commit.

**6. [Shape] The separator guard reddens the round trip through its sentence, not by mixing
Invoices' mail into Work's file.** The folder's mail is read by its exact path, so the filter
only counts; dropping the separator counts Work itself as inside it, and the round trip's
"1 folder inside it was not included" and the kept-bytes case's exact sentence go red.

**7. [Brief] One read-only shell command opened with a do-nothing assignment named after a
banned text tool,** the slip 13-44.6 to 13-44.9 recorded; nothing ran the tool. One commit
message in the scratchpad was extended through a Python heredoc, and a stray no-op heredoc
went to `/dev/null`; neither touched a tracked file. Every tracked file was changed with Edit
or Write, by `cargo fmt`, or by `scripts/guards.sh`.

## TDD Gate Compliance

Task 1: `e8f3d404` (test, red: five writer cases, three loop cases, five export_tree cases and
the count check, accepted by `red-commit.sh`) then `51fbc763` (feat). Task 2 is the removal
decision 6 makes, with no red, said in its commit message. Task 3: `bdd551f6` (test, red: three
readings, accepted) then `23a5ad18` (feat). Task 4 is documentation.

## Known Stubs

None. Every new function has a non-test caller: the menu item reaches the handler, the handler
the loop, the loop the writer and the sentence; Export Mailbox reaches the shared step.

## Threat Flags

None beyond the register. T-13-45-01: `.partial` with `create_new`, renamed only at a finish
something went into, `abandon` on a write failure, three cases and a guard record. T-13-45-02:
the shared sentences, the folders-not-included sentence and its guard record. T-13-45-03:
`a_folder_to_write_out` in both handlers, the census and its companion, a guard record.
T-13-45-SC: nothing added to `Cargo.toml`.

## Self-Check: PASSED

The two created source files exist; the five code commits are in `git log --oneline
main..HEAD`; the four marks are made in the documents commit.

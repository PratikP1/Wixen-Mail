---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 50
subsystem: import
tags: [import, attachments, gap-13, research-5]
status: complete
requires: [13-48, 13-49]
provides:
  - "message_files::MessageFromAFile::files, the files of the one message being filed, walked only when the parse lists any"
  - "AttachmentWithContent::all_from_a_parse, the one pairing of a parse with its files, shared by the import and parts_left_behind::keep_every_part"
  - "file_one_imported_message storing a message's files, and WhetherItWasWrittenDown::ItIsInTheFolder carrying how many were too large to keep"
  - "importing_messages::what_files_too_large_to_keep_left, said by both closing sentences"
  - "import_tree::FoldersImported::carry_the_mail_counts, the one place the folder import carries the mail's counts"
affects: [13-51]
tech-stack:
  added: []
  patterns: ["one pairing of a parse with its walk for every place that records a message it has whole", "a count taken by the rule the store keeps by, so what is said and what was kept cannot disagree"]
key-files:
  created:
    - .planning/phases/13-new-features-most-from-the-outlook-gap-audit/13-50-SUMMARY.md
  modified:
    - src/application/message_files.rs
    - src/application/importing_messages.rs
    - src/application/import_tree.rs
    - src/application/parts_left_behind.rs
    - src/application/exporting_mail.rs
    - src/data/message_cache/attachment_content.rs
    - src/service/outlook_data_file/one_saved_message.rs
    - src/presentation/wx_app.rs
    - tests/every_number_carries_its_command_and_its_date.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
decisions:
  - "The reader's records are built in parts_left_behind::keep_every_part on the day, so that is the reader's caller of all_from_a_parse; wx_app.rs holds no from_a_parsed_part call."
  - "The count of files too large to keep travels on WhetherItWasWrittenDown::ItIsInTheFolder, from the filing that kept the files, rather than being worked out again by each caller."
  - "The plural sentence says the files stay only where you imported them from, because a folder or zip import's large files may sit in several files."
metrics:
  duration: "about 3 hours 30 minutes"
  completed: 2026-10-03
estimate:
  tokens: 90000
actuals:
  tokens: 24000    # chars/4: about 30,000 in the code and test commits, about 66,000 in guards, pages, ledger and planning
  tasks: 3
  commits: 5
---

# Phase 13 Plan 50: Imported messages keep their files Summary

A message brought in from a saved message, a mailbox file, a zip, a folder or a message Outlook
saved now arrives with the files it carried, stored with their names, types and bytes through
the same pairing the reader uses when it keeps a message it fetched. A file over 25 MB is listed
on its message and not kept, and both closing sentences count it and say it stays only in what
was imported. Exporting the folder as message files writes an imported message's files back out.

## The finding, measured

RESEARCH-5's finding was right. Before any change, the first case,
`application::importing_messages::end_to_end::test_a_message_imported_from_a_file_keeps_the_files_it_carried`,
failed with `left: []` against the PDF it carried: the row said the message had files and none
was stored. The mailbox, saved Outlook message and signed cases failed the same way, and the
export round trip wrote the imported message with no file (`left: []`).

## What works now

- `MessageFromAFile` gains `files`, filled in `read_out_of` by `mime::attachments_with_bytes`
  only when the parse lists an attachment. The compiler named no other construction site.
- `AttachmentWithContent::all_from_a_parse(message_id, parsed, files)` pairs by position, a row
  past the walk's end listed without a file. `parts_left_behind::keep_every_part`, the reader's
  keeping of a message it fetched, calls it in place of its own mapping, storing the same rows.
- `file_one_imported_message` stores the files in a statement after the signed-form block,
  logging a failure without the file's name or bytes and still counting the message as filed.
  It answers `ItIsInTheFolder { files_too_large_to_keep }`, counted by
  `is_small_enough_to_keep`, now `pub(crate)`, which is the rule the store keeps by.
- `what_files_too_large_to_keep_left` says, with the limit read from
  `LARGEST_ATTACHMENT_KEPT_BYTES`: "1 file was over 25 MB, the most Wixen Mail keeps of one
  file, so it is listed on its message and stays only in the file you imported from", and for
  more than one, "...so they are listed on their messages and stay only where you imported them
  from". Both `what_the_mail_import_did` and `what_the_folder_import_did` call it, and both
  every-sentence checks hold it.
- The window's archive loop carries the mail's counts through
  `FoldersImported::carry_the_mail_counts`, which replaced two hand-written lines and carries the
  new count with them.
- The `.pst` import is unchanged: its messages list no files, so nothing is walked or stored, and
  its sentence still counts the files that stayed in the data file. Its 8 cases pass.

## Commits

| Commit | What | Hook |
|---|---|---|
| `cc98a454` | test: the kept-files cases, the export round trip, the pairing case, the stub, the fixture | red, 167 s |
| `13f0f0a0` | feat: the files carried and filed, the shared pairing; 2 records new, 1 moved, 12 remeasured | affected, 306 s |
| `077a9b15` | test: the too-large count and sentence, the carrying | red, 207 s |
| `673ce623` | feat: the count, the sentence, the carrying; 2 records new, 13 remeasured | affected, 412 s |
| docs | the guide, the figure registered, the changelog, the ledger, GAP-13's note, this summary, the four marks | this commit |

## Counts

Taken 2026-10-03:

| Target | Before | After |
|---|---|---|
| `cargo test --lib application::importing_messages::` | 42 | 48 |
| `cargo test --lib application::import_tree::` | 36 | 38 |
| `cargo test --lib application::exporting_mail::` | 6 | 7 |
| `cargo test --lib data::message_cache::attachment_content::` | 22 | 23 |
| `cargo test --lib application::message_files::` | 51 | 51 |
| `cargo test --lib application::parts_left_behind::` | 8 | 8 |
| `cargo test --test every_number_carries_its_command_and_its_date` | 26 | 26 |

`grep -c 'all_from_a_parse('`: 1 in `importing_messages.rs`, 1 in `parts_left_behind.rs`, 0 in
`wx_app.rs`; `from_a_parsed_part(` in `wx_app.rs` 0 and in `parts_left_behind.rs` 0.
`files_too_large_to_keep` appears in both `importing_messages.rs` and `import_tree.rs`;
`LARGEST_ATTACHMENT_KEPT_BYTES` in `importing_messages.rs`, which composes the sentence.
`THE_FIGURES_THAT_RESTATE_A_CONSTANT` went from 12 to 13 with the guide's figure; planting 24 in
the guide's sentence turned `test_every_figure_that_restates_a_constant_agrees_with_it` and its
two companions red with "docs/USER_GUIDE.md: "without the original. A file larger than 24 MB"
restates LARGEST_ATTACHMENT_KEPT_BYTES, which is 25 MB", and putting 25 back made all 26 pass.
`tr '\n' ' ' < docs/USER_GUIDE.md | grep -c 'stays only in the file you imported from'`: 1.

## Guard records

Four new; one moved; the arrived-since count 626 to 630.

| Record | Break | Red |
|---|---|---|
| an imported message is filed with the files it carried | the files statement made `let _ = &read.files;` | the four kept-files cases, the too-large case, the export round trip |
| the reader and the import pair each part with its own file | `files.get(at)` made `files.get(at + 1)` | the pairing case, the fetched-parts case, the four kept-files cases, the export round trip |
| files too large to keep are said by both imports | `match how_many.min(0)` | the two sentence cases and the end-to-end too-large case |
| an imported message goes out again with its files | the export's files made `Vec::new()` | the round trip and two `what_an_export_holds` cases in `wx_app.rs` |

"a message fetched for its parts keeps every part with its file" moved its anchor from
`parts_left_behind.rs` into `all_from_a_parse` and names the import cases too. The signed-mail
record's anchor, the whole `if let Some(raw) = &read.the_form_it_arrived_in` block, occurs once
in `importing_messages.rs` and was left byte for byte; its red list gained the new signed case.

The first remeasure named fourteen records in one call, `--log` in the background, 1,753 s:
thirteen agreed and "the form a signed message arrived in is really kept" also reddened the new
signed case, corrected by hand and remeasured alone, 239 s. The second named thirteen, 1,664 s:
eleven agreed; the files record also reddened the too-large case and the export record two
`wx_app.rs` cases, both corrected by hand and remeasured together, 391 s.

## Ledger

Opened 800 (`unmet-truth`), the finding, and marked it fixed with `13f0f0a0` and `673ce623`, so
the gap's history stays on the ledger (guardrail 9). Opened 801 (`unrun-verify`, the tester): a
real mailbox file with attachments imported and a file opened from it. 798 gained a dated
sentence: the folder import's carrying is now one function, where its three missing counts
would go. Both halves written for all three; the header reads 696 open, 105 fixed, 801 in all.
Nothing touches a server, so nothing goes to phase 14.

## Deviations from Plan

**1. [Premise] The reader's records are built in `parts_left_behind::keep_every_part`, not
`wx_app.rs:24757`.** 13-21.3 moved them there after this plan's premises were taken. That is
the caller switched to `all_from_a_parse`, and the acceptance grep reads it in place of
`wx_app.rs`, which holds no `from_a_parsed_part` call at all.

**2. [Order] The export round trip went into task 1's red commit.** Once task 1 stored the files
it would have passed before task 2 changed anything, so it was red where the defect was.

**3. [Shape] The count travels on `ItIsInTheFolder { files_too_large_to_keep }`** from the filing
that kept the files, and `count_one_written` adds it. Two assertions on the old unit variant
changed to name the field.

**4. [Shape] `FoldersImported::carry_the_mail_counts`.** The plan had the window carry the new
count by hand beside the two it already carried; one function with a case holds all three, so a
count cannot be added and left behind again (ledger 798 is that shape).

**5. [Scope] `one_saved_message.rs`, not in the plan's file list,** gained
`for_tests::a_saved_message_carrying_two_files`, and five of its test constants became
`pub(super)` for it. No test was added there.

**6. [Wording] The plural sentence says "stay only where you imported them from".** A folder or
zip import's large files can sit in several files.

**7. [Brief] Changelog in the documents commit; no commit on `main` after the merge** to add its
mode line here, since the brief allows one documents commit, before the merge.

**8. [Brief] A banned text tool's name was typed once, as a do-nothing assignment** in a
read-only command over `STATE.md`. No tracked file was touched by it. Every tracked file was
changed with Edit or Write, by `cargo fmt`, or by `scripts/guards.sh`.

## TDD Gate Compliance

Task 1: `cc98a454` (test, red: six cases and the count check, accepted by `red-commit.sh`) then
`13f0f0a0` (feat). Task 2: `077a9b15` (test, red: four cases and the count check, accepted) then
`673ce623` (feat). Task 3 is documentation and the figure's registration, taken red by hand.

## Known Stubs

None. Every import that reads a message from a file reaches `file_one_imported_message`, which
stores the files; both closing sentences call the sentence function, and the window's loop calls
the carrying.

## Threat Flags

None beyond the register. T-13-50-01: the store's limit refuses each file over 25 MB, counted;
files are held for one message and let go when it is filed. T-13-50-02: the two new log lines
name no file and carry no bytes. T-13-50-03: one `all_from_a_parse`, a case and a record.
T-13-50-04: the kept-files cases, the count and sentence, ledger 800. T-13-50-SC: nothing
installed.

## Self-Check: PASSED

The four code commits are in `git log --oneline main..HEAD`; this summary exists; the four marks
are made in this commit.

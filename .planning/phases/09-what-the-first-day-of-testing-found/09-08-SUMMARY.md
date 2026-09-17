---
phase: 09-what-the-first-day-of-testing-found
plan: 08
subsystem: importing mail, the Outlook data file reader, Save As, the export writer, guards
tags: [pst, outlook, import, save-as, eml, export_tree, importing_messages, wired, guards, changelog]

requires:
  - phase: 04-writing-and-reading-a-message-in-full
    provides: "service::outlook_data_file, the reader that yields ItemInTheDataFile in the cache's own types and composes each Mail item through message_files::written_as_one_message; importing_messages::file_one_imported_message, the one writer that marks a row as filed here; export_tree::rebuilt_from_what_is_stored and the files walk"
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-07: the pattern of a red commit with stubs answering the tree as it is, the count check's remedy run after the green, and a wired.rs companion that reads the tree before splicing"
provides:
  - "application::importing_an_outlook_data_file: brought_in (opens a data file and walks its folders), one_folder_filed (files each of the five kinds: Mail through each_message_in(ReadAs::OneMessage) and file_one_imported_message under Imported in the file's own folder; Appointment, Contact, Task, Note under the local account), WhatCameAcross and what_the_data_file_import_did, the closing sentence"
  - "import_tree::WhatWasChosen::AnOutlookDataFile, keyed on outlook_data_file::HOW_ONE_BEGINS, now public; import_tree::where_a_folder_of_the_data_file_lands"
  - "importing_messages::a_folder_for_imported_mail, moved out of the window so three imports make folders one way; importing_messages::saving_as and SavingAs, the Save As decision, with CHOOSE_THE_MESSAGE_TO_SAVE"
  - "export_tree::one_message_written_out: one stored message as a single-message file, exactly as it arrived where that was kept"
  - "wx_app::save_the_message_as and one_message_saved_to; the import worker's three-way dispatch; *.pst in the picker"
  - "tests/wired.rs: what_save_as_does with its companion, the three-way dispatch reading, the signed-form reading widened to the new module"
  - "Six guard records measured, one older record corrected from what the remedy found"
affects: [09-10, which takes #53's points 3 and 7 and corrects the four documents and closes the phase's read of FOUND-11; the tester, who has the first .pst this program will ever meet; #52, whose reading half should note Save As keeps a signed original byte for byte]

actuals:
  tokens: 25000
  tasks: 2
  commits: 5

tech-stack:
  added: []
  patterns:
    - "When a reader's input cannot be built in a test, the seam is the reader's output type: the walk over a real input is a few lines over the reader's own iterators, named as unrun where whoever runs it will hear it, and the filing of what the walk hands over is what the tests hold"
    - "A worker dispatches on every answer of the one question it asks, with a wired.rs reading that names each answer and the call it leads to; an answer the worker recognises and hands to nothing is the shape a guard record breaks"
    - "A save command asks one decision for its refusal and its name and one writer for its bytes, and the reading that holds it follows the handler into its fallible half rather than reading the handler alone"

key-files:
  created:
    - src/application/importing_an_outlook_data_file.rs
  modified:
    - src/application/import_tree.rs
    - src/application/importing_messages.rs
    - src/application/export_tree.rs
    - src/application/mod.rs
    - src/presentation/wx_app.rs
    - src/service/outlook_data_file.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/KEYBOARD_SHORTCUTS.md

key-decisions:
  - "The reader is wired, not retired, on the plan's reasoning: it already speaks the cache's types, the one seam exists, and what this program replaces is Outlook"
  - "The new module's tests hand it items of each kind and read them back out of a temporary cache, because neither this program nor the outlook-pst crate can write a data file; the walk over a real file is a thin half that says it is unrun"
  - "Appointments out of a data file go into opening::IMPORTED_CALENDAR under the local account, where an imported calendar file's already go, and contacts, tasks and notes go under the local account with no list or folder, which the writers do not need; every one is re-homed there by the filing whatever account the reader was asked for"
  - "The mail folder for a data file's folder is made when its first message needs it, so a folder of nothing but contacts leaves no empty mail folder behind; a folder the file names unusably is refused once and its mail stays in the file, said in the sentence"
  - "Save As writes through a new export_tree::one_message_written_out rather than through written_as_one_message in the window, because the exporter already owns stored-message-to-bytes, the files walk and the signed-original rule, and a saved message and an exported one should be the same file; it does not go through ending_where_a_line_ends, which is what a trip through an archive changes and a single file must not"
  - "The Save As name replaces path separators before safe_file_name sees the subject, because that function keeps only the last segment of anything that looks like a path and Invoices/March is one subject"
  - "The reader's refusals stay in Error::Other and the seam takes the words out, because moving the reader onto InPlainWords is a change to the reader wanting its own red (ledger 500)"

patterns-established:
  - "A count check cannot see a test written in a new file no record names: 09-07's reading of a signed message in the cache reddened an older record and was found only because this plan's tests made that record's remedy run"

requirements-completed: []

coverage:
  - id: D1
    description: "File, Import Mailbox lists *.pst, a chosen data file goes to its own reader, each Mail item is filed the way a saved .eml is under Imported in the file's own folder with the marker, and appointments, contacts, tasks and notes land under the local account through the cache's four writers"
    requirement: FOUND-11
    verification:
      - kind: unit
        ref: "src/application/import_tree.rs#test_an_outlook_data_file_goes_to_its_own_reader"
        status: pass
      - kind: unit
        ref: "src/application/import_tree.rs#test_a_folder_of_the_data_file_lands_under_imported_by_the_archives_rules"
        status: pass
      - kind: unit
        ref: "src/application/importing_an_outlook_data_file.rs#test_mail_out_of_the_data_file_lands_under_imported_with_the_marker"
        status: pass
      - kind: unit
        ref: "src/application/importing_an_outlook_data_file.rs#test_a_message_the_folder_already_holds_is_left_as_it_is"
        status: pass
      - kind: unit
        ref: "src/application/importing_an_outlook_data_file.rs#test_each_of_the_four_kinds_lands_under_the_local_account_and_not_the_servers"
        status: pass
      - kind: unit
        ref: "src/application/importing_an_outlook_data_file.rs#test_a_folder_with_a_name_this_computer_cannot_use_keeps_its_mail_and_hands_over_the_rest"
        status: pass
      - kind: unit
        ref: "src/application/importing_an_outlook_data_file.rs#test_a_refusal_from_the_reader_ends_the_folder_and_is_kept_to_be_said"
        status: pass
      - kind: integration
        ref: "tests/wired.rs#test_importing_mail_sends_each_kind_of_file_to_its_own_reader"
        status: pass
      - kind: other
        ref: "grep -c '\\*.pst' src/presentation/wx_app.rs answers 2"
        status: pass
    human_judgment: false
  - id: D2
    description: "The closing sentence counts each kind, says what stayed behind and about a password, and ends by saying no real Outlook data file has been through this program"
    requirement: FOUND-11
    verification:
      - kind: unit
        ref: "src/application/importing_an_outlook_data_file.rs#test_the_closing_sentence_counts_each_kind_and_says_no_real_file_has_been_read"
        status: pass
      - kind: unit
        ref: "src/application/importing_an_outlook_data_file.rs#test_the_closing_sentence_says_what_stayed_behind_and_about_the_password"
        status: pass
      - kind: unit
        ref: "src/application/importing_an_outlook_data_file.rs#test_an_import_that_brought_nothing_says_so_rather_than_counting_nothing"
        status: pass
    human_judgment: false
  - id: D3
    description: "File, Save As writes the message under the cursor as .eml through the ordinary save dialog with a name made from its subject, refuses with a reason when nothing is selected, keeps a signed original byte for byte, and the reader's Save Attachment is the attachment case, said on the shortcuts page"
    requirement: FOUND-11
    verification:
      - kind: unit
        ref: "src/application/importing_messages.rs#test_save_as_offers_the_subject_as_the_file_name_ending_in_eml"
        status: pass
      - kind: unit
        ref: "src/application/importing_messages.rs#test_save_as_takes_the_path_out_of_a_subject_before_offering_it_as_a_name"
        status: pass
      - kind: unit
        ref: "src/application/importing_messages.rs#test_save_as_calls_a_message_with_no_subject_something"
        status: pass
      - kind: unit
        ref: "src/application/importing_messages.rs#test_save_as_with_no_message_chosen_says_so_rather_than_saving_nothing"
        status: pass
      - kind: unit
        ref: "src/application/export_tree.rs#test_one_message_saved_as_a_file_reads_back_with_its_files_and_no_separator"
        status: pass
      - kind: unit
        ref: "src/application/export_tree.rs#test_one_message_saved_as_a_file_is_written_exactly_as_it_arrived_when_that_was_kept"
        status: pass
      - kind: unit
        ref: "src/application/export_tree.rs#test_one_message_saved_as_a_file_is_not_written_when_its_text_was_never_downloaded"
        status: pass
      - kind: integration
        ref: "tests/wired.rs#test_save_as_writes_the_chosen_message_rather_than_sending_a_status_line"
        status: pass
      - kind: integration
        ref: "tests/wired.rs#test_the_reading_of_save_as_can_see_the_stub_put_back"
        status: pass
      - kind: other
        ref: "grep -c '\"Save As: no message selected\"' src/presentation/wx_app.rs answers 0; docs/KEYBOARD_SHORTCUTS.md:485 names both commands"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether a real Outlook data file reads, and whether a file Save As wrote opens in another mail program"
    requirement: FOUND-11
    verification: []
    human_judgment: true
    rationale: "FOUND-11's [S] line, ledger 499 and 502. No data file can be built here and none has been read; nobody has opened a saved .eml elsewhere. The tester's, when he has a .pst to hand"

duration: 1h19m
completed: 2026-09-16
status: complete
---

# Phase 9 Plan 08: The Outlook data file read from the picker, and Save As saving Summary

**File, Import Mailbox lists `*.pst` and hands a chosen Outlook data file to a new module,
`application::importing_an_outlook_data_file`, which files each message the reader composes the
way one saved `.eml` is filed, under Imported in the file's own folder with the marker the sync
leaves alone, and its appointments, contacts, tasks and notes under the local account through the
cache's four writers, then says what came across, what stayed in the file, whether the file had a
password, and that no real Outlook data file has been through this program. File, Save As, a stub
since the item was added, writes the message under the cursor as `.eml` through the exporter's own
writer, a kept signed original byte for byte, with a name made from the subject with the path taken
out; the reader keeps Save Attachment and the shortcuts page says which does which. Proven against
items of each kind handed to the filing and read back out of a temporary cache, and against this
program's own reader reading the saved file back; whether a real `.pst` reads, and whether a saved
file opens elsewhere, is the tester's.** #53 advanced with the merge commit, points 1 and 2. Nothing
pushed.

## Performance

- **Duration:** 1 h 19 min from the branch to the merge, of which about 35 minutes were the
  count check's remedy over 23 records, 4.5 the first three records' measurement, 2.5 the one
  corrected record measured again, and 12.5 the two whole gates (373 s on the branch, 379 s on
  `main`'s hook at the merge)
- **Started:** 2026-09-17T01:23:29Z (first commit 01:28:15Z)
- **Merged:** 2026-09-17T02:42:11Z at `06fdc9b7`
- **Tasks:** 2
- **Files modified:** 11 (1 created)

## What landed

| Where | Before | Now |
|---|---|---|
| The picker, `import_a_mailbox` | `*.zip;*.eml;*.mbox` | `*.zip;*.eml;*.mbox;*.pst`, labelled "Mailboxes and Outlook data files"; the item's description names an Outlook data file |
| `import_tree::what_was_chosen` | two answers from the bytes; a `.pst` fell to the archive reader and was refused as not a mailbox archive | three: `AnOutlookDataFile` when the file begins `!BDN`, before the mail question is asked |
| The worker, `fill_folders_from` | `if MailInOneFile { return } else archive` | a `match` on all three answers; the third returns `importing_an_outlook_data_file::brought_in` with the worker's `say` for the running count |
| Mail out of a data file | nowhere | `each_message_in(&bytes, ReadAs::OneMessage)` then `file_one_imported_message`, into `Local/Imported/<the file's folder>` made on the first message, by `import_tree`'s naming rules; a folder named unusably is refused once and its mail stays |
| Appointments, contacts, tasks, notes | nowhere | `save_calendar_event`, `save_contact`, `save_task`, `save_note`, each re-homed to `WHERE_IMPORTED_THINGS_GO` by the filing; appointments into `IMPORTED_CALENDAR` |
| The closing sentence | none | counts each kind, then what was already here, what would not save, folders refused, the reader's four counts of what stayed behind, each size refusal in the reader's words, the password, and always "No real Outlook data file has been through this program before, so check what arrived against Outlook" |
| `ID_SAVE_AS` | `send_status(.., "Save As: no message selected")` whatever was selected | `save_the_message_as`: the decision, the save dialog opening in the download folder with the suggested name, a worker reading the row, its text, its files and its signed original, `one_message_saved_to` writing through `export_tree::one_message_written_out`, the status line saying what was saved and where, `ErrorOccurred` for a save that did not happen |
| `docs/KEYBOARD_SHORTCUTS.md:485` | "Save message or attachment to file" | the message as `.eml` named after its subject; an attachment from the reader's own Save Attachment, `Ctrl+S` there |

The closing sentence for one item of each kind, from the test that holds it:

> Imported 3 messages into 2 folders, 1 appointment, 2 contacts, 1 task, 1 note. No real Outlook
> data file has been through this program before, so check what arrived against Outlook.

The suggested file name for a subject holding a slash, from the test that holds it: `Invoices/March`
is offered as `Invoices_March.eml`, and `..\..\Windows\notes: "Re: the engine?"` as
`.._.._Windows_notes_ _Re_ the engine__.eml`. An empty subject is `message.eml`; nothing selected is
refused with "Choose the message to save first. Select one message in the list."

## Task commits

| Commit | What |
|---|---|
| `e80abb88` | test(09-08): the red half of task 1, ten named; stubs answering the tree as it is |
| `53fa1a96` | feat(09-08): the module, the arm, the folder function, the folder helper moved, the picker and the worker, three records, the changelog and the dated entry |
| `12918ba4` | test(09-08): the red half of task 2, seven named and the count check, two green against the stubs and said so |
| `2356ac80` | feat(09-08): the decision, the writer, the handler, the readings, three records, the remedy over 23 with one older record corrected, the shortcuts row and the changelog |
| `06fdc9b7` | Merge 09-08 into `main` |

Branch `the-outlook-file-is-read-and-save-as-saves` from `main` at `1c1ce7f7`. Not pushed; 74
commits unpushed before the commit that lands this summary.

## Honest RED and GREEN

Task 1's red named ten. Against a `what_was_chosen` without the arm, a folder function answering
nothing, a filing that consumed its items and wrote nothing, and an empty sentence, all ten were red
for their own reasons: the arm test found `AnArchive`, the folder test found `None`, the marker test
found no folder under Imported, and the six module tests found nothing filed and nothing said. The
gate in `red` mode ran exactly the ten. The count check did not fire, because no record named
`import_tree.rs` or the new module.

Task 2's red named seven and the count check. Against a decision that refused everything, which is
what the command did, and a writer that wrote nothing, three name tests and two writer tests were
red on their assertions and the two `wired.rs` readings panicked in `body_of` where no handler was;
two passed against the stubs and the message says which, nothing chosen and no text. The count check
fired on `importing_messages.rs` 37 to 41, `export_tree.rs` 35 to 38 and `wired.rs` 72 to 74, naming
twenty records. The green's first run of the writer test found the body read back with a trailing
line ending, because the writer went through `ending_where_a_line_ends`, whose own comment says a
message written out on its own keeps its body as stored; the writer now does not, and the test held.
The `wired.rs` reading's first run found `writes_through_the_exporter` false, because the writer is
called from the handler's fallible half `one_message_saved_to` and not from the handler; the reading
follows the handler into that half now, which is what it should have read from the start.

The three-way dispatch reading in `wired.rs` arrived green in task 2's green commit, its behaviour
having been taken red in task 1 by the arm test and the module tests; its guard record breaks it
and was measured. Said here rather than hidden in a red list it was never in.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/import_tree.rs`, `src/application/importing_an_outlook_data_file.rs`, `src/application/mod.rs` | `--lib application::import_tree`, `--lib application::importing_an_outlook_data_file`, and on the red commit `--lib application` because `mod.rs` changed |
| `src/application/importing_messages.rs` | `--lib application::importing_messages` on the three commits that touched it |
| `src/application/export_tree.rs` | `--lib application::export_tree` on the red and green of task 2 |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app` (199 tests by the count check's count) and the eleven coupled targets `--suites-for` answers for it, on both green commits |
| `src/service/outlook_data_file.rs` | `--lib service::outlook_data_file`, 38 tests, on task 1's green |
| `tests/wired.rs` | `--test wired` on the red and green of task 2 |
| `guards/guards.toml`, `docs/changelog.md`, `docs/KEYBOARD_SHORTCUTS.md` | no scoped target; rode the green commits, and the house_style tests read all three on every commit |

Every commit ran formatting, clippy, the four script suites and the whole-tree guards.
`scripts/check.sh all` on the branch at `2356ac80`, output to a file and the exit status read
directly, never piped: exited 0 in 373 s on its first run, 7,862 passed and none failed over 66
result lines, the release build included; 20 more than 09-07's 7,842, which is this plan's 10, 3, 4
and 3. `main`'s hook ran `all` again on the merge: 7,862 and none failed, 379 s. Ledger 374's
keyring race did not appear on either run.

## Guard records

852 records by the TOML reader before, 858 after; census 802 + 50 before, 802 + 56 after, the line
at `guards/guards.toml:84` moved in each green commit. Six new, one older corrected, all measured
through `scripts/guards.sh --remeasure`, `WIXEN_TEST_THREADS` untouched.

| Record | File and suite | Break | Red | Run |
|---|---|---|---|---|
| mail out of a data file is filed with the marker the sync leaves alone | `importing_an_outlook_data_file.rs` | the row filed through `upsert_message` instead | 1 | rebuild 40 s, run 48 s |
| each kind out of a data file is counted when it is saved | `importing_an_outlook_data_file.rs` | `one_saved` counts nothing | 3 | 40 s, 48 s |
| an Outlook data file is sent to its own reader rather than the archive's | `import_tree.rs` | the arm keyed on `b"never"` | 1 | 39 s, 48 s |
| Save As takes the path out of a subject before offering it as a name | `importing_messages.rs` | the separators left in | 1 | 40 s, 49 s |
| Save As reaches a handler that writes rather than a status line | `wx_app.rs`, `wired` | the stub's line put back in the arm | 2 | 16 s, 2 s |
| an Outlook data file the worker recognises is handed to its reader | `wx_app.rs`, `wired` | the third arm does nothing | 1 | 16 s, 2 s |

**What the remedy found.** The count check fired once, on task 2's red, naming twenty records
(fifteen on `wired.rs`, four on `importing_messages.rs`, one on `export_tree.rs`); it was run once
after the green with the three new task 2 records added, 23 in all, about 35 minutes. Twenty-two
were exactly what they said. One was not: "the form a signed message arrived in is really kept",
whose break reddened 09-07's `reading_a_message::tests::test_a_signed_message_in_the_cache_has_its_verdict_and_an_enveloped_one_its_sentence`,
written the same day in a new file no record named, which is the case `CLAUDE.md` says the count
check cannot see. Corrected by hand with a comment saying why, `reading_a_message.rs` added to its
`tests_last_seen` at 7, and measured again: all ten named went red and nothing else.

Counts written: the new module 8, `import_tree.rs` 33, `importing_messages.rs` 41 (was 37),
`export_tree.rs` 38 (was 35), `wired.rs` 75 (was 72; the three-way dispatch reading was added
after the count check had said 74, so the remedy wrote 75), `wx_app.rs` 199 (unchanged; 56 records
name it now, was 54), `reading_a_message.rs` 7 on the corrected record. The count check fired on
task 2's red commit and on nothing after its remedy.

## Premises the tree contradicted

1. **The reader's tests have no fixture to reuse.** The plan said "the reader's own 38 tests have
   fixtures; reuse them" and "build a data file the way `outlook_data_file.rs`'s own tests do". The
   reader's header says "An Outlook data file cannot be written by this program or by the reader
   underneath it, so no test here builds one", and the crate's README says it is read-only; its
   tests build items at the property level through private types. So the new module is two halves:
   `one_folder_filed` takes an iterator of `Result<ItemInTheDataFile>` and is tested against one
   of each kind read back from a temporary cache, and `brought_in` is the walk over the reader's
   iterators, unrun and said so in the module header, the changelog and the sentence. Observation
   618.
2. **`safe_file_name` keeps only the last segment of a path.** The plan asked for "path separators
   and other unsafe characters replaced"; handed `Invoices/March`, the one function that knows what
   a name may hold answers `March`. The decision replaces `/` and `\` with `_` first and hands the
   rest to it, and the record breaks exactly that.
3. **The reader's sentences arrive as `Error::Other`**, whose display puts "Error:" in front, so a
   size refusal said at the end would have been heard as "Error: Something in Inbox...". The seam
   takes the words out; the reader is unchanged, on the plan's own rule, and the move to
   `InPlainWords` is ledger 500.
4. **The exporter's `ending_where_a_line_ends` must not be on Save As's path**, which its own
   comment says and the first green run proved: the body read back with a line ending it was not
   stored with.
5. **`wired.rs` is named by 15 records at `1c1ce7f7`, not the plan's 14**, and 17 now;
   `wx_app.rs` by 54, not the README's 50, and 56 now. The count check counts 199 test functions
   in `wx_app.rs` where `grep -c '#\[test\]'` answers 196 and 72 in `wired.rs` where it answers 73;
   the records carry the check's count.
6. **The worker's folder helper lived in the window.** `a_folder_on_this_computer` was private to
   `wx_app.rs` and the new module needed it; it moved beside `file_one_imported_message` as
   `importing_messages::a_folder_for_imported_mail`, and the two window callers use it. No test
   moved with it: its behaviour is held by the new module's marker test, which makes the folder
   through it.
7. **09-07's reading of a signed message in the cache had reddened an older record** since the day
   it was written, found by this plan's remedy and corrected above.

Premises 1 (the reader unreached), 2 (wire, not retire), 3 (each `Mail` item is one message) and 4
(Save As a stub; the attachment case another frame's) of the plan held exactly.

## Deviations from plan

**1. [Rule 3] The new module's tests hand it items rather than reading a fixture.** Premise 1
above. Ledger 503.

**2. [Decision] `export_tree.rs` gained `one_message_written_out`.** Not in the plan's file lists,
which named `message_files.rs` for the writer. The exporter already owns `rebuilt_from_what_is_stored`,
the files walk and the signed-original rule, and a saved message and an exported one should be the
same bytes; putting the function in `message_files` would have meant duplicating the files walk in
the window. `message_files.rs` is unchanged. Ledger 503.

**3. [Decision] The folder helper moved from the window to `importing_messages`.** Premise 6.
Ledger 503.

**4. [Sequencing] The three-way dispatch reading arrived green in task 2's green.** Said under
Honest RED and GREEN. Ledger 503.

**5. [Decision] The reader's `Error::Other` words taken out at the seam, not the reader changed.**
Premise 3. Ledger 500 and 503.

**6. [Process] No tracked file was edited by a script.** Every edit to a tracked file went through
Read then Edit or Write; `cargo fmt` ran before each commit; carriage returns measured with
`tr -cd '\r' | wc -c` on every changed file before each commit and on the ledger after it, zero on
each; no em-dash in any file this plan wrote, measured with `grep -c` for the byte sequence. The
exception set for scripted edits on a tracked file is zero, as 09-07 left it. Commit messages were
written to the scratchpad and passed with `-F`; the two whole-gate runs, the merge and every remedy
wrote to a file and the exit status was read directly. `git commit` and `git merge`, not
`gsd-tools query commit`. Nothing was sent to any window; no binary was started; the tester's
profile was not read. `Cargo.toml` untouched; no package added; `WIXEN_TEST_THREADS` at its default.

Everything else executed as written.

## Threat register

T-09-26: `HowMuchToAllow` and the reader's bounding are reused unchanged through `opened`, and the
work runs on the archive import's `spawn_blocking` worker. T-09-27: every `Mail` item goes through
`file_one_imported_message` and the first record breaks exactly that, filing the row through the
sync's own writer and reddening the marker reading. T-09-28: the decision replaces both separators
before `safe_file_name`, the second record breaks it, and the test holds a subject with a slash and
one with backslashes, quotes and a colon. T-09-29: the handler has no attachment branch, the reading
checks both halves for `save_attachment(` and `ID_SAVE_ATTACHMENT`, and the companion splices the
stub back. T-09-SC: no package added. New surface outside the register: a data file's folder names
become folder paths on this computer, through the same `the_folder_named_by` an archive's go through,
so a step out of a folder, a device name and a character Windows will not take are refused rather
than repaired, and a refused folder is counted and said.

## Ledger

`.planning/WINDOWS.md` 499 to 503 written through `gsd-tools windows append`, both halves, no
backslash in any description (the nine in the file predate this plan); entry 97 corrected by hand
in both halves with a dated sentence, and `the_planning_files_agree_with_themselves` green after,
16 passed. 498 before, 503 after; 470 open before, 475 after.

| id | kind | what |
|---|---|---|
| 499 | unrun-verify | no real Outlook data file has been through the import; `brought_in`'s walk is unrun; FOUND-11's `[S]` line, the tester's |
| 500 | todo | the reader's refusals are `Error::Other`, heard with "Error:" first; move them to `InPlainWords` with a red of their own |
| 501 | todo | the calendar, contacts, tasks and notes modules are not reloaded after a data file import the way the folder tree is |
| 502 | unrun-verify | nobody has opened a file Save As wrote in another mail program or run the command by hand |
| 503 | deviation | the five departures above |
| 97 | corrected | described an importer path that did not exist; it exists now, so the deviation is real and stays open |

## Known stubs

None. `brought_in` has one caller, the worker's third arm; `one_folder_filed` is called by
`brought_in`; `what_the_data_file_import_did` by `brought_in`; `where_a_folder_of_the_data_file_lands`
by `TheMailFolder::for_mail`; `a_folder_for_imported_mail` by the two window imports and the new
module; `saving_as` by `save_the_message_as`; `one_message_written_out` by `one_message_saved_to`,
which `save_the_message_as`'s worker calls; `save_the_message_as` by the `ID_SAVE_AS` arm.
`HOW_ONE_BEGINS` is read by `what_was_chosen` and by the reader's own `how_it_begins`.

## Not done here, on purpose

Reading a real `.pst` (ledger 499) and opening a saved `.eml` elsewhere (502) are the tester's.
The reader's error variant (500) and the module reloads after an import (501) are todos. #53's
point 3, the folder picker, and point 7, the guide, are 09-10's, with the four documents; points 4
to 6 are later work. No `FOUND` requirement is ticked, on the phase's rule that the last plan reads
each clause by clause; the closing read should count FOUND-11's first two `[D]` lines as met by the
readings named in the coverage block and its `[S]` line about a real file as the tester's. Nothing
pushed.

## Self-Check: PASSED

`src/application/importing_an_outlook_data_file.rs` exists; `grep -c '\*.pst'
src/presentation/wx_app.rs` answers 2; `grep -c '"Save As: no message selected"'
src/presentation/wx_app.rs` answers 0; `grep -rn 'outlook_data_file::' src` outside the reader and
the new module finds `import_tree.rs:540` and `wx_app.rs:12893`; `guards/guards.toml` holds 858
records by the TOML reader; `docs/changelog.md` holds one line beginning "**Corrected on 2026-09-16:**
from the day this entry was written". Commits `e80abb88`, `53fa1a96`, `12918ba4`, `2356ac80` and
`06fdc9b7` are in `git log --oneline` on `main`.

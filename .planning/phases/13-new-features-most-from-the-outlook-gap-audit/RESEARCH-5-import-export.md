# Phase 13, group 5: the rest of mail import and export (GAP-13) - Research

**Researched:** 2026-09-24, against `main` at `630e2a67` (phase 12 closed, tree clean).
**Domain:** writing mail out as a bare mailbox file and as loose saved messages; reading
Outlook's single-message file (`.msg`, a Compound File Binary container); whether a `.pst`
can be written at all.
**Confidence:** HIGH for what the tree holds and for the package measurements (every figure
below was taken today with the command beside it); MEDIUM for the `.msg` reader design (the
container was read on four real Outlook-written files, the property mapping on none yet);
HIGH that `.pst` export has no route this project should take.

Every claim is marked **(stated)** when I read it in a file, an issue or a spec page this
session, or **(derived)** when it is my inference from what I read. `[VERIFIED]`, `[CITED]`
and `[ASSUMED]` follow the GSD provenance rules.

## Summary

Two of the three halves are small, because the tree already has almost everything they need.
The exporter can already turn a stored message into mailbox-file bytes
(`export_tree::added_to_the_archive`) and into a single saved message
(`export_tree::one_message_written_out`); the Save As command writes the second to disk; and
a decision that picks `.eml` for one message and `.mbox` for several
(`importing_messages::writing_out`) has been written, tested and reached by nothing but its
tests since it was added (derived from a search, below). A bare `.mbox` of one folder and a
folder of loose `.eml` files are therefore two thin writers plus two File menu items, with a
round trip back through the existing imports as the test.

`.msg` reading is the only part that needs a new package. The file is a Compound File Binary
container holding MAPI properties, and the tree already has a complete, tested mapping from
MAPI properties to a message (`outlook_data_file::a_message_from`, `who_sent_it`,
`one_person`), built for the `.pst` reader. What is missing is the container. `cfb` 0.15.0
adds one lock entry and two build packages, has no `unsafe`, rejects looped chains, and
opened all four real Outlook-written samples I could find. The ready-made `.msg` crate,
`msg_parser`, refused a valid file and dropped a sender on the exact path this program would
call, so it is not recommended. Windows' own structured-storage reader is a real alternative
with no new package.

`.pst` export should be said plainly to be out. The only Rust writer is a four-day-old,
one-person fork of the reader this project already uses, published under the same library
name, which `Cargo.toml` already records a decision against; the Windows route (Extended MAPI
with the Personal Folders provider) needs Outlook installed, which the people leaving Outlook
for this program by definition may not have, and which this machine does not have.

**Primary recommendation:** five plans, in this order: bare `.mbox` export (13-I1), loose
`.eml` export (13-I2), the `.msg` reader on `cfb` (13-I3, after Pratik confirms the
package), `.msg` through both import commands (13-I4), and the pages that say which of the
three is built and why `.pst` export is not (13-I5).

## Architectural Responsibility Map

| Capability | Primary tier | Secondary tier | Rationale |
|---|---|---|---|
| Deciding what an export is made of, file names, counts, sentences | application (`export_tree`, `importing_messages`) | none | Pure values in and out, tested without a disk, as the existing exporter is (stated: `export_tree.rs:9-11`) |
| Writing bytes to disk (one mailbox file, a tree of `.eml` files) | service (`mailbox_archive`) | none | "the only part of that work that touches the disk" (stated: `mailbox_archive.rs:5-6`) |
| Opening a `.msg` container and reading its property streams | service (a child of `outlook_data_file`) | `cfb` crate | The file is a stranger's; the one place that touches the file turns its values into this program's (stated pattern: `outlook_data_file.rs:947-952`) |
| Choosing which reader a chosen file goes to | application (`import_tree::what_was_chosen`) | none | Decided from the first bytes, never the name (stated: `import_tree.rs:524-547`) |
| Pickers, menu items, the worker, what is announced | presentation (`wx_app.rs`) | accessibility (`announce`, status topics) | The window hands over and says; it does not decide (stated: `wx_app.rs:14421-14435`) |

<phase_requirements>
## Phase Requirements

| ID | Description | Research support |
|---|---|---|
| GAP-13 | Mail export as a bare mbox or loose eml files, msg read, and pst export, or each said plainly to be out. `[D]` Each of the three built or refused with a sentence on the page saying which and why; a refused format is not a menu item. `[S]` A real Outlook file is his. (stated: `.planning/REQUIREMENTS.md:5875-5885`) | Sections 1 to 3 below; plans 13-I1 to 13-I5 |
</phase_requirements>

## Project constraints that bind this group (from CLAUDE.md)

Read in full this session. The ones every plan below has to carry:

- Red, green, refactor on every change; `workflow.tdd_mode` is true; a red commit names its
  failing tests with `Fails-until-green:` and is made on a branch.
- No `unwrap` or `expect` outside tests and `build.rs`; errors through `common::Error`.
- Done means a non-test path reaches it. `writing_out` (below) is the standing example of the
  opposite, in the very module this group extends.
- Untrusted input stays untrusted: a `.msg` is a stranger's file, every size it states is
  written by whoever built it, and nothing it says is believed (the `.pst` reader's rule,
  stated at `outlook_data_file.rs:36-42`).
- "If you expect bug reports from something, that belongs in the product": the `.msg` reader
  says it is new, where the person hears the import's closing sentence.
- Mnemonic letters are one shared set per menu; a plan that hands out a letter owes the
  search. Shortcuts go in `docs/KEYBOARD_SHORTCUTS.md` in the same commit.
- Files written by rule: `docs/changelog.md` for any user-visible change,
  `guards/guards.toml` for a new or re-measured guard, `.planning/WINDOWS.md` for a ledger
  entry, `docs/development/measurements.md` where a figure is taken.
- A plan lists the guard records it touches by both readings: `tests_last_seen` or `file`,
  and records whose `before` text lies inside a region it edits.
- `cargo test` takes one `--lib` filter; several module paths are several runs joined by `&&`.
- Every package is confirmed by Pratik before it is installed.
- No AI attribution in commits, branches, comments or documents.

---

## 1. Bare `.mbox` and loose `.eml` export (#53 point 4)

### What the issue asks (stated)

#53, point 4: "Export only as a zip of `.mbox`; never a bare `.mbox` or loose `.eml`, so one
folder for Thunderbird means unzipping by hand." Pratik's comment of 2026-09-17: "Points 4 to 6
(a bare .mbox or loose .eml export, .msg, .pst export) are later work ... This stays open for
4 to 6." (`gh issue view 53 --comments`, read today.)

### What the tree already has

**The menu and the handlers (stated, read today).**

| What | Where |
|---|---|
| File menu, in full | `src/presentation/wx_app.rs:6661-6728` |
| `&Export Mailbox...`, "Write this folder and everything inside it out to a file", no shortcut | `wx_app.rs:6708-6712` |
| Its dispatch arm | `wx_app.rs:5179-5188` |
| `export_a_mailbox`: refuses with no cache, no account or folder, and anything that is not a real folder (`folder_tree::WhichRow::Folder`), then a Save dialog for `*.zip`, then a worker | `wx_app.rs:15096-15185`; the folder guard at `:15130-15133`; the picker at `:15135-15140` |
| `write_the_mailbox_out`: walks the folder and its subfolders, one message at a time, counting | `wx_app.rs:15196-15294` |
| `an_export_that_broke_off`: the sentence said instead of a count when the file stops partway | `wx_app.rs:15302-15307` |
| `save_the_message_as` and `one_message_saved_to`: Save As, the `.eml` of the message under the cursor | `wx_app.rs:14940-15042`, `:15050-15089` |

**The decisions (stated).**

| What | Where |
|---|---|
| `rebuilt_from_what_is_stored`: a stored row and its text as a `ParsedMessage` | `src/application/export_tree.rs:92-111` |
| `WhatBecameOfIt` and `counted_in`: written, or left out because the text was never downloaded, with files left behind and signatures that could not be kept counted | `export_tree.rs:162-226` |
| `added_to_the_archive`: one message into mailbox-file bytes, a kept signed original written as it arrived | `export_tree.rs:260-297` |
| `one_message_written_out`: one message as a saved-message file, `None` when its text was never downloaded | `export_tree.rs:309-328` |
| `FolderInTheFile` (`stored_at`, `named`) and `where_each_folder_goes`: each folder's path made safe for a file system, told apart ignoring case | `export_tree.rs:471-532`; uniqueness at `:615-635` (private) |
| `FoldersExported` and `what_the_folder_export_did`: the closing sentence with its three extra sentences | `export_tree.rs:641-736` |
| `WritingOut { Refused, OneMessage, AnArchive }`, `the_file_ends_with` answering `.eml` and `.mbox`, and `writing_out(how_many)` | `src/application/importing_messages.rs:550-591` |
| `saving_as`: a subject made into a file name, separators to `_`, then `attachment_name::safe_file_name`, `message` when blank | `importing_messages.rs:622-650` |
| `MessagesExported` and `what_the_mail_export_did` | `importing_messages.rs:665-695` |

**The writer (stated).** `mailbox_archive::written_to` opens the zip at once so a full disk
is found before a long run (`src/service/mailbox_archive.rs:904-917`);
`ArchiveBeingWritten` has `start_a_file`, `add_a_folder_with_nothing_in_it`,
`write_into_it` (refused when no file is started) and `finish` (syncs to disk)
(`mailbox_archive.rs:895-995`). There is no writer for a plain file or for a directory tree.

**Two seams nothing reaches (derived, from a search).**
`grep -rn "writing_out(\|what_the_mail_export_did\|MessagesExported\b" src tests` returned
only definitions, test calls (`importing_messages.rs:1139-1153`, `:1217-1219`, `:1230`), the
`const` uses of `the_file_ends_with` at `export_tree.rs:464` and `importing_messages.rs:658`,
and `FoldersExported`'s field at `export_tree.rs:654`. So `writing_out(how_many)`,
`CHOOSE_THE_MESSAGES_TO_EXPORT` and `what_the_mail_export_did` are production code that no
non-test path calls. 13-I1 is where one of them either gets a caller or goes; a plan that
adds a third export path beside them without deciding is the shape guardrail 1 names.

**A stale sentence in the exporter's own header (derived).** `export_tree.rs:42-44` says the
files a message came with were never kept, "so a message that arrived carrying three of them
exports as the message and none of them". The same file at `:245-251` and the export at
`wx_app.rs:15230-15235` carry the files the computer kept. The header is older than the
attachment store. Correct it in 13-I1's refactor step.

**The import these exports have to round-trip through (stated).** Import a Folder of
Messages opens a `DirDialog` (`wx_app.rs:14480-14501`) and hands the folder to the same
worker; `mailbox_archive::opened` reads a folder or a zip into the same entries
(`mailbox_archive.rs:12-19`, `:189`); `import_tree::what_was_chosen` answers `AnArchive` for a
folder (`src/application/import_tree.rs:536-539`). So a folder of `.eml` files with folders
inside it is exactly what the folder import already reads, which makes the round trip a real
test of both halves.

**The pages (stated).** `docs/USER_GUIDE.md:963-1006` ("Import and Export", a three-row
table); `docs/KEYBOARD_SHORTCUTS.md:613-617` (rows for Save As, Import Mailbox, Import a
Folder of Messages, Export Mailbox, all "(none)"); `docs/changelog.md:1173-1180` (the
gathered "What issue 53 asked for that this build does not do" paragraph);
`docs/comparison.md:98-114` ("Goes out 'the same way' is still generous: export writes one
zip of mbox files, and nothing else").

**Tests and records (stated, counted today).** `grep -c '#\[test\]'`: `export_tree.rs` 38,
`importing_messages.rs` 41, `mailbox_archive.rs` 31. Guard records, read with the TOML
parser (`python -c "import tomllib; ..."` over `guards/guards.toml`, 1,088 records):
`export_tree.rs` is named in `tests_last_seen` by 1 record and as `file` by none;
`importing_messages.rs` 5 and 2; `mailbox_archive.rs` 0 and 0; `tests/wired.rs` 19 and 2;
`wx_app.rs` 119 and 112. Records whose `before` text sits inside the regions these plans edit
(measured with a script that finds each record's `before` in its file and reports the line):
`wx_app.rs:6718`, "a PGP private key somebody can import", anchored on the whole
`.append_item(ID_IMPORT_PGP_KEY, ...)` call, occurs once; `importing_messages.rs:632`, "Save
As takes the path out of a subject before offering it as a name", anchored on
`'/' | '\\' => '_',`. Neither is duplicated by inserting items above the PGP item or by
calling `saving_as`; both are named in the plans below as records to re-read after the edit.

**Existing wiring readings (stated).** `tests/wired.rs:1977`
`test_no_two_items_on_one_menu_claim_the_same_letter` reads every `Menu::builder()` block and
refuses two items on one letter; `tests/wired.rs:3270-3300`
`test_nothing_treats_a_saved_search_as_a_folder_on_a_server` reads `export_a_mailbox` for
`WhichRow::Folder`. The new export handlers must pass the same guard; the cheapest way is a
shared helper both call, with that test reading the helper.

### Proposed plans

**13-I1: One folder out as a bare mailbox file** (S to M, 3 tasks)

- Closes: GAP-13's "bare mbox" line, #53 point 4 first half.
- What it builds: File, `Export Folder as a Mailbox &File...` (letter F, see Accessibility),
  no shortcut. A Save dialog for `*.mbox` offering the folder's own name. The chosen folder
  alone, written as one mailbox file, one message at a time; its subfolders are not in it and
  the closing sentence says so and names Export Mailbox for them. A folder where no message
  went in writes no file and says why (a mailbox file with no messages is one this program's
  own import turns away, stated at `export_tree.rs:241-244`).
- Service: a plain-file writer beside `ArchiveBeingWritten`, opened early, refusing a write
  before it is started, `finish` syncing to disk, and the file removed again when nothing
  went in.
- Application: the closing sentence built from `FoldersExported` and the same three extra
  sentences; decide whether this is `what_the_mail_export_did`'s caller (it is the sentence
  for one archive) or whether that function and `writing_out` go (Pratik question 6).
- Presentation: the handler shares the readiness and folder guard with `export_a_mailbox`
  through one helper, and the worker reuses the per-message loop rather than copying
  `write_the_mailbox_out`.
- Files: `src/application/export_tree.rs`, `src/application/importing_messages.rs`,
  `src/service/mailbox_archive.rs`, `src/presentation/wx_app.rs`,
  `tests/mail_goes_out_in_every_shape.rs` (new, see below), `docs/USER_GUIDE.md`,
  `docs/KEYBOARD_SHORTCUTS.md`, `docs/changelog.md`, `guards/guards.toml`,
  `.planning/WINDOWS.md`.
- Depends on: nothing in this group. Shares the File menu with group 1's Print and group 2's
  key manager, so it is sequenced after them and re-takes the letter search.
- Spoken or shown: yes (a menu item, three sentences). Branch pushed with a pull request.
- The failing test that starts it:
  `application::export_tree::tests::test_one_folder_written_as_a_mailbox_file_reads_back_as_the_same_messages`:
  three stored messages, one never downloaded, one whose body stops mid-line, written through
  the new path into a `tempfile` file, read back with
  `message_files::each_message_read_piece_by_piece`, asserting two messages with their
  subjects and bodies and the count sentence saying one was left out. Then the wiring reading
  in the new test file: the menu item exists, its arm reaches the handler, and the handler
  reaches the writer and the folder guard.
- Why a new test file rather than `tests/wired.rs`: a test added to `wired.rs` changes its
  count, and 19 records name it in `tests_last_seen`, so the count check would ask for 19
  re-measurements. A new file is named by none. The letter check in `wired.rs` still reads
  the new item without changing.

**13-I2: A folder out as saved messages, one file each** (M, 3 tasks)

- Closes: GAP-13's "loose eml" line, #53 point 4 second half.
- What it builds: File, `E&xport Folder as Message Files...` (letter X), no shortcut. A
  folder picker (`DirDialog`, the one Import a Folder of Messages uses at `wx_app.rs:14492`).
  The chosen folder's messages as `.eml` files, each written by `one_message_written_out`
  (so a kept signed original goes out byte for byte, as Save As does), and each subfolder as
  a folder of its own, laid out by `where_each_folder_goes`. The same round trip as the zip:
  what this writes, Import a Folder of Messages reads back.
- Names: from the subject through the same cleaning `saving_as` uses, told apart within one
  folder ignoring case (`one_nothing_else_has_taken`, today private at `export_tree.rs:615`,
  made shared), and never written over a file already there: the writer opens with
  `create_new` and takes the next number when the name is taken on disk. Whether a date goes
  in front of the subject is Pratik question 4.
- A message whose text was never downloaded is left out and counted, as in every export.
- Files: `src/application/export_tree.rs`, `src/application/importing_messages.rs`,
  `src/service/mailbox_archive.rs`, `src/presentation/wx_app.rs`,
  `tests/mail_goes_out_in_every_shape.rs`, `docs/USER_GUIDE.md`,
  `docs/KEYBOARD_SHORTCUTS.md`, `docs/changelog.md`, `guards/guards.toml`,
  `.planning/WINDOWS.md`.
- Depends on: 13-I1 (the shared handler helper, the per-message loop, the sentence).
- Spoken or shown: yes.
- The failing test that starts it:
  `application::export_tree::tests::test_a_folder_written_as_message_files_comes_back_through_the_folder_import`:
  a folder `Work` with a subfolder `Work/Invoices`, two messages sharing one subject and a
  third whose subject is `CON`, written into a `tempfile` directory, then opened with
  `mailbox_archive::opened` and placed with `import_tree::where_the_folders_land`, asserting
  two folders, three messages, two distinct names for the shared subject, and no file named
  as a device. A second red case writes into a directory that already holds `Hello.eml` and
  asserts the old file's bytes are unchanged.

### Pitfalls for these two (derived unless marked)

- **Opening the file before knowing anything goes in.** The zip writer opens early on
  purpose (stated, `mailbox_archive.rs:904-908`). For a bare mailbox file that leaves an
  empty file behind when every message was left out, and an empty file is one this
  program's own import refuses. Open early, and remove it again when nothing went in; or open
  at the first message and check the destination can be created first. Either way, a test.
- **Overwriting in a chosen folder.** A `DirDialog` has no overwrite prompt. `create_new`
  is the only thing that makes "never written over" true; a check-then-write races.
- **Long paths.** 120-character names (`attachment_name.rs:39`, stated) nested several
  folders deep pass Windows' 260-character path limit. Whether `std::fs` on this toolchain
  handles that without the `\\?\` prefix is `[ASSUMED]`; a test with a deep tree settles it
  before the plan relies on it.
- **Line endings.** A message written on its own keeps its body exactly as stored; one in a
  mailbox file gains a final line ending (stated, `export_tree.rs:383-388`). The bare
  mailbox file takes the archive rule, the loose files take Save As's.
- **The saved-search guard.** A handler that forgets `WhichRow::Folder` writes an empty
  file for a saved search and says it worked (stated reason at `wx_app.rs:15122-15129`).

### Accessibility for 13-I1 and 13-I2

- **Controls.** Two native menu items and the two Windows common dialogs already in use
  (Save for I1, the folder picker for I2). No new dialog, so no dialog mnemonics to allocate.
- **Names on both channels.** A native menu item's name is its label on both UI Automation
  and MSAA; the help string is the second argument, which the code at `wx_app.rs:6713-6717`
  says "is what Windows hands a screen reader" (stated). Both items say "(no shortcut)"
  implicitly by carrying none; each description says what it writes and where, in plain
  words: "Write this folder's mail, without the folders inside it, into one mailbox file" and
  "Write this folder's mail as one saved message per file, with a folder for each folder
  inside it". To confirm by ear: both labels and descriptions under NVDA and Narrator
  (ledger lines, the tester's).
- **Focus.** After each modal dialog closes, focus returns to where it was, as after Export
  Mailbox; nothing new is focused. Verify by ear once.
- **What is announced.** The opening sentence at normal priority before the worker starts
  (the pattern at `wx_app.rs:15149-15151`), progress under one status subject so forty
  thousand messages are heard at their latest count rather than forty thousand times
  (stated rule, `wx_app.rs:14693-14695`), and one closing sentence that says what was
  written, what was left out and why, and, for I1, that subfolders were not included.
  Distinct from Export Mailbox's sentence by naming the shape ("into one mailbox file",
  "as 212 message files in 3 folders").
- **Mnemonic letters.** The File menu today, read at `wx_app.rs:6661-6728`: `&New` (N),
  `&Save` (S), `Save &As` (A), `Check &Mail` (M), `Open &Draft` (D), `&Import Mailbox` (I),
  `Import a F&older of Messages` (O), `&Export Mailbox` (E), `Import PGP Private &Key` (K),
  `&Quit` (Q). F is free (the comment at `wx_app.rs:6697-6701` says "nothing here claims F
  now"), and X is free. `E&xport` is also the letter Windows programs conventionally give
  Export. Group 1 (Print, Ctrl+P) and group 2 (the key manager) add items to this menu in
  earlier plans, so the planner re-takes the list the day 13-I1 is written:
  `sed -n '/let file = Menu::builder()/,/\.build();/p' src/presentation/wx_app.rs | grep -o '"[^"]*&[^"]*"'`.
  The `wired.rs` letter test refuses a collision in any case.
- **Shortcuts.** None, for the reason the existing comment gives (`wx_app.rs:6688-6690`):
  done when somebody moves in or out. Two rows in `docs/KEYBOARD_SHORTCUTS.md` with
  "(none)", beside Export Mailbox's row at `:617`.

### What cannot be verified here

- Nobody has opened the zip export, Save As's file or either new file in another mail
  program (ledger 502, stated, covers Save As). Thunderbird Daily is installed on this
  machine (`C:\Program Files\Thunderbird Daily\thunderbird.exe`, checked today), so the
  tester can open a bare `.mbox` there; that is a ledger line per plan, not a test.
- Nothing here touches a server, so there is nothing for phase 14.

---

## 2. Reading Outlook's `.msg` (#53 point 5)

### What the issue asks (stated)

#53, point 5: "`.msg` (Outlook's single-message file): nothing reads or writes it; no Compound
File Binary reader in the tree." GAP-13 asks for `msg read` only. Writing `.msg` is not in the
requirement (derived); the page says so.

### What the tree already has

**Absence, searched.** `grep -rn -i "\.msg\b\|\"msg\"\|D0CF11E0\|0xD0, 0xCF\|compound file\|cfb" src tests docs`
found one unrelated match in `src/data/message_cache/contacts.rs:1896` (the string `"msg"`),
a base64 line in `src/service/signed_mail.rs`, and the unrelated cipher-mode crate `cfb-mode` in
`docs/development/pgp-implementation-choice.md:257`. `grep -n 'name = "cfb"' Cargo.lock`
returned nothing. So there is no Compound File Binary reader and no `.msg` handling.
(stated, commands run today)

**The MAPI property mapping, already written for `.pst` (stated).**
`src/service/outlook_data_file.rs` (3,615 lines, 38 tests) turns the reader's values into its
own at one place and decides everything after that without a data file:

| What | Where |
|---|---|
| `WhatItSaid { Words, Whole, When, YesOrNo, Bytes }` | `outlook_data_file.rs:954-965` |
| `WhatTheItemSaid { said: BTreeMap<u16, WhatItSaid> }` | `:969-971` |
| `TheItem` with `words`, `whole`, `when`, `yes`, `bytes`, `named` | `:978-1042` |
| Property ids: `MESSAGE_CLASS 0x001A`, `SUBJECT 0x0037`, `BODY 0x1000`, `HTML_BODY 0x1013`, `CODE_PAGE_OF_THE_TEXT 0x3FFD`, `SENT_AT 0x0039`, `ARRIVED_AT 0x0E06`, `INTERNET_MESSAGE_ID 0x1035`, `SENDER_NAME 0x0042`, `SENDER_ADDRESS 0x0065`, `SENDER_SMTP_ADDRESS 0x5D02`, `WHO_REALLY_SENT_IT_NAME 0x0C1A`, `WHO_REALLY_SENT_IT_SMTP_ADDRESS 0x5D01`, `RECIPIENT_KIND 0x0C15`, `DISPLAY_NAME 0x3001`, `EMAIL_ADDRESS 0x3003`, `SMTP_ADDRESS 0x39FE`, `WRITTEN_TO 1`, `COPIED_IN 2` | `:794-861` |
| `a_message_from(item, went_to) -> ParsedMessage`: subject without its marker, sender, To and Cc, date, identifiers, both bodies, `attachments: Vec::new()` | `:1712-1749` |
| `who_sent_it`, `one_person` (SMTP address first, the name kept when there is no address) | `:1768-1849` |
| `text_in(bytes, code_page)` for 8-bit text | `:1186` |
| `what_one_property_said`, the one crossing from the reader's values | `:2282-2301` |
| `who_it_went_to`: rows to To and Cc, blind copies dropped on purpose | `:2426-2458` |

These are private. A child module of `outlook_data_file` (for example
`src/service/outlook_data_file/one_saved_message.rs`, declared with `mod` inside the parent)
reaches private items of its parent without widening anything (derived from Rust's privacy
rules, `[ASSUMED]` as a rule statement, trivially checked by the first compile). No guard
record names `outlook_data_file.rs` in `file` or `tests_last_seen` (counted today: 0 and 0),
so moving or sharing its code costs no re-measurement.

**The import routing (stated).** `import_tree::WhatWasChosen { AnArchive, MailInOneFile,
AnOutlookDataFile }` (`import_tree.rs:496-504`); `what_was_chosen` tests the `.pst` signature
`HOW_ONE_BEGINS = b"!BDN"` (`outlook_data_file.rs:107`) before asking whether the bytes are
mail (`import_tree.rs:536-547`); the worker dispatches on all three
(`wx_app.rs:14638-14652`), and `tests/wired.rs:3515`
`test_importing_mail_sends_each_kind_of_file_to_its_own_reader` reads that dispatch. The
Import Mailbox picker's filter is `*.zip;*.eml;*.mbox;*.pst` (`wx_app.rs:14450-14453`). A
`.msg` chosen today falls to `message_files::what_the_file_holds`, which answers `NotMail`, so
`what_was_chosen` answers `AnArchive` and the zip reader refuses it as not an archive (derived
from `import_tree.rs:543-546` and `mailbox_archive.rs:262`).

**The one place imported mail is filed (stated).** `importing_messages::file_one_imported_message`
takes a `MessageFromAFile` and writes the row, its text and, for a signed message, the bytes
it arrived as (`importing_messages.rs:331-387`).

**A finding beside the requirement: imported messages do not keep their files (derived,
needs a test to confirm).** `file_one_imported_message` never calls
`replace_attachments_with_content`; its callers are the sync, `how_it_arrived` tests and two
sites in `wx_app.rs` (`grep -rn replace_attachments_with_content src`, today).
`filing::a_row_filed_here` sets only `has_attachments: !parsed.attachments.is_empty()`
(`src/application/filing.rs:79`). So an `.eml`, `.mbox` or zip import appears to bring a
message's words and a flag saying it had files, and not the files. No page, changelog line or
ledger entry says so (`grep` of the ledger for import with attach, today: nothing). This is
guardrail 9's shape, and it decides what a `.msg` import can promise about attachments. It is
Pratik question 7.

**Records near the routing (stated).** `import_tree.rs:540`, "an Outlook data file is sent
to its own reader rather than the archive's", anchored on the `HOW_ONE_BEGINS` line, occurs
once; `wx_app.rs:14643`, "an Outlook data file the worker recognises is handed to its
reader", anchored on the whole `AnOutlookDataFile` arm, occurs once. A new arm beside each
does not duplicate either anchor; both are re-read after the edit.

### The format (stated from the specification, read today)

- A `.msg` is a Compound File Binary file (it opens with `D0 CF 11 E0 A1 B1 1A E1`
  `[ASSUMED]`, the CFB signature from memory; the probe below opened every sample by
  that route, which is consistent with it).
- Each property of the message has a 16-byte entry in a stream named
  `__properties_version1.0`; fixed-length values sit in the entry, variable-length ones in
  separate streams `[CITED: learn.microsoft.com MS-OXMSG "Property Stream"]`.
- The top-level property stream header is 32 bytes: Reserved (8), Next Recipient ID (4),
  Next Attachment ID (4), Recipient Count (4), Attachment Count (4), Reserved (8)
  `[CITED: learn.microsoft.com MS-OXMSG "Top Level", ed03f930]`.
- A variable-length entry is Property Tag (4), Flags (4), Size (4), Reserved (4); for a
  Unicode string, Size is 2 plus the stream's size; a zero-length string stream is an error
  the reader must report `[CITED: MS-OXMSG "Variable Length Property or Multiple-Valued
  Property Entry", bac41dfb]`.
- The header sizes for recipient and attachment storages (8 bytes) and for an embedded
  message (24 bytes), and the stream naming `__substg1.0_IIIITTTT` with `001F` Unicode,
  `001E` 8-bit and `0102` binary, are `[ASSUMED]` from memory. The names were confirmed
  on the real samples below; the two header sizes were not read on a spec page and the plan
  reads them there before relying on them.

### The probe (measured today, throwaway projects under the scratchpad)

Built with the pinned `rustc 1.98.1 (48a229cea 2026-09-01)` for `x86_64-pc-windows-msvc`,
each with its own `CARGO_TARGET_DIR`, in
`...\scratchpad\phase-13-probes\build-probe` and `...\dump-probe`. Nothing was installed or
built in the repository.

The oracle matrix: one file written by `cfb` itself and the four real `.msg` samples shipped
inside the `msg_parser` 0.3.6 crate (`data/test_email.msg`, `attachment.msg`,
`unicode.msg`, `ascii.msg`; `bad_outlook.msg` is an 11-byte broken file), each opened by
three readers.

| File | `cfb` 0.15.0 | Windows `StgOpenStorageEx` (ole32) | `msg_parser` 0.3.6 |
|---|---|---|---|
| written by `cfb` (subject, body, class, sender name and SMTP address) | opened, 7 entries, every value read back | opened, 7 elements | **refused: "Error parsing file with ole: Unknown node type"** |
| `test_email.msg` (Outlook, Unicode) | opened; `0042` and `0065` hold `marirs@outlook.com` | opened, 50 elements | opened; **sender address empty** although `0065` holds it |
| `attachment.msg` (Exchange sender) | opened; `0065` holds an X500 address, `0042` the name | opened, 91 elements | opened; sender given as the X500 address |
| `unicode.msg` | opened; transport headers `007D001F` present | opened, 45 elements | opened; subject, sender, 2 attachments |
| `ascii.msg` (8-bit strings, `001E`) | opened; every string read | opened, 26 elements | not asked |
| `bad_outlook.msg` (11 bytes) | refused: "Invalid CFB file (11 bytes is too small)" | not asked | refused: "Invalid OLE File" |

The control that makes the refusal count: `cfb` read back the file it wrote and Windows'
own reader opened the same file, so the file is a valid container and `msg_parser`'s refusal
is its defect, on the exact call (`Outlook::from_path`) this program would make. The empty
sender on `test_email.msg` is the second defect on the same path: the existing `one_person`
reads `SENDER_NAME 0x0042` and `SENDER_ADDRESS 0x0065` and would have named the sender.

What the samples also show (stated, from the dump): every one carries a plain-text body
(`1000001F` or `1000001E`) and compressed RTF (`10090102`); none of the four carries an HTML
body stream (`1013`). A message whose only markup is Outlook's compressed RTF therefore
arrives as its plain text, which is what the `.pst` reader already does ("the older markup
Outlook keeps beside the words ... never read at all", `outlook_data_file.rs:2337-2340`).

### Package audit (dependency-audit skill, invoked)

First, what the tree already has: no CFB reader (searched, above); `windows` 0.62.2 is in
the tree with `Win32_System_Com` on (`Cargo.toml:400-403`), and the structured-storage APIs
sit behind one more feature, `Win32_System_Com_StructuredStorage`, which the probe needed
together with `Win32_Security` (already on) (stated:
`windows-0.62.2/Cargo.toml:573`, `StgOpenStorageEx` at
`src/Windows/Win32/System/Com/StructuredStorage/mod.rs:944`). `outlook-pst` 1.2.0 reads
`.pst` only.

Lock cost was measured the way the skill asks: the project's own `Cargo.toml`, `Cargo.lock`
and `rust-toolchain.toml` copied to a scratch directory, the candidate added under
`[dependencies]`, `cargo metadata` run, and the two lockfiles diffed by `(name, version)`
with a Python `tomllib` script (`...\phase-13-probes\lockdiff.py`). The control: the same
script reported the one added package in every run, so it does register a change. The lock
held 723 packages before.

**`cfb` 0.15.0: recommended.**

```
Dependency: cfb@0.15.0
Purpose: open the Compound File Binary container a .msg is, and read its streams; also
  writes one, which lets tests build .msg fixtures (the .pst reader could never have any)
Size: 1 new lock entry (cfb). Build graph on x86_64-pc-windows-msvc: 2 new packages, cfb and
  web-time 1.1.0, which is in the lock today only through quinn, quinn-proto and
  rustls-pki-types and is not built for Windows now (cargo tree -e normal --target
  x86_64-pc-windows-msvc -i web-time: only cfb). fnv 1.0.7 and uuid 1.24.0 are already built.
Maintenance: 22 versions since 2017-03-20; 0.15.0 published 2026-09-18; last commit
  2026-09-20 (a performance change merged by the owner); contributors mdsteele 170,
  sftse 37, francisdb 35, ikrivosheev 12; 63 stars; 4 open issues and pull requests, none
  on the reading path (#82 raw timestamps, #48 a question, #38 mutability, #13 async);
  64.8 million downloads, 1.69 million a week.
License: MIT, and the LICENSE file ships in the published crate (listed today).
Safety: no `unsafe` in src (grep count 0); a directory loop is refused ("loop in tree",
  src/internal/directory.rs:191); a sector pointed to twice is refused
  (src/internal/alloc.rs:181). MSRV 1.74, below this project's 1.88, so the floor does not
  move and clippy's MSRV-driven lints do not change.
Advisories: none. ~/.cargo/advisory-db (updated 2026-09-24) has no directory for cfb;
  cargo audit on the probe lock, with no suppressions, reported the same three advisories
  (RUSTSEC-2023-0071 rsa, RUSTSEC-2026-0194 and -0195 quick-xml) as on every other probe, the
  project's existing ones, so cfb adds none. Control: the same database holds directories
  for rsa and quick-xml.
Builds: yes, release, x86_64-pc-windows-msvc, rustc 1.98.1, in the probe (11.3 s).
Legitimacy check: OK (gsd-tools package-legitimacy, today). No build script.
Alternatives: Windows' structured storage (below); msg_parser (below); writing a CFB reader
  (header, DIFAT, FAT, MiniFAT, the directory tree, stream chains, all hardened against a
  stranger's file: several hundred lines of the parser this project most wants to not own).
Defaults overlapping rules we already hold: none found; cfb::open is lenient, open_strict
  exists (lib.rs:425); the plan decides which, with a test on a sample.
Measured unchanged: not measured (nothing was installed in the repository).
Recommendation: ADD, on Pratik's confirmation, in 13-I3 task 1.
```

**Windows `StgOpenStorageEx` / `IStorage`: the named alternative, not recommended.**

```
Dependency: none new; one feature (Win32_System_Com_StructuredStorage) on windows 0.62.2
Purpose: the same container read, by Windows' own ole32
Size: 0 packages; more of an already-vendored crate compiled
Maintenance: Windows; patched by Windows Update
License: already accepted (windows crate, MIT OR Apache-2.0 [ASSUMED from memory])
Advisories: none for the crate; ole32's own history is Microsoft's
Builds: yes, the probe (3.3 s incremental)
Against it: unsafe COM calls on every read; COM initialised on the import worker's thread;
  opens by path, so a .msg inside a zip or read piece by piece needs StgOpenStorageOnILockBytes
  over an in-memory ILockBytes, more COM; Windows-only, so tests and the reader sit behind
  cfg(windows) with a refusal elsewhere; fixtures would be written through StgCreateStorageEx
  in unsafe test code.
For it: no new maintainer to trust; the parser of a stranger's file is the operating
  system's.
Recommendation: SKIP unless Pratik prefers no new package (question 3).
```

**`msg_parser` 0.3.6: not recommended.**

```
Dependency: msg_parser@0.3.6
Purpose: a ready-made .msg reader with its own OLE reader
Size: 1 new lock entry; hex, regex, serde, serde_json, thiserror already built
Maintenance: one maintainer (marirs, 51 of 57 commits); 13 versions, six of them on
  2026-04-14 and 15; last push 2026-05-24; 10 stars; 2 open issues
License: MIT, LICENSE ships
Advisories: none (same audit as above)
Builds: yes (probe)
Against it: refused a valid container on the path this program would call; returned an
  empty sender where the file names one; flattens every value to a string (times as ISO
  text, named properties as a string map), which throws away the typed mapping this tree
  already has; reads the whole file into memory up to its own 256 MiB cap
  (src/ole/constants.rs:18).
Recommendation: SKIP.
```

**`tiny_msg` 0.2.0** (one publisher, 1,204 downloads, no repository, depends on the older
`cfb` 0.10): not audited further, SKIP. **`compressed-rtf` 1.0.1** (Microsoft's, same
repository as `outlook-pst`, adds 1 lock entry, ships no LICENSE text, as `outlook-pst` does
not either): only needed to read RTF-only markup, which is out of scope here and deferred.

**The fixtures question.** The four real samples above sit in the `msg_parser` crate under
MIT, but one says it was made "to experiment with the MS Outlook MSG Extractor" and one "by
Aspose.Email", so where they came from and under what terms is `[ASSUMED]`-unknown. Recommend
not vendoring them: tests build `.msg` fixtures with `cfb`'s writer, as the probe did, and the
real-file proof is Pratik's (question 8).

### Proposed plans

**13-I3: The `.msg` reader** (M, 3 tasks; the first a `checkpoint:human-verify`)

- Closes: GAP-13's "msg read" line, the reading half; #53 point 5.
- Task 1 is Pratik's confirmation of `cfb` 0.15.0 (the audit above), then the manifest line
  with its comment, in a commit of its own ahead of any red test, and the lock checked to
  have gained exactly one entry. `grep -rln 'Cargo.toml' tests scripts` first, for any
  census that reads the manifest (the skill's question 7; not run today, the executor runs
  it).
- Task 2, red then green: a child module of `outlook_data_file` that opens a container from
  anything that reads and seeks, refuses one that is not a message (no
  `__properties_version1.0`, or a class that is not mail), reads the fixed-length entries and
  the string and binary streams it names into `WhatTheItemSaid`, reads each
  `__recip_version1.0_#` storage into To and Cc through the same rule `who_it_went_to`
  applies, and hands back a `ParsedMessage` from `a_message_from`, so a `.msg` and a `.pst`
  message cannot come to be read two ways. Every stream read is bounded by a limit in the
  spirit of `outlook_data_file::HowMuchToAllow`, counting what really came out.
- Task 3: what stays behind is counted, not dropped: attachments (unless question 7 says
  otherwise), RTF-only markup, blind copies, and a file that is an appointment, contact,
  task or note (question 2). The refusal sentences, in plain words, one per cause: not an
  Outlook message, damaged partway, more than this program will read.
- Files: `Cargo.toml`, `Cargo.lock`, `src/service/outlook_data_file.rs`,
  `src/service/outlook_data_file/one_saved_message.rs` (new), `docs/changelog.md` (the
  dependency line only if a user-visible change lands here; otherwise 13-I4 carries it),
  `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: Pratik's confirmation of the package. Nothing else in this group.
- Spoken or shown: no; nothing reaches a person until 13-I4.
- The failing test that starts it:
  `service::outlook_data_file::one_saved_message::tests::test_a_saved_outlook_message_reads_as_the_message_it_holds`:
  a fixture written with `cfb` holding a Unicode subject, a plain body, a sender name with an
  SMTP address, one To and one Cc recipient storage and a `SENT_AT` time in the fixed-length
  stream, read back to a `ParsedMessage` with each field asserted. Beside it, one case per
  refusal, an 8-bit fixture with a code page (`ascii.msg`'s shape), an Exchange sender with
  no SMTP address (`attachment.msg`'s shape: the name and the X500 address kept, as
  `one_person` does), and a stream larger than the limit.

**13-I4: `.msg` through both import commands** (M, 3 tasks)

- Closes: GAP-13's "msg read", the reaching half.
- What it builds: `WhatWasChosen::AnOutlookMessage` from the first eight bytes, tested before
  the mail question as the `.pst` signature is; the worker's arm reading the chosen file and
  filing through `file_one_imported_message` under Imported, like a chosen `.eml`
  (`one_file_of_mail_brought_in`, `wx_app.rs:15335-15373`); the picker's filter gains
  `*.msg`; inside a chosen folder or zip, an entry that begins like a container goes to the
  same reader instead of being counted as not mail, read whole through
  `one_entry_read_through` under `most_one_thing_unpacks_to` (`mailbox_archive.rs:40-48`,
  stated), which is the case of somebody who dragged a hundred messages out of Outlook into a
  folder. The closing sentence says the `.msg` reading is new and has read only test files,
  and counts what stayed behind.
- Files: `src/application/import_tree.rs`, `src/application/importing_messages.rs`,
  `src/presentation/wx_app.rs`, `tests/mail_goes_out_in_every_shape.rs` (or its own new
  file), `docs/USER_GUIDE.md`, `docs/KEYBOARD_SHORTCUTS.md` (the Import Mailbox and Import a
  Folder rows name `.msg`), `docs/changelog.md`, `docs/comparison.md`, `guards/guards.toml`,
  `.planning/WINDOWS.md`.
- Depends on: 13-I3.
- Spoken or shown: yes (the picker's filter text, the closing sentence, the pages in Help).
- The failing test that starts it:
  `application::import_tree::tests::test_a_saved_outlook_message_goes_to_its_own_reader`,
  then a real-database test in `importing_messages` that files a `cfb`-built fixture and
  reads back one row with its subject and text; then the wiring reading, the counterpart of
  `wired.rs:3515`, that the worker dispatches on the fourth answer. Adding the fourth answer
  to `wired.rs:3515`'s list instead is a body change (count unchanged) and re-reads the
  record at `wx_app.rs:14643`.

### Accessibility for 13-I3 and 13-I4

- No new control. The Import Mailbox picker's filter label becomes "Mailboxes, saved
  messages and Outlook files (`*.zip;*.eml;*.mbox;*.msg;*.pst`)"; the Save/Open common
  dialog reads the "Files of type" list on both channels natively. Verify by ear once.
- What is announced: the existing opening sentence, the existing progress line, and a
  closing sentence that separates what arrived from what stayed behind, each cause its own
  sentence (the `.pst` import's pattern). "Experimental" is said once, at the end, not per
  message.
- No mnemonic or shortcut changes. The Import Mailbox row in `docs/KEYBOARD_SHORTCUTS.md`
  (`:615`) and the guide's table (`docs/USER_GUIDE.md:972`) name `.msg`.

### What cannot be verified here, and what cannot be finished

- No Outlook on this machine (no `OUTLOOK.EXE` under either Office path; the mail clients
  registered are Hotmail and Thunderbird, checked today), so no `.msg` this program reads
  can come from a known Outlook here. The probe read four real Outlook-written files with
  the container reader; the property mapping has read none. A ledger `unrun-verify` line,
  the tester's, with the `.msg` he saves (question 8).
- A signed `.msg` keeps its signed MIME inside an attachment `[ASSUMED]`, so rebuilding the
  message from properties loses the signature; counted and said, not verified.
- Nothing touches a server; nothing for phase 14.

---

## 3. Writing a `.pst` (#53 point 6)

### What the issue asks (stated)

#53, point 6: "`.pst` export: no writer, by a recorded decision (`Cargo.toml:249-251`)." The
recorded decision, read today at `Cargo.toml:242-253`: the reader is "Microsoft's own
(github.com/microsoft/outlook-pst-rs), which is the reason for choosing it over the fork that
also writes ... only reading is needed. Reading is also the whole of the risk ... Nothing here
writes one."

### Every route, looked for

| Route | What was found | Verdict |
|---|---|---|
| A Rust writer | `outlook-pst-rw` 1.2.9, "MIT-licensed fork of Microsoft's `outlook-pst` crate ... adds narrowly scoped creation and incremental append support for Unicode PST files" (its README, read today). Created 2026-08-09, ten versions in four days, last 2026-08-12; 2,148 downloads in all; the repository `dongs0104/outlook-pst-rs` is a fork with 0 stars, 18 commits by its one author, merged from branches named `agent/...`. Publishes its library under the same name, `outlook_pst`, as the reader already in the tree (`[lib] name = "outlook_pst"` in its manifest), so both could only coexist renamed. ANSI files, encrypted files and reclaiming space are unsupported (README). Ships no LICENSE text. Lock cost 1 entry; builds its dependencies from what is already here. Legitimacy check: **SUS** (low downloads). No advisory. | Not recommended. It is, in all likelihood, the fork `Cargo.toml` already declined (derived: that comment is older than 2026-09-05 and this fork is the only writing one on crates.io). Its output has not been opened in Outlook by anyone this project can point to, and cannot be here. |
| Windows, Extended MAPI | Creating a `.pst` goes through MAPI's Personal Folders store provider; "To correctly install the MAPI subsystem, install an application that contains a MAPI-based subsystem, such as Microsoft Outlook" `[CITED: learn.microsoft.com/office/client-developer/outlook/mapi/installing-the-mapi-subsystem]`. `C:\Windows\System32\mapi32.dll` exists here, and no Outlook does. | Not a route: it works only where Outlook is installed, and the person exporting a `.pst` is usually leaving it or has left. |
| Outlook automation, or a commercial MAPI library | Needs Outlook, or a paid licence | Out, same reason, and a licence decision nobody has asked for. |
| Writing it ourselves from MS-PST | The format is public (MS-PST); the fork's writer is about 4,900 lines on top of a 16,700-line reader (derived: `find src -name "*.rs" | xargs cat | wc -l`, 21,601 against 16,735, today), with B-trees, allocation maps and CRCs to get right | Out: many sittings, and a file only Outlook can judge. |

**Recommendation: `.pst` export said plainly to be out.** Not a menu item. The guide, the
comparison page and the changelog say it is not written and why, and what to do instead
(Pratik question 1 has the sentence). The upstream reader has four open pull requests about
real-world files (#61 "Fix 2 panics", #62 "tolerate real-world archive quirks", #63 "accept
omitted HNPAGEMAP free counts", #65 embedded-message attachments; read today with
`gh api repos/microsoft/outlook-pst-rs/pulls/N`). They concern the `.pst` import that already
shipped, not this group, and #63 names a public real `.pst` (the EDRM Enron set) that could
close ledger 499 without Pratik's own file. Noted here for whoever owns ledger 499.

### Proposed plan

**13-I5: The pages say which of the three is built, and why `.pst` is not** (S, 2 tasks)

- Closes: GAP-13's `[D]` line ("each of the three built or refused with a sentence on the page
  saying which and why; a refused format is not a menu item"), and #53 on the tree side.
- Task 1, the house-style targets that read documents first: the guide's Import and Export
  section gains the two export rows, `.msg` in the import rows, and a short paragraph: what
  goes out in which shape, that `.msg` is read and not written, and that `.pst` is neither
  written nor on any menu, with the reason in one sentence. The comparison page's "still
  generous" sentence corrected with a date. The changelog's gathered paragraph at
  `docs/changelog.md:1173-1180` rewritten to say what was built and what is out.
- Task 2: the ledger closes or re-words 502 and opens `unrun-verify` lines for the new
  shapes; `gh issue` comment drafted for Pratik, not posted (publishing happens on purpose).
- Files: `docs/USER_GUIDE.md`, `docs/comparison.md`, `docs/changelog.md`,
  `docs/KEYBOARD_SHORTCUTS.md` (if 13-I1 to 13-I4 left anything), `.planning/WINDOWS.md`.
- Depends on: 13-I1, 13-I2, 13-I4.
- Spoken or shown: yes (the guide and the changelog are Help topics, `src/application/help.rs:66-107`).
- TDD: documentation, one of the listed exceptions; the document-reading targets still run.
  If the page gains a sentence a test should hold (for example, that no menu item names
  `.pst` as an export), that test is red first.
- If this group is the phase's last, the closing plan also carries `scripts/check.sh all`
  by hand before its merge (CLAUDE.md, "A phase's closing plan runs the full gate once").

---

## Plan summary

| Plan | Title | Closes | Depends on | Spoken or shown | Size | Starting red test |
|---|---|---|---|---|---|---|
| 13-I1 | One folder out as a bare mailbox file | GAP-13 bare mbox; #53.4 | group 1 and 2 File menu plans (ordering only) | yes | S to M | `export_tree::tests::test_one_folder_written_as_a_mailbox_file_reads_back_as_the_same_messages` |
| 13-I2 | A folder out as saved messages, one file each | GAP-13 loose eml; #53.4 | 13-I1 | yes | M | `export_tree::tests::test_a_folder_written_as_message_files_comes_back_through_the_folder_import` |
| 13-I3 | The `.msg` reader | GAP-13 msg read (reading); #53.5 | Pratik confirms `cfb` | no | M | `outlook_data_file::one_saved_message::tests::test_a_saved_outlook_message_reads_as_the_message_it_holds` |
| 13-I4 | `.msg` through both import commands | GAP-13 msg read (reaching) | 13-I3 | yes | M | `import_tree::tests::test_a_saved_outlook_message_goes_to_its_own_reader` |
| 13-I5 | The pages say which is built and why `.pst` is not | GAP-13 `[D]`; #53.6 | 13-I1, 13-I2, 13-I4 | yes | S | documentation (exception); a page reading if a sentence is to be held |

Optional, only if Pratik says yes to question 7: **13-I6, imported messages keep their files**
(M): `file_one_imported_message` stores each attachment's content through
`replace_attachments_with_content`, for `.eml`, `.mbox`, zip, folder and `.msg` alike, with a
real-database red test that imports a message carrying a file and reads the file back.

Every plan writes `docs/changelog.md` and `.planning/WINDOWS.md`, and every plan but 13-I5
writes `guards/guards.toml`, so no two share a wave. `docs/development/measurements.md` is written only if a plan takes a figure
(none planned).

## Package legitimacy audit

| Package | Registry | Age | Downloads | Source repo | Verdict | Disposition |
|---|---|---|---|---|---|---|
| `cfb` 0.15.0 | crates.io | 9 years (2017-03-20) | 1.69 M a week | github.com/mdsteele/rust-cfb | OK | Recommended; 13-I3 task 1 waits for Pratik |
| `msg_parser` 0.3.6 | crates.io | 5 years | 9.7 k a week | github.com/marirs/msg-parser-rs | OK | Not recommended (defects measured above) |
| `outlook-pst-rw` 1.2.9 | crates.io | 6 weeks | 167 a week | github.com/dongs0104/outlook-pst-rs (fork) | SUS | Not recommended; `.pst` export out |
| `compressed-rtf` 1.0.1 | crates.io | 1.5 years | 3.5 k a week | github.com/microsoft/outlook-pst-rs | OK | Deferred; not needed by these plans |

Packages removed as SLOP: none. Flagged SUS: `outlook-pst-rw`, which no plan installs. None
of the four has a build script.

## Environment availability

| Dependency | Needed by | Available | Version | Fallback |
|---|---|---|---|---|
| Rust toolchain, pinned | every plan | yes | 1.98.1 | none needed |
| `cfb` from crates.io | 13-I3 | yes (resolved and built in the probe) | 0.15.0 | Windows structured storage |
| Thunderbird, to open an exported `.mbox` by hand | 13-I1, 13-I2 manual lines | yes, Thunderbird Daily | not read | the tester's own mail program |
| Outlook, for `.msg` samples and any `.pst` check | 13-I4 manual line; `.pst` | no | none | Pratik's own Outlook elsewhere, or none |

## Validation architecture

| Property | Value |
|---|---|
| Framework | `cargo test`, unit tests beside the code, `tempfile` for disk |
| Quick run, per plan | `cargo test --lib application::export_tree::` and, joined with `&&`, `cargo test --lib service::outlook_data_file::` or `cargo test --lib application::import_tree::` as the plan needs, then `cargo test --test mail_goes_out_in_every_shape` |
| Full suite | `bash scripts/check.sh all`, once, in the phase's closing plan |

Wave 0 gaps: the new test file `tests/mail_goes_out_in_every_shape.rs` (13-I1 creates it);
a `cfb`-built fixture helper inside the reader's test module (13-I3). Each new red test
added to `export_tree.rs`, `importing_messages.rs` or `import_tree.rs` changes a count the
records at 1, 5 and 1 name; the count check prints the `scripts/guards.sh --remeasure`
command, and running it is not optional.

## Security domain

| ASVS area | Applies | Control |
|---|---|---|
| V5 input validation | yes | a `.msg` is a stranger's file: every size bounded by what really came out, `cfb`'s own refusals of looped and doubly-pointed chains, one sentence per refusal |
| V12 files | yes | file names from subjects through `safe_file_name` (devices, separators, bidirectional overrides, 120 characters); `create_new` so nothing is overwritten; subfolder names through `where_each_folder_goes` |
| V6 cryptography | no | a signed message is written as it arrived where kept; nothing is signed or encrypted here |
| V2, V3, V4 | no | nothing reaches a server |

| Threat | STRIDE | Mitigation |
|---|---|---|
| A crafted `.msg` that loops or claims a vast stream | Denial of service | `cfb` refusals; per-stream and per-file limits counted on real bytes |
| A subject that names a path or a device | Tampering | `saving_as`'s separator rule, then `safe_file_name` |
| An export that silently overwrites a person's files | Tampering | `create_new`; numbered names |
| An export that reads as complete and is not | Repudiation (of the backup) | the left-out counts and sentences the zip export already says |

## Questions only Pratik can answer

1. **Outlook data file export (`.pst`).** Nothing can write one here without Outlook, and the
   one Rust library that tries is six weeks old, written by one person, and is the fork your
   `Cargo.toml` already turned down. Recommendation: say plainly it is out. The guide would
   read: "Wixen Mail does not write Outlook data files (`.pst`). Only Outlook can check one,
   and the one library that writes them is too new to trust with your mail. To take mail to
   Outlook, export a folder as message files and drag them into Outlook, or let Outlook
   download the same account." Is that right, and is the drag-in advice true for the Outlook
   you use? (`[ASSUMED]` that classic Outlook takes dragged `.eml` files.)
2. **Which `.msg` files to read.** Mail only, or also appointments, contacts, tasks and notes
   saved as `.msg`? Recommendation: mail only now; the others are refused by name ("That is
   an Outlook appointment, not a message. Wixen Mail reads saved messages from `.msg` files").
   The other kinds need Outlook's named properties, a second reader of its own.
3. **The package.** Adding `cfb` (one new library, well used, MIT, no unsafe code), or using
   Windows' own reader with no new library but more unsafe code? Recommendation: `cfb`.
4. **File names for message files.** Subject only ("Hello.eml"), or the date first
   ("2026-09-24 Hello.eml") so a folder listing reads in the order the mail came?
   Recommendation: date first.
5. **The bare mailbox file.** This folder alone, with a sentence saying the folders inside it
   are left out (recommendation), or each folder as its own `.mbox` file in a folder you
   choose?
6. **The two new menu items.** "Export Folder as a Mailbox File" (Alt+F then F) and "Export
   Folder as Message Files" (Alt+F then X), next to Export Mailbox. Or one Export submenu
   with the three shapes in it, which changes where the existing Export Mailbox is?
   Recommendation: two items, nothing you have learned moves. And the old unused decision
   that picked `.eml` for one selected message and `.mbox` for several: use it for exporting
   selected messages later, or remove it? Recommendation: remove it in 13-I1, since the two
   new items cover what it was for.
7. **Imported mail loses its attached files** (found while researching, needs one test to
   confirm). A message imported from `.eml`, `.mbox`, a zip or a folder appears to arrive
   with its words and a mark saying it had files, and without the files, and nothing says
   so. Fix it in this group as its own plan (recommendation), or record it for later? The
   answer also decides whether a `.msg`'s attachments come in or are counted as left behind.
8. **A real `.msg`.** Can you save two or three messages from Outlook (one with an
   attachment, one from a work address) for the by-hand check? The tests will build their
   own files; the ones inside another library carry notes saying they came from other
   projects, so I would not copy them in.

## Assumptions log

| # | Claim | Section | Risk if wrong |
|---|---|---|---|
| A1 | The CFB signature is `D0 CF 11 E0 A1 B1 1A E1` | 2, format | The router misses `.msg`; caught by the first fixture test |
| A2 | Recipient and attachment property-stream headers are 8 bytes, an embedded message's 24 | 2, format | Fixed-length values read at the wrong offset; read the spec page before task 2 |
| A3 | A child module reaches its parent's private items | 2, tree | None; the first compile says |
| A4 | `std::fs` on this toolchain writes past 260 characters without help | 1, pitfalls | Deep exports fail; a test settles it |
| A5 | A signed `.msg` keeps its MIME in an attachment | 2, cannot verify | The signature sentence is wrong; say "not kept" without a cause |
| A6 | Classic Outlook accepts dragged `.eml` files | 3, question 1 | The advice on the page is wrong; Pratik confirms |
| A7 | `windows` crate licence is MIT OR Apache-2.0 | 2, audit | None for the recommendation |
| A8 | The four samples' origins are other projects | 2, fixtures | Only matters if they were copied in, which is not recommended |

## Sources

- In the repository, read this session: `CLAUDE.md`; `.planning/ROADMAP.md:1463-1513`;
  `.planning/REQUIREMENTS.md:5875-5885`; the phase 12 README and `12-08-PLAN.md`; the files
  and lines cited above; `.planning/WINDOWS.md` entries 97, 499, 501, 502, 503, 511.
- GitHub: `gh issue view 53 --comments`; `gh api` for mdsteele/rust-cfb,
  marirs/msg-parser-rs, dongs0104/outlook-pst-rs, microsoft/outlook-pst-rs (activity,
  contributors, open issues and pull requests 61, 62, 63, 65, issue 60).
- crates.io API for `cfb`, `msg_parser`, `tiny_msg`, `outlook-pst-rw`, `outlook-pst`,
  `compressed-rtf`; the published crate files under `~/.cargo/registry/src`.
- [MS-OXMSG: Property Stream](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/20c1125f-043d-42d9-b1dc-cb9b7e5198ef)
- [MS-OXMSG: Top Level](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/ed03f930-fdef-4cc8-b400-b69007d7f416)
- [MS-OXMSG: Variable Length Property or Multiple-Valued Property Entry](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/bac41dfb-c824-4e3c-9b5e-b61106f6739f)
- [Installing the MAPI Subsystem](https://learn.microsoft.com/en-us/office/client-developer/outlook/mapi/installing-the-mapi-subsystem)
- Probes: `C:\Users\prati\AppData\Local\Temp\claude\C--Users-prati-Documents-projects-Wixen-Mail\ee5f65c6-033f-4d20-9966-4517170e1c1f\scratchpad\phase-13-probes\`
  (`build-probe`, `dump-probe`, `lock-*`, `lockdiff.py`).

**Valid until:** the tree moves under these files, or 2026-10-24 for the package figures.

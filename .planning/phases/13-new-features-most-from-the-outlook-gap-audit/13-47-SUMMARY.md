---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 47
subsystem: import
tags: [msg, outlook, cfb, import, gap-13]
status: complete
requires: [13-46]
provides:
  - "service::outlook_data_file::one_saved_message::{read, SavedOutlookMessage, LeftInTheFile, WhyItWasNotRead, HOW_A_SAVED_MESSAGE_BEGINS}"
  - "service::outlook_data_file::went_to_from, the recipient rule both Outlook readers share"
affects: [13-48, 13-49, 13-50]
tech-stack:
  added: ["cfb 0.15.0"]
  patterns: ["a child module reaching its parent's private mapping through super::", "every stream read through take() against what may still come out"]
key-files:
  created:
    - src/service/outlook_data_file/one_saved_message.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/service/outward.rs
    - src/service/outlook_data_file.rs
    - guards/guards.toml
    - .planning/WINDOWS.md
decisions:
  - "Answer (a) received: cfb 0.15.0, confirmed by Pratik on 2026-09-24 (phase README decision 52, his words: \"Yes to all questions.\")."
  - "The lenient cfb::CompoundFile::open, because its loop and doubly pointed sector refusals are unconditional."
  - "Markup is counted as kept only in Outlook's format when the compressed RTF is there and no web-page markup is, whatever the plain words."
metrics:
  duration: "about 2 hours 40 minutes"
  completed: 2026-10-03
estimate:
  tokens: 110000
actuals:
  tokens: 22000    # chars/4: 62,198 added in the code commits, about 25,000 in the documents
  tasks: 4
  commits: 6
---

# Phase 13 Plan 47: The `.msg` reader Summary

Answer (a) received: `cfb` 0.15.0, confirmed by Pratik on 2026-09-24 ("Yes to all questions.",
phase README decision 52). A saved Outlook message is now read through the `.pst` reader's own
mapping into the bytes of one message, with its files, and a count of what stays in the file.
Nothing a person does reaches it yet; 13-48 wires it to both import commands.

## What works now

`service::outlook_data_file::one_saved_message::read` takes anything that reads and seeks and
answers a `SavedOutlookMessage { mail, left_in_the_file }` or a `WhyItWasNotRead`, each refusal
with its own sentence. The class is sorted by the parent's `the_kind_of`, the sender by
`who_sent_it` and `one_person`, the recipients by `went_to_from`, the message by `a_message_from`
and `message_files::written_as_one_message`, so a `.msg` and a `.pst` message cannot come to be
read two ways.

Read by hand on 2026-10-03 against the four real Outlook samples in `msg_parser` 0.3.6's crate,
through a test added for the run and removed before any commit (the samples are not in the tree):

| Sample | Subject | Sender | To, Cc | Files brought | Left in the file |
|---|---|---|---|---|---|
| `test_email.msg` | Test Email | `marirs@outlook.com` | 1, 2 | 2 (a jpg and a `.msg` held as bytes) | 1 file not brought (an embedded message), 3 recipients left off, markup only in Outlook's format |
| `attachment.msg` | FW: ... | Exchange name and X500 address kept | 1, 0 | 3 (doc, png, jpg, with their types) | markup only in Outlook's format |
| `unicode.msg` | Test for TIF files | Brian Zhou, `brizhou@gmail.com` | 1, 1 | 2 tiffs | markup only in Outlook's format |
| `ascii.msg` (one-byte text) | creating an outlook message file | `from@domain.com` | 1, 0 | none | markup only in Outlook's format |
| `bad_outlook.msg` (11 bytes) | refused: not a saved Outlook message | | | | |

`msg_parser` gave an empty sender for `test_email.msg` in the research's probe; this reader names it.

## Premise 5, settled

MS-OXMSG on learn.microsoft.com, read 2026-10-03:

- 2.4.1.2, Embedded Message object Storage: "Reserved (8 bytes) ... Next Recipient ID (4 bytes)
  ... Next Attachment ID (4 bytes) ... Recipient Count (4 bytes) ... Attachment Count (4 bytes)",
  24 bytes.
- 2.4.1.3, Attachment Object Storage or Recipient Object Storage: "Reserved (8 bytes): This field
  MUST be set to zero when writing a .msg file and MUST be ignored when reading", 8 bytes.
- 2.4.1.1, Top Level: the same four counts between two 8-byte reserved fields, 32 bytes.

The probe the research named (`phase-13-probes/dump-probe`) had lost its source, so a new one was
built in the scratchpad against `cfb` 0.15.0. Each `__properties_version1.0` length modulo 16:
the top level 0 in all four samples; every recipient and attachment storage 8 (six, one, two and
one recipients, three, three and two attachments); and the embedded message inside
`test_email.msg` 8, which is 24 modulo 16. Spec and samples agree with A2, 8 and 24.

A1, the signature: `D0 CF 11 E0 A1 B1 1A E1`, `cfb`'s own `MAGIC_NUMBER`
(`src/internal/consts.rs:8-9`).

**Which open.** `cfb`'s loop refusal (`src/internal/directory.rs:190-191`, `malformed!("loop in
tree")`) and its doubly pointed sector refusal (`src/internal/alloc.rs:180-181`,
`malformed!("sector {} pointed to twice", ...)`) sit outside every `validation.is_strict()` test,
so both run under the lenient `open` (`src/lib.rs:416`) as well as `open_strict` (`:425`). The
reader uses `CompoundFile::open`.

## Attachment property ids, each read on its page (task 4)

| Property | Id | Page |
|---|---|---|
| `PidTagAttachMethod` | `0x3705`, PtypInteger32 | MS-OXPROPS 2.601; values in MS-OXCMSG 2.2.2.9 (afByValue 1, afByReference 2, afByReferenceOnly 4, afEmbeddedMessage 5, afStorage 6, afByWebReference 7) |
| `PidTagAttachLongFilename` | `0x3707`, PtypString | MS-OXPROPS 2.595 |
| `PidTagAttachFilename` | `0x3704`, PtypString | MS-OXPROPS 2.593 |
| `PidTagAttachMimeTag` | `0x370E`, PtypString | MS-OXPROPS 2.602 |
| `PidTagAttachDataBinary` | `0x3701`, PtypBinary | MS-OXPROPS 2.589 |
| `PidTagRtfCompressed` | `0x1009`, PtypBinary | MS-OXPROPS 2.943 |

The value types are MS-OXCDATA 2.11.1's.

## Task 2, the package

- `which-checks.sh` on the staged three paths with the message file: `all`. The hook's mode line:
  `check.sh: mode all`, and `check.sh: all passed after 1171 s`.
- `cargo audit` in that run: "No advisory outside .cargo/audit.toml, and nothing is being held
  open."
- `grep -c '^\[\[package\]\]' Cargo.lock`: 723 before, 724 after; the one new entry is `cfb`.
  One other line in the lock moved: `snafu-derive` 0.9.2's `heck` went from 0.4.1 to 0.5.0. Both
  were already locked, `snafu-derive` asks for `">= 0.4, < 0.6"`, and 0.4.1 stays for
  `ouroboros_macro`, so no package came or went; cargo re-resolved it when the manifest changed.
- `A_CRATE_THAT_CANNOT`: 55 to 56.
- The commit holds exactly `Cargo.toml`, `Cargo.lock` and `src/service/outward.rs`, and is the
  first on the branch to touch the manifest.

## Commits

| Commit | What | Hook |
|---|---|---|
| `8a18b688` | build: cfb 0.15.0, the census entry, the lock | all, 1171 s |
| `962499be` | test: ten reader cases and the shared-rule case | red, 199 s |
| `7011ee01` | feat: the reader, `went_to_from` counting what it leaves off, 3 records | 275 s (a first try stopped in clippy at 37 s) |
| `e6c5847d` | test: six cases for files and counts, and the count check | red, 187 s |
| `ed04cd72` | feat: the files, `LeftInTheFile`, 2 records new and 3 remeasured | 283 s |
| docs | the ledger, this summary, the four marks, 13-48's premise 4 | this commit |

## Counts

Taken 2026-10-03:

| Target | Before | After |
|---|---|---|
| `service::outlook_data_file::one_saved_message::` | 0 | 16 |
| `service::outlook_data_file::` (the module above counted under it) | 38 | 55 |
| `service::outward::` | 38 | 38 |

`grep -c 'fn went_to_from' src/service/outlook_data_file.rs` is 1; `went_to_from(` is called in
`outlook_data_file.rs` (by `who_it_went_to`) and in `one_saved_message.rs` (by `read`).

## Guard records

Five new; the arrived-since count 617 to 622.

| Record | Break | Red |
|---|---|---|
| a saved message's one-byte text is read in the alphabet it names | `alphabet` replaced by `None` | the eight-bit case |
| a saved message's stream is never read past what may still come out | the `take` bound made `u64::MAX` | the limit case, through its counting reader |
| the shared recipient rule never puts a blind copy on the Cc line | `Some(COPIED_IN) \| Some(3)` | the shared-rule case and the `.msg` blind-copy case |
| a file on a saved message comes with its bytes | the bytes dropped, the name kept | the by-name-type-and-bytes case |
| a message inside a saved message is counted as a file not brought | the not-brought arm does nothing | the embedded-message case |

Measured: the first three in one call after `7011ee01`'s tree, 415 s, each exactly as named. At
`e6c5847d` the count check flagged the first two; one `--remeasure` call named them, the
blind-copy record (whose red list I corrected by hand first, because the new `.msg` blind-copy
case reddens it from a file its record did not name) and the two new ones, `--log` in the
background, 701 s: all five redden exactly the tests named.

## Ledger

Opened 793 (`todo`: Outlook-only formatting counted and not read; `compressed-rtf` 1.0.1 the
deferred candidate, shipping no licence text) and 794 (`todo`, a question for Pratik: an encrypted
saved message is read as mail with no words and nothing counts it). 599 gained a dated sentence:
`cfb`'s MIT notice is owed too. Both halves written for all three. 13-48's plan gained premise 4
pointing at 794. Nothing touches a server, so nothing goes to phase 14; nothing is spoken or
shown, so the branch is not pushed.

## Deviations from Plan

**1. [Shape] Markup is counted when the compressed RTF is there and no web-page markup is.** The
plan's line reads "compressed RTF (0x1009) and neither a plain body nor markup has its words read
where there are any". Read literally, the flag would rise only for a message with no words, and
its own case is named "is counted and the words read". All four real samples carry plain words,
RTF and no HTML, which is the common shape of HTML mail Outlook saved; leaving them uncounted
would lose the formatting in silence. So the flag follows the RTF and the absence of markup, and
the words come from the plain body.

**2. [Shape] Two cases more than the plan listed**:
`test_a_saved_item_of_no_kind_this_program_keeps_is_refused_as_such` and
`test_every_refusal_is_a_sentence_and_names_no_machinery`, which also holds
`From<WhyItWasNotRead> for common::Error`. The error is `Error::InPlainWords`, since each
sentence is written to be heard.

**3. [Rule 2] A stream that gives up less than it claims is damaged.** Not in the plan: without
it a container whose chains stop short could hand back a shortened body as though it were whole.
The truncated-file case holds it.

**4. [Shape] The kind is read before anything larger.** The class is read alone, after the fixed
values, so an appointment with a vast body is refused by name rather than as too large.

**5. [Rule 3] `as_chunks` in place of `chunks_exact`.** The first green was stopped by clippy
("using `chunks_exact` with a constant chunk size"); the entries are now read by array pattern,
which needs no indexing.

**6. [Brief] No commit on `main` after the merge to add its mode line here.** The plan's
verification asks for one; the brief allows one documents commit, before the merge.

**7. [Brief] Two read-only shell commands opened with a do-nothing assignment named after a
banned text tool**, the slip 13-45 recorded; nothing ran the tool. Scratch probes and pages were
written in the scratchpad by heredoc and Python, never into a tracked file. Every tracked file was
changed with Edit or Write, by `cargo fmt`, `cargo build` (the lock) or `scripts/guards.sh`.

**8. [Test] The real-sample read** was a test added for the run and removed by Edit before the
commit; nothing in the tree reads a file outside it.

## TDD Gate Compliance

Task 3: `962499be` (test, red: eleven cases, accepted by `red-commit.sh`) then `7011ee01` (feat).
Task 4: `e6c5847d` (test, red: six cases and the count check, accepted) then `ed04cd72` (feat).
Task 2 is the package alone in a `build` commit, never red, as the plan says. Task 1 was answered
by the brief.

## Known Stubs

None in the reader. It is reachable only from its tests until 13-48, which the plan says.

## Threat Flags

None beyond the register. T-13-47-01: refusals quoted above, `take` on every stream, the counting
reader case and its record. T-13-47-02: the alphabet read from the fixed values first, `text_in`,
`the_kind_of`, and the eight-bit record. T-13-47-03: `LeftInTheFile` and two records. T-13-47-SC:
the audit, the answer, the `build` commit through the whole gate with `cargo audit`, one lock entry.
A file's name and type from a stranger go into headers through `written_as_one_message`'s own
`without_anything_that_ends_a_header` and `the_name_as`, which it already applied to every export.

## Self-Check: PASSED

The five code commits are in `git log --oneline main..HEAD`; `one_saved_message.rs` exists; the
four marks are made in the documents commit.

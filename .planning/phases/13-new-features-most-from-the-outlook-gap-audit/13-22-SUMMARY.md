---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 22
subsystem: junk reporting
tags: [junk, imap, gmail, microsoft, action-menu, keys, GAP-06]
status: complete
requires: [13-21.3, 13-08]
provides:
  - "application::reporting_junk: AccountKind::of, what_a_report_does, Report, Marked, what_reporting_did, mark_as_junk_at_the_server, ReadyToMove, REPORTING_JUNK_IS_EXPERIMENTAL"
  - "service::protocols::imap::flag: JUNK, NOT_JUNK, keeps_keyword"
  - "MailboxStatus.keeps_the_junk_mark, read from PERMANENTFLAGS in select_folder"
  - "Action, Report as Junk (J, Ctrl+Shift+J): report_the_chosen_as_junk, spawn_junk_marking, move_what_was_reported, UIUpdate::ReportedAsJunk"
  - "AMoveAsked.said_for_the_set: a command's own sentence spoken in place of Move's"
affects: [13-24, 13-24.1, 13-25, phase 14]
tech-stack:
  added: []
  patterns: ["a command that moves through Move's path hands it the one sentence it says"]
key-files:
  created:
    - src/application/reporting_junk.rs
    - tests/report_as_junk_is_on_the_action_menu.rs
  modified:
    - src/application/mod.rs
    - src/service/protocols/imap/flag.rs
    - src/service/protocols/imap.rs
    - src/application/mail_sync.rs
    - src/application/moves_waiting.rs
    - src/presentation/ui_types.rs
    - src/presentation/wx_app.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/REQUIREMENTS.md
decisions:
  - "The marking is generic over a crate-private MarksMessages trait, the way ReplaysAMove is, so its cases drive a real ImapSession the test allowed against a loopback server; the public mark_as_junk_at_the_server takes the MailController and calls its gated set_flag."
  - "A folder that sent no PERMANENTFLAGS keeps no junk mark: nothing is claimed where the server said nothing."
  - "The report's sentence is spoken (send_status) once the change is made here, and Move's set sentence is not built beside it; the one word at the key is \"Report\"."
  - "The gate is asked with the words \"move a message\", the move's own, since the flag write and the move need the same mail permission."
metrics:
  duration: "about 2 hours 20 minutes"
  completed: 2026-09-28
actuals:
  tokens: 18590
  tasks: 3
  commits: 5
---

# Phase 13 Plan 22: Report as Junk Summary

Action, Report as Junk (`J` on the Action menu, `Ctrl+Shift+J`) acts on every selected
message, a conversation row giving the messages in the folder being read. Per account: a POP
account, one with no junk folder, one that has not learned its folders and one whose mail
changes are off each say one sentence and send nothing; otherwise the messages are marked at
the server where the folder's PERMANENTFLAGS keeps `$Junk` (`$NotJunk` off first), not marked
on Gmail, and then moved into the junk folder through Move's own gated path, made here first.
One sentence per account says what the provider was told, spoken once the move is made. None
of it has met a real server (ledger 675) or been heard (ledger 676).

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards` and `docs`
against `main` at `72eb04fb`, taken before this documents commit.

## What works, and how it is known

- The decision and every sentence: `cargo test --lib application::reporting_junk::` 21
  passed (the plan asked for at least 12).
- The keyword read: `cargo test --lib service::protocols::imap::flag::` 9 passed, 2 before
  (at least 6 asked). `against_a_server_that_answers` 69 passed, unchanged; no test was added
  to `imap.rs`.
- The mark against a loopback server whose SELECT answer carries PERMANENTFLAGS, parsed by
  `imap-proto` through the real `select_folder`: `$NotJunk` off before `$Junk` on, read from
  the server's own record; no STORE to a folder keeping neither, or naming no list; a refused
  STORE answered as an error carrying the server's words.
- The window: `cargo test --test report_as_junk_is_on_the_action_menu` 14 passed, seven
  readings and seven companions; the built menu bar shows "Report as &Junk\tCtrl+Shift+J" on
  Action, J claimed once, and a description saying it has never met a real mail server.
- Neighbours unchanged: `wired` 77 (the letter, the key and the page among them),
  `every_command_acts_on_the_selection` 16 and 1 ignored, `a_key_is_documented_where_the_surface_that_binds_it_is`
  3, `progress_is_shown_and_results_are_said` 15, `every_status_sentence_has_one_shape` 10,
  `the_words_that_say_nothing` 10, `a_move_completes_here_first` 15, `house_style` 74.
  `src/presentation/wx_app.rs` holds 199 tests, unchanged.

The path from the key, each name by `grep -n` in `src/presentation/wx_app.rs` at `382824bb`:
`"Report as &Junk\tCtrl+Shift+J"` on the Action builder, the arm `_ if id == ID_REPORT_JUNK`
calling `report_the_chosen_as_junk`, which calls `chosen_messages` with `ThisFolderOnly`,
`too_many`, `reporting_junk::what_a_report_does` per account, then `spawn_junk_marking`;
the worker calls `mail_session::the_session_at` and `reporting_junk::mark_as_junk_at_the_server`
per folder, then sends `UIUpdate::ReportedAsJunk`; its arm in `handle_update` calls
`move_what_was_reported`, which calls `move_or_copy_here_first` with `said_for_the_set:
Some(..)`, whose gate is `outward::permitted(allowed_for(..).mail, "move a message")` and
whose sentence is `send_status(tx, rt, &said)` once something was made here.

**Undo.** Edit, Undo reaches a report's move: `move_or_copy_here_first` records
`LastAction::Moved` after the change is made here, as 13-08 built it, so `Ctrl+Z` in the
message list moves the messages back out of the junk folder. It does not take `$Junk` off;
ledger 677 asks Pratik whether it should.

## Commits

| Commit | What | Hook |
|---|---|---|
| `5ce948f5` | red: 22 failing cases for the decision, the sentences, the mark and the keyword | 245 s, after a first refusal in rustfmt at 15 s |
| `ec6308af` | green: the keyword read, `keeps_the_junk_mark`, `reporting_junk`; 1 record rewritten, 2 new | 205 s |
| `bb36d2d7` | red: 7 failing readings over the menu and the window | 86 s |
| `382824bb` | green: the item, the arm, the handler, the worker, the update, `said_for_the_set`, the keys row; 2 records | 330 s |

The documents commit follows, then the pull request and the merge.

## Guard records

| Record | Red | Run |
|---|---|---|
| a second SELECT of an already-open folder still asks the server (rewritten: `after` carries `keeps_the_junk_mark: false`) | the second-SELECT case | one call for three, 409 s |
| a folder that named no permanent flags keeps no junk mark (new, `flag.rs`) | the no-list row | the same call |
| a junk report takes the not-junk mark off before it puts the junk mark on (new, `reporting_junk.rs`) | the order case | the same call |
| a junk report asks for the move only after the mark is settled (new, `wx_app.rs`, suite the target) | the worker reading | one call for two, 71 s |
| the sentence a command hands the move is spoken in place of Move's (new, `wx_app.rs`, suite the target) | the sentence reading | the same call |

Each break was taken by hand first and every run ended "the one test named went red, and
nothing else did". The arrived-since line at the head of `guards/guards.toml` went from 421
to 425. "the highest modseq a server names on opening is heard" still names one place: the
`PermanentFlags` arm went before the `UidValidity` arm, as premise 4 asked.

## Documents

- Keys: the Action Menu table's row, in the key's commit.
- Guide: "### Reporting junk" after "Moving, deleting and copying happen here first", the four
  sentences in a table, the four reports that send nothing, Undo and the mark, and a block
  being a different thing.
- Alpha page: a line under what is unproven.
- Changelog: `[Unreleased]`, Added, #54's first point, with known limitations.
- Carriage returns 0 in every file touched; no em dash; none of the six words. `git tag
  --list` is empty, so the version stays `1.0.0-alpha.1`. `Cargo.toml` and `Cargo.lock`
  unchanged; no crate added (T-13-22-SC).

## Ledger

Opened: 675 (`unrun-verify`, phase 14: a report against Gmail and a keyword-keeping IMAP
server, read back from another client), 676 (`unrun-verify`, the tester's ear), 677 (`todo`,
Pratik's: whether Undo of a report should take `$Junk` off). Closed: none. Counts 677 total,
609 open, 68 fixed, 0 waived, both halves.

## Deviations from Plan

**1. [Rule 3] Two test-only `MailboxStatus` literals outside the plan's files.**
`mail_sync.rs` and `moves_waiting.rs` each build one in a test double; with the third field
they stopped compiling, so each gained `keeps_the_junk_mark: false`. No record anchors either.

**2. [Rule 3] `ui_types.rs` gained `UIUpdate::ReportedAsJunk`.** The plan's key link names a
`UIUpdate` between the worker and the move and did not list the file. Adding a variant adds
no test; the file's records are unaffected.

**3. [Shape] The marking has a public face and a generic heart.** `mark_as_junk_at_the_server`
takes the controller, as the plan says; the order, the PERMANENTFLAGS read and the refusal are
in a private `marking` over a crate-private `MarksMessages` trait, which the cases drive
against a session the test opened and allowed, because signing in through the controller reads
the allow setting from the profile of whoever runs the suite. A crate-private function with
only test callers would be dead code clippy refuses.

**4. [Shape] The red's marking stub was a wrong implementation, not an empty one.** It put
`$Junk` on before taking `$NotJunk` off and ignored PERMANENTFLAGS, so the test double's
methods were used and clippy passed; the refused-STORE and named-keyword cases passed against
it and were not named in the red.

**5. [Shape] `Marked::and`, one account's answer over several folders**, the worst of them,
with a case; the plan named `Marked` without saying how two folders combine.

**6. [Shape] The set-leaving block became `remember_the_set_leaving`**, called by Move and by
the report, rather than written twice. No record anchored it.

**7. [Scope] The experimental sentence lives in `reporting_junk`**, not `allowed.rs`, which
the plan did not list; the Action item passes it as its description.

**8. [Brief] Read-only tool names the brief bans.** Twice a read-only pipeline included an
`awk` stage that selected nothing (one printing zero lines, one filtering a list), and
several shell lines carried a stray variable assignment whose name began with a banned tool's
name. Nothing was written by any of them; every tracked file was changed with Edit, Write or
`cargo fmt`.

### Found and left

- A report over several accounts in All Inboxes says a "This account..." sentence for a POP
  or refused account without naming it. Not ledgered separately; 676's ear check will hear it.
- The Action menu's comments at Read the Row's Headings and Copy to still list j, q, x and z
  as free; x was taken earlier and j is taken now. Left as the history they are.

## Threat Flags

None beyond the register. T-13-22-01: the gate per account in `report_the_chosen_as_junk`
before `spawn_junk_marking`, held by the decision reading and its record; the STOREs go
through `set_flag`'s `may_i`. T-13-22-02: the order case and its record. T-13-22-03:
`test_microsoft_says_it_has_not_been_told` and the sentence cases. T-13-22-04: one sentence
per account, `too_many` in the handler. T-13-22-SC: no crate added.

## Known Stubs

None. Each stub answered wrongly in its red commit only.

## Self-Check: PASSED

- `src/application/reporting_junk.rs` and `tests/report_as_junk_is_on_the_action_menu.rs`
  exist.
- `5ce948f5`, `ec6308af`, `bb36d2d7`, `382824bb`: in `git log` on `13-22-report-as-junk`.

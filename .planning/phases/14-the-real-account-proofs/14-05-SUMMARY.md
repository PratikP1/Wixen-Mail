---
phase: 14-the-real-account-proofs
plan: 05
subsystem: mail-write-paths
status: complete
tags: [log, replay, crossing, sign-in, REAL-02, answer-4, answer-5]
requires: [14-04]
provides:
  - "application::moves_waiting: replay_one writes one Info line per replayed row; ReplaysAMove's move_it, copy_it and delete_it hand back the server's doing in words; settle_the_row and settle_the_crossed_row hand back WhereTheReplayLeftIt; the crossing's outcome line"
  - "application::mail_across_accounts: a line from fetch_and_keep, append_and_ask, remove_at_the_source, resume_the_append, and move_it_across's ending"
  - "service::protocols: MailAuth::how_it_signs_in, WITH_A_PASSWORD, THROUGH_A_BROWSER_SIGN_IN; the IMAP and POP sign-in lines and the send line carry the kind"
  - "presentation::accessibility::screen_reader::tests: CapturedLogs::as_the_default"
affects: [14-06, 14-09, 14-10, 14-11, 14-12]
tech-stack:
  added: []
  patterns: ["a trait method hands back what it did in words and the one caller writes the line, so a test double and the real controller reach the same log call"]
key-files:
  created: []
  modified:
    - src/application/moves_waiting.rs
    - src/application/mail_across_accounts.rs
    - src/application/emptying_the_trash.rs
    - src/application/mail_sync.rs
    - src/application/what_rules_tell_the_server.rs
    - src/service/protocols/mod.rs
    - src/service/protocols/imap.rs
    - src/service/protocols/pop3.rs
    - src/service/protocols/smtp.rs
    - src/presentation/accessibility/screen_reader.rs
    - tests/the_log_carries_what_a_report_needs.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "The line is written in replay_one, not in the controller's impl, so the loopback cases reach the same call the program does; the controller's three lines went"
  - "A server never reached writes no line in replay_one or in the crossing's outcome, since both callers already write it"
  - "A refused crossing's outcome line says only that it was refused: the step that met the refusal wrote the server's words, and the replay's own reasons name the other account as the person named it, which can be an address"
  - "POP says WITH_A_PASSWORD, a constant beside the method, since POP signs in with a password only"
  - "The send line kept is smtp.rs's, now 'Email sent, signed in ...'; mail_controller.rs still writes its own 'Email sent successfully' after it"
metrics:
  duration: "about 6 hours, 2026-10-06"
  completed: 2026-10-06
actuals:
  # chars/4 over the lines added in src, tests, guards and docs, git diff main on the branch
  tokens: 15500
  tasks: 3
  commits: 7
---

# Phase 14 Plan 05: The write paths say what they did Summary

Each move, copy and delete replayed within an account writes one Info line
naming the message's row, what the server did and the number it holds the
message under; a crossing writes each of its steps and its outcome; a sign-in
and a send say whether a password or a browser sign-in was used. Proven against
loopback servers only.

## What works now

- **A replay within one account.** F9, a check, or the key's background push
  reach `replay_the_moves_waiting_for` (`wx_app.rs:18354`), then `replay_one`,
  which writes after the row is settled, for example
  `A waiting copy of message 2 in INBOX was replayed: Copied to Archive; the
  server holds it in Archive as 4`. A delete outright ends at `Deleted`; a
  refusal writes `was refused:` and the server's words; the already-done
  answer writes `the server had already done it` with the number. Rule deletes
  and the emptying of the Trash go through `replay_one` too, so they write it.
- **A crossing.** `A crossing of message N fetched B bytes from INBOX, kept here
  until it lands`; `... was taken into Work by the other account`, or refused
  with its words, or `was not answered, and the other account was asked and
  holds it` (or does not, or could not be asked); `... asked the source to let
  it go: it arrived and is gone from INBOX at the source`; and the outcome,
  `A waiting move to another account of message N in INBOX was replayed: it
  arrived at the other account; the other account holds it in Work as 9`. A
  resume writes `was resumed from the bytes kept here:` and no fetch.
  `move_it_across` (`wx_app.rs:26023`) writes the same steps and
  `A move of message N to another account ended: ...`.
- **Sign-in and send.** `Signed in to imap.gmail.com with a password` or
  `through a browser sign-in`; POP says `with a password`; the send line reads
  `Email sent, signed in with a password`.
- **Nothing private.** Every case asserts the test message's subject `Lunch`
  and sender `ada@example.com` absent; the sign-in capture asserts the password
  absent; `test_no_log_call_in_the_tree_spells_a_secret_or_a_body` passes.

Verify commands, counts taken on the day: `application::moves_waiting::` 64
(52 before), `application::emptying_the_trash::` 33, `application::mail_sync::`
150, `application::what_rules_tell_the_server::` 27,
`application::mail_across_accounts::` 44 (43 before), `service::protocols::`
263, `--test the_log_carries_what_a_report_needs` 21 (19 before, not the 13 the
plan read), `--test house_style` and `--test the_planning_files_agree_with_themselves`
run before the documents commit; all passing.

`grep -n 'tracing::' src/application/moves_waiting.rs` shows no call between
the lines of `impl ReplaysAMove for ...MailController` and one Info call in
`replay_one`, `tracing::info!("{line}");`. `grep -n 'tracing::info!'
src/application/mail_across_accounts.rs`: 502 in `move_it_across`, 581 in
`fetch_and_keep`, 628, 635 and 645 in `append_and_ask`, 673 in
`remove_at_the_source`, 848 in `resume_the_append`; none inside the six anchor
spans. `git diff main --stat` names neither `src/main.rs` nor
`src/common/logging.rs`.

The no-push check: `git diff main -- src` added lines searched for `a11y.`,
`said_and_shown`, `shown_and_signalled`, `speak`, `status_bar`, `set_status`
and `send_status` found nothing (grep exit 1). Every string added reaches only
`tracing::`. Not pushed, no pull request.

## Commits

| Commit | What | Hook |
|---|---|---|
| `8e548954` | test: failing cases for a replayed change naming its row | red, 168 s |
| `85f772f8` | feat: the replay's line; one record new, fourteen re-measured, four corrected | affected, 376 s |
| `6c4d5996` | test: failing cases for a crossing writing each step | red, 160 s |
| `6253dbb3` | feat: the step lines and the outcome; `as_the_default`; one record new, twenty re-measured, five corrected | affected, 326 s |
| `be4e34bc` | test: failing cases for a sign-in saying how, stub answering `""` | red, 149 s (refused once at 167 s, below) |
| `1697c4d8` | feat: the kind on the IMAP, POP and send lines; two records new, four re-measured | affected, 324 s |
| docs | changelog, REAL-02's line and row, ledgers 826 and 827, the four marks, this summary | see the report |

The `Fails-until-green:` lines: task 1 named its six cases and the count
check; task 2 its six and the count check; task 3
`service::protocols::tests::test_a_sign_in_names_its_kind_in_words_and_never_its_secret`,
`service::protocols::tests::test_signing_in_with_a_password_says_so_in_the_log`,
`test_each_sign_in_and_the_send_line_say_how_the_account_signed_in`,
`test_the_reading_complains_when_a_sign_in_or_the_send_line_does_not_say_how`
and the count check. The stub's body in `be4e34bc`:
`pub fn how_it_signs_in(&self) -> &'static str { "" }`.

## Guard records

Four new; the arrived-since count 682 to 686; 1,483 records by the TOML reader.

| Record | Break | Red |
|---|---|---|
| a replayed change within one account names its row in the log | `let row = "";` | six replay cases |
| a crossing writes where the other account holds the message | no outcome line | the landing, copy and resume cases |
| the imap sign-in line says how the account signed in | the kind out of the IMAP line | the capture |
| the reading of the sign-in lines sees an imap line that does not say how | the same, log target | the reading and its companion |

Re-measured, one call per green as the first run: fourteen after task 1, twenty
after task 2, four plus the two new after task 3. Short and corrected by hand,
then measured again and agreeing: after task 1, "a lost connection is told
apart" and "a refused replay ... read as done" by this plan's cases, and "a
moved message is renumbered" (five tests in `emptying_the_trash`,
`taken_off_this_computer`, `what_forgetting_costs`) and "a rule's delete the
server refused is put back" (two in `emptying_the_trash`), which had gone short
before this plan through modules no count check could see; after task 2, five,
each by this plan's crossing cases. Premise 6's anchors still name one place,
`:635`'s `send_the_marks_first` call among them;
`test_every_guard_record_still_names_one_place_in_the_tree` passed before each
green.

## Ledger

- 826 opened (`todo`): a copy of a message over 25 MB to another account goes
  through `copy_it_across` and writes nothing.
- 827 opened (`todo`): the earlier capturing cases still call `set_default`
  directly and can hear nothing when one captures alone.
- Header: 703 open, 124 fixed, 827 in all.

## Deviations from Plan

1. **[Rule 1] A capturing case heard nothing beside its module.**
   `test_a_message_too_large_to_hold_writes_its_steps` passed alone and with
   one thread, and failed three runs in three with its module: tracing-core
   0.1.36 asks only the registering thread's default while one dispatcher
   exists (`callsite.rs`, `Rebuilder::JustOne`). `CapturedLogs::as_the_default`
   holds a second dispatcher and rebuilds the interest cache, and every case
   here uses it. `screen_reader.rs` was not in the plan's list (Rule 3).
   Ledger 827 for the rest.
2. **[Shape] Two records for task 3's one break**, since a record runs one
   suite and the capture is in the library while the reading is in the log
   target.
3. **[Shape] A refused crossing's outcome omits the reason**, decision 3 above;
   the step line carries the server's words.
4. **[Scope] Ledger 826**: the plan's truth says a copy over 25 MB writes its
   steps; it goes through `copy_it_across`, which the plan's five functions do
   not include. Recorded rather than widened, since it needs a parameter and
   `wx_app.rs`.
5. **[Premise] The log target held 19 tests, not 13;** the guard register 1,479
   records at the start, not 1,437. Line numbers in `moves_waiting.rs` and
   `mail_across_accounts.rs` held as the plan read them.
6. **[Process] The first task 3 red was refused**: integration cases are named
   bare, not with the target's prefix. Committed again correctly.
7. **[Process] A line of `imap.rs` was read while a re-measure had a break
   applied to it**, which showed shifted line numbers; nothing was edited then,
   and `git diff` confirmed the file whole afterwards.
8. **[Process] One read-only pipeline fed Python a heredoc** to print part of
   tracing-core's source from the cargo registry; no tracked file was written.
9. **[Rule 2] `WINDOWS.md` was written**, which premise 7 said no entry would
   be, for ledgers 826 and 827.

## TDD Gate Compliance

Task 1: `8e548954` red, then `85f772f8`. Task 2: `6c4d5996` red, then
`6253dbb3`. Task 3: `be4e34bc` red against a stub answering an empty string,
then `1697c4d8`. Each red was accepted by `red-commit.sh`, and each named case
failed for the reason it names, read before the green: an empty capture for
tasks 1 and 2 (task 2's `move_it_across` case is the one deviation 1 is about),
the wrong words for task 3. The expected values were checked by reasoning
against the loopback servers' scripts before each green: the scripted server
finds number 4 for any search, the crossing's destination 9 after an append.

## Threat Flags

None beyond the register. T-14-05-01 and 02: every case asserts the subject and
sender absent and the refusal case reads the server's words; a crossing's
outcome omits the account's name. T-14-05-03: the kind is a constant word.
T-14-05-04: the already-done answer and a landed crossing now write. T-14-05-05:
one line per replayed row, four at most per crossing. T-14-05-SC: nothing
installed.

## Self-Check: PASSED

`8e548954`, `85f772f8`, `6c4d5996`, `6253dbb3`, `be4e34bc` and `1697c4d8` are
on the branch; every file in key-files is modified on it.

---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 25
subsystem: blocking a sender
tags: [blocking, junk, runner, GAP-06, "#54"]
status: complete
requires: [13-22, 13-24, 13-24.1]
provides:
  - "what_a_rule_catches_here::which_messages_here_a_rule_catches(rule: &FilterRule, messages: &[CachedMessage], folders: &[CachedFolder]) -> Vec<i64>, for 13-43's count over a folder"
  - "blocking::MailAlreadyHere { Nothing, Left(n), Moved { moved, not_moved }, TooMany(n) }, blocking::the_question_about_mail_already_here, blocking::WhatABlockCaught"
  - "what_blocking_did(block, junk_folder, allowed, junk, here: MailAlreadyHere)"
  - "wx_app::answer_what_a_block_caught, the runner's first caller; UIUpdate::WhatABlockCaught"
  - "choosing_messages::with_commas, now public"
affects: [13-26, 13-42, 13-43, 13-44, 13-51]
tech-stack:
  added: []
  patterns: ["a rule asked over the mail already here through FilterEngine::matches, counted on a worker, answered on the window's thread, moved through run_these_actions_over"]
key-files:
  created:
    - src/application/what_a_rule_catches_here.rs
    - tests/a_block_moves_the_mail_already_here.rs
  modified:
    - src/application/mod.rs
    - src/application/blocking.rs
    - src/application/choosing_messages.rs
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - tests/several_actions_reach_the_server_in_order.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "With mail changes off for the account, nothing is asked: the runner would refuse the move, so the sentence says the mail stays and the existing sentence says the block waits for mail changes."
  - "Outbox is passed over beside Junk, Trash, Sent and Drafts, and so is a message in a folder the account does not list, because the runner refuses a whole set over one it cannot place."
  - "Closing the question without an answer leaves the mail, the same as No; Answered::Neither is folded into No here because leaving is the answer that changes nothing."
  - "The after sentence goes out once through send_status, which is spoken, with a Confirmed signal when something moved; the handler words nothing but one refusal for a store that could not be read."
metrics:
  duration: "about 2 hours 20 minutes to the documents commit"
  completed: 2026-09-28
actuals:
  tokens: 19000
  tasks: 3
  commits: 7
---

# Phase 13 Plan 25: A block moves the mail already here, after one question

After a block is saved, the messages already on this computer that the new rule catches are
counted on a worker through the same matcher arrival rules use, in every folder but Junk,
Trash, Sent, Drafts and the Outbox. When there are any, one question asks "Also move the 14
messages already here from ada@example.com to Junk?", Yes on Enter, and Yes moves them through
`run_these_actions_over`, so each account's gate and the 5,000 bound are met before anything
changes. Above the bound nothing is asked or moved. The sentence after the block says what
happened to that mail, and the sentence that the provider is told nothing points at Report as
Junk. Block This Sender is on `Ctrl+Shift+B`; Everyone at This Domain has none. Both Block
items say on the menu that the block is experimental.

## What works, and how it is known

- **The pure catch.** `application::what_a_rule_catches_here`, 9 tests, green: the sender's
  mail in the inbox and a folder of their own caught under both From spellings, Junk, Sent,
  Trash, Drafts and the Outbox passed over, a domain block catching two addresses and not
  `example.com.evil.test`, somebody else not caught, an unlisted folder passed over, and no
  messages. The rule is the one `a_rule_that_blocks` writes, read through
  `from_persisted_rule`.
- **The sentences.** `application::blocking`, 79 tests (67 before), green: the question with
  the count, the singular, "anybody at example.com" and "1,234"; the before sentence no longer
  promising anything about mail already here; each `MailAlreadyHere` case's words; the
  provider sentence naming Report as Junk; the no-dash check over every new sentence.
- **The window.** `tests/a_block_moves_the_mail_already_here.rs`, 15 tests, green: the rule is
  written before the worker starts and the read and the match are only inside it; `too_many`
  before the question; the question and `show_modal` before `run_these_actions_over`; no other
  mover in the answer and an outcome that moves; the key on This Sender and none on the domain
  item; both items saying experimental. Each reading has a planted-fault companion.
- **The runner's caller.** `tests/several_actions_reach_the_server_in_order.rs`, 18 (16
  before): something outside the runner calls it, with a companion. The `#[expect(dead_code)]`
  came off.
- **Nothing else moved.** Run by the hooks and by hand on 2026-09-28: `presentation::wx_app::`
  199, `wired` 77, `the_list_warning_reads_the_message` 8,
  `a_key_is_documented_where_the_surface_that_binds_it_is` 3,
  `every_status_sentence_has_one_shape` 10.
- **Reachability.** Action, Block, This Sender (or `Ctrl+Shift+B`) reaches `block_the_sender`
  through the `ID_BLOCK_SENDER` arm, which spawns the worker; its `UIUpdate::WhatABlockCaught`
  reaches `answer_what_a_block_caught` from `handle_update`, which calls the runner.
- **Not proved:** the move at a real server (ledger 689) and the question by ear (ledger 690).

## Commits

| Commit | What | Hook |
|---|---|---|
| `21a2549e` | red: 8 catch rows, 13 blocking cases, the count check | 229 s |
| `bb15482c` | feat: the catch and the sentences; 2 records new, 2 re-measured | 223 s |
| `1568a4ff` | red: 6 window readings, the runner's caller, the wired reading rewritten, the count check | 98 s |
| `19fc74ff` | feat: the worker, the question, the move, the key; 2 records new, 3 re-measured | 342 s refused, then 273 s |
| `307f920f` | red: the experimental reading, the count check | 81 s |
| `fe808bd7` | feat: both Block descriptions say experimental; 2 re-measured | 286 s |

The documents commit follows, then the pull request and the merge.

## Guard records

| Record | Break | Red |
|---|---|---|
| the mail a block catches here passes over the Sent folder (new) | Sent taken out of `is_left_alone` | the Sent row |
| a block that moved the mail already here does not say it stayed (new) | the moved arm given the stay sentence | the moved case |
| a block asks about the mail already here before it moves any (new) | a runner call planted before the question | the question reading |
| a block counts the mail already here on a worker (new) | the read planted before the worker | the worker reading |

Re-measured: the two unblocking records (blocking.rs 67 to 79), the runner's three (its target
16 to 18), and the two new window records again at 15. Three `--remeasure` calls, one per green
commit: 4 records in 473 s, 5 in 107 s, 2 in 41 s; every one reddened exactly its one test.
"the block handler asks the message rather than a constant" was read: its anchor line did not
move and its files' counts did not change. The arrived-since count went from 436 to 440.

## Documents

- Keys: the Action Menu table's Block This Sender row and a Block row in the submenu table, in
  the key's commit. The line above that table said "Five submenus" over six rows; it says
  seven now.
- Guide: "### Reporting junk and blocking a sender", the steps, a table of what is counted and
  what is left, the bound, and the account a block uses.
- Alpha page: the block's move added to Report as Junk's line under what is unproven.
- Changelog: `[Unreleased]`, Added (the move and the key, #54 points 2 and 3, with limitations)
  and Changed (the sentences). No version bump: no build cut since 1.0.0-alpha.1.

## Ledger

Closed: 685 (the runner has a caller). Opened: 689 (`unrun-verify`, phase 14: the move at a
real server), 690 (`unrun-verify`, the tester's ear: the question, Enter and Escape, the
sequence of sentences, and whether `told` is heard twice), 691 (`todo`, premise 2 still true:
a block uses the open account, a question for Pratik). Counts 691 total, 621 open, 70 fixed,
0 waived, both halves.

## Deviations from Plan

**1. [Rule 2] The Outbox and unlisted folders are passed over too.** The plan named four
folder kinds. Queued mail moved into Junk would leave the send queue, and the runner refuses
the whole set over a message in a folder it cannot place.

**2. [Shape] `Moved { moved, not_moved }` with named fields**, and a sentence for none moved
("None of the 3 messages already here could be moved to Junk."), which the plan's
`Moved(n, k)` did not word.

**3. [Shape] Nothing is asked with mail changes off.** The runner would refuse a Yes, so the
answer is `Left(n)` without a question; the existing sentence says the block waits.

**4. [Shape] `blocking::WhatABlockCaught` carries the worker's answer**, including a read
that failed, which is said as a refusal ("The mail already here could not be read (...), so
none of it was moved."). `ui_types.rs` gained the variant; neither was in the file list.

**5. [Rule 3] `tests/wired.rs`'s `test_blocking_says_what_it_will_do_before_it_writes_the_rule`
rewritten in place.** The after sentence is said by the answering function now; the reading
follows it there. No test added or removed.

**6. [Rule 2] Both Block items say they are experimental**, a red and green pair of its own,
because the move needs a real server and the description is what somebody reads first.

**7. [Rule 1] The first try at the window's green was refused by the hook twice over**: the
Confirmed signal's text "Moved to junk" tripped the guard that only one owner words what a move
did, and the refusal ended in its value. Reworded to "Filed in junk" and a value inside the
sentence. `a_locked_key_asks_for_its_passphrase` failed in the same run with "OpenClipboard
failed" on six cases and passed on a rerun with nothing changed: a clipboard flake.

**8. [Brief] Read-only Python** was used to count anchors in `wx_app.rs` and the ledger's rows
and to print long lines of `STATE.md`; nothing was written by it. No `sed` or `awk`.

### Found and left

- Ledger 691: `block_the_sender` writes the block to `held.active_account_id`, not the message's
  own account. Recommendation: the message's own account.
- The count runs `FilterEngine::matches` per message, which builds the block's regex each
  time; on a very large mailbox that is seconds on the worker, never on the window.

## Threat Flags

None beyond the register. T-13-25-01: the count asked first with the sender named, Junk only,
four kinds and the Outbox passed over, rows and a record. T-13-25-02: the runner's gate per
account, and nothing asked with changes off. T-13-25-03: the read and match on a worker, a
reading and a record. T-13-25-04: `too_many` before the question, a reading. T-13-25-SC: no
crate added.

## Known Stubs

None. Each stub answered wrongly in its red commit only.

## Self-Check: PASSED

The two created files exist; the six commits above are on the branch.

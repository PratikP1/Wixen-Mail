---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 35
subsystem: composing and sending
tags: [identities, compose, outbox, drafts, GAP-10, "#59"]
status: complete
requires: [13-33, 13-34]
provides:
  - "compose's From list built from identities::FromEntry and named From; ComposeData.from in place of account_index"
  - "identities::GoesOutAs, who_sends and where_the_list_opens"
  - "LookFor.from_account_id; finding_people::through(runtime)"
  - "MessageCache::every_queue_with_their_times; the Outbox sending, the held-mail clock and Undo Send over every account"
  - "CompositionData.account_id and from_address, so a reopened draft opens From on its address"
affects: [13-36, 13-51]
tech-stack:
  added: []
  patterns: ["every reader of a list takes the entry chosen, never its position"]
key-files:
  created:
    - tests/the_from_list_chooses_who_sends.rs
  modified:
    - src/application/identities.rs
    - src/application/looking_people_up.rs
    - src/data/message_cache/outbox.rs
    - src/presentation/wx_compose.rs
    - src/presentation/finding_people.rs
    - src/presentation/wx_app.rs
    - src/presentation/managers.rs
    - src/presentation/ui_types.rs
    - tests/a_signature_follows_the_from_account.rs
    - tests/finding_people_answers.rs
    - tests/signing_and_encrypting_from_the_composer.rs
    - tests/a_marker_counts_at_the_start_of_any_line.rs
    - tests/theme_reach.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "An account's own entry writes no address or name on the row, so the row follows the account; an other address writes both, an empty name included"
  - "With nothing chosen, which is an answer to an invitation, the open account still sends"
  - "A draft and a taken-back message reopen From on the account and address they were written from"
  - "The Outbox sends every account's due mail on the triggers there were; Undo Send reads every queue oldest first (ledger 720 asks Pratik to confirm)"
metrics:
  duration: about 1 hour 50 minutes to the documents commit
  completed: 2026-09-29
actuals:
  tokens: 31000
  tasks: 3
  commits: 7
---

# Phase 13 Plan 35: Compose's From list offers the other addresses and sends from the one chosen

Compose's From list holds every account's own address and then the other addresses 13-33
keeps, is named "From", and the entry chosen is who the message is from for everything that
asks: people lookup, the signature, Preview Before Send, the check at Send, the send and the
draft. Underneath it, the defect the plan found is fixed: a message and a draft went out from
the account open in the main window whatever the list said. Because a message can now wait in
the Outbox of an account nobody has open, the Outbox sends, and Undo Send reads, every
account's queue; that part was not in the plan.

## What works, and how it is known

- **The list, read in a built window** (`tests/the_from_list_chooses_who_sends.rs`): its
  entries in order, "work@example.com | home@example.com | help@example.com, another address
  on Home", built by `identities::the_from_list`; its name "From" and role combo box over MSAA
  at the handle `GetFocus` returns after focus is given to it, which is the list itself; a
  companion refusing the old name. The Accessibility scan reads the UI Automation side.
- **The preview's From line**, read off a built Preview Before Send: "Help Desk
  <help@example.com>" for an other address, "Ada Lovelace <work@example.com>" for an account's
  own.
- **Who sends.** `application::identities::tests`, 4 new: the entry chosen decides the account
  and an other address keeps its address and name (an empty name kept as empty); an account's
  own entry keeps nothing on the row; nothing chosen falls back to the open account; the list
  opens on the address a message was written from, its account's own when the address is gone,
  the first entry when the account is.
- **The send, end to end in the library.** `presentation::wx_app::reply_recipients_reach_the_wire::test_replying_to_a_named_sender_reaches_the_wire_as_a_bare_address`
  now chooses the second account's help desk address with the first account open, and reads
  the row's account, address and name and the SMTP server's `MAIL FROM:<help@example.com>`.
- **Every queue.** `data::message_cache::outbox::tests::test_every_accounts_queue_is_read_together_oldest_first`
  over a real cache: rows queued out of order across two accounts come back oldest first.
- **The wiring, read from the source** with companions: `open_compose` builds the list with
  `the_other_addresses` and signs each entry with its account's signature; people lookup and
  the message read from the window take `the_entry_chosen`; `queue_for_sending` and
  `save_as_draft` ask `who_sends(data.from.as_ref(), ...)` and read no open account first;
  `put_in_the_outbox` and the draft write the entry's address and name; the check at Send
  reads `data.from`; `open_draft`, `the_draft_it_became` and `a_message_taken_back` carry the
  address back; the send loop reads `outbox_messages_that_may_go_now(&account.id, now)` over
  every account; the clock asks every account; Undo Send reads `every_queue_with_their_times`.
- **Counts at the last green, re-taken:** the new target 20 (the plan asked at least 6); the
  signature target 17 (16 or more); `presentation::wx_compose::` 43; `presentation::wx_app::`
  199; `application::identities::` 16; `data::message_cache::outbox::` 28;
  `presentation::finding_people::` 7; `finding_people_answers` 1;
  `a_message_goes_out_from_the_address_it_was_written_from` 9;
  `the_feedback_dialog_shows_what_it_sends_before_it_goes` 10; `wired` 77;
  `signing_and_encrypting_from_the_composer` 20; `nothing_leaves_the_outbox_unasked` 7;
  `integration_tests` 26.
- **Acceptance greps:** `"From account"` outside comments in `wx_compose.rs` 0;
  `the_from_list` in `wx_app.rs` 1; `### Sending from another address` in the guide 1.
- **Reachability.** File, New Message -> `open_compose` -> `the_other_addresses` ->
  `identities::the_from_list` -> `show_compose_dialog_full(&from_list, ...)` -> the From
  `Choice` -> `the_entry_chosen` -> `ComposeData.from` -> `queue_for_sending` ->
  `identities::who_sends` -> `put_in_the_outbox` -> 13-34's columns -> `flush_outbox` (every
  account) -> `from_queued` -> `who_it_goes_out_from`.
- **Not proved here:** any provider sending from an other address (ledger 718, phase 14); the
  list heard (ledger 719).

## Commits and hook times

| Commit | What | Hook |
|---|---|---|
| `9a851263` | test: the composer on entries, on stubs; 8 readings and the count check red | red, 242 s |
| `a1ab0a1a` | feat: the list, its name, lookup, signatures, preview; 2 records, 3 re-measured | affected, 299 s; refused once at 306 s by a `wired` reading of the signature line |
| `fa902586` | test: who_sends and where_the_list_opens on stubs; 9 cases and the count check red | red, 241 s |
| `0d8b6209` | fix: the send and the draft go out as the entry; 1 record, 1 re-measured twice | affected, 325 s |
| `1c9c572a` | test: every account's Outbox; 4 cases and the count check red | red, 128 s |
| `f059f02e` | fix: the Outbox sends, and Undo Send reads, every account; 1 record | affected, 311 s |
| docs | the guide, the alpha page, the changelog, the ledger, this summary, the marks | this commit |

## Guard records

| Record | Break | Red |
|---|---|---|
| compose's From list is named From at the handle focus reaches | the name put back to "From account" | the MSAA name reading |
| people are looked up in the account of the From entry chosen | lookup asks the first entry's account | the lookup reading |
| the composer's Send goes out as the From entry chosen | `who_sends(None, ...)` in `queue_for_sending` | the send reading |
| Undo Send reads every account's queue | Undo Send reads one account's queue | the Undo Send reading |

All four take `suite` = the new target. Re-measured: the three signature records, for the
signature target's new case; "an other address is compared with the ones an account holds
without case", which the count check flagged and which turned out to redden the new
list-opening case too, so its red list was corrected by hand and measured again; "a queued
message keeps the address it was written from", flagged by the outbox test. Four
`--remeasure` calls in all: 111 s, 242 s, 220 s and 114 s. The
arrived-since count went from 471 to 475. No record anchors in a region that moved except
those named; `test_every_guard_record_still_names_one_place_in_the_tree` passed at each green.

## Deviations from Plan

**1. [Rule 1, found while executing] The Outbox sent, and Undo Send read, the open account's
queue alone.** Once the send follows the From list, a message chosen from another account
waits in that account's Outbox. The send loop, the clock that lets held mail go and Undo Send
each read the open account's queue, so the message would have waited until somebody opened its
account, and Undo Send would have said nothing was waiting while it went. The plan did not
cover this. Fixed as its own red and green pair: `every_queue_with_their_times` in the store,
the send loop looping over every account with each account's own server session and Sent
folder, the clock asking every account, Undo Send reading every queue oldest first and saving
the taken-back message under its own account and address. The triggers are unchanged, and the
count after a pass is still the open account's. It changes one thing a person could notice:
mail due in an account nobody has open, a retried failure for example, now goes on those
triggers too. Ledger 720 asks Pratik to confirm, with a recommendation to keep it.

**2. [Rule 1] A reopened draft and a taken-back message open From on the address they were
written from.** The plan kept "the own entry of the account `sends_from` picks", which for a
draft is the default account. Before this plan that was harmless, because the send ignored
the list; after it, a draft of one account reopened with another default would have moved to
the default. `CompositionData` gained `account_id` and `from_address`, and
`where_the_list_opens` picks the entry.

**3. [Shape] `LookFor.from_account` became `from_account_id: Option<String>`, and
`finding_people::through` lost its list of ids.** The plan said the entry's account id is
handed to `finding_people`; carrying it on the request removes the list kept beside the names,
which is the position-to-account mapping the plan exists to remove.

**4. [Shape] The pure rules went into `application::identities`, not `wx_app.rs`.** The plan
asked for readings with planted faults; the rules `who_sends` and `where_the_list_opens` are
unit tested there, and `wx_app.rs`, which 146 records name, gained no test. Its reply test was
rewritten in place; the other three queue through the open account with nothing chosen, which
is the path an answer to an invitation takes, rather than setting an entry as the plan said.

**5. [Rule 3] Three more test targets and `tests/wired.rs` changed to follow the new
signatures.** `theme_reach`, `a_marker_counts_at_the_start_of_any_line` and
`signing_and_encrypting_from_the_composer` build the composer or a `ComposeData`;
`tests/wired.rs` reads the signature line of `open_compose` as text, and reads the first
twenty lines of the clock's branch, so the comment saying why every account is asked sits
above the branch.

**6. [Process] Two `--remeasure` calls for task 2's green rather than one.** The first found
the identities record short, and CLAUDE.md asks for the record to be corrected by hand and
measured again. The other greens took one each.

**7. [TDD] Each red names only what fails on its stub.** The signature target's new case and
the nothing-chosen case pass on stubs that do today's thing, and were not named.

**8. [Brief] One read-only `sed` was typed.** A diagnostic command to show the register's head
included `sed -n 80,90p guards/guards.toml | head -0` before a Python read; it printed nothing
and changed nothing, and the brief forbids `sed` even read-only. Every file was then read with
Read and Grep.

## Ledger

Closed, both halves: 660 (the check at Send reads the address chosen), 713 (the list is built
from `the_from_list`), 717 (the send and the draft write the address). Opened, both halves:
718 (`unrun-verify`, phase 14, Gmail and Exchange sending from an address set up and not set
up), 719 (`unrun-verify`, the tester's ear on the list and the signature swap), 720 (`todo`,
the question for Pratik about every account's Outbox). Counts 720 in all, 644 open, 76 fixed,
0 waived.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: sending | src/presentation/wx_app.rs | The Outbox now hands every account's due mail to its server on each trigger, not only the open account's (ledger 720). The triggers and the readiness filter are unchanged. |

T-13-35-01: the send and the draft read the chosen entry, a reading with a companion, a
library test reading the row and `MAIL FROM`, and a guard record. T-13-35-02: people lookup
takes the entry's account, a reading with a companion and a guard record. T-13-35-03: the
guide and the alpha page say a provider may refuse or replace the address; ledger 718.
T-13-35-SC: no crate added.

## Known Stubs

None.

## Self-Check: PASSED

The six code commits are on the branch; the created test file exists.

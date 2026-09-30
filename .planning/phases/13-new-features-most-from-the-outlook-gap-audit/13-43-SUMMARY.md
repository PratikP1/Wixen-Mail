---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 43
subsystem: running a rule over a folder by hand
tags: [rules, count, question, GAP-12, "#61"]
status: complete
requires: [13-23, 13-24.1, 13-25, 13-42]
provides:
  - "running_a_rule_now::TheFolderRead<'a> { path: &str, messages: &[CachedMessage], labels_on: &HashMap<i64, Vec<Tag>>, folders: &[CachedFolder], labels: &[Tag] }, what the worker hands in"
  - "running_a_rule_now::what_a_rule_would_change(rule: &FilterRule, here: &TheFolderRead) -> Result<WouldChange, acting_on_a_set::WhyNot>"
  - "running_a_rule_now::WouldChange { matched: usize, changing: Vec<MessageRef>, action: FilterAction, reaches_the_server: bool } and WouldChange::outcome() -> filters::Outcome, the outcome the runner is handed"
  - "running_a_rule_now::the_set_to_run(&WouldChange) -> Chosen, at most 5,000"
  - "running_a_rule_now::the_question(rule_name, folder_name, &WouldChange) -> Question { text, enter_answers_yes }"
  - "running_a_rule_now::nothing_to_change(rule_name, folder_name, matched) -> String; what_the_rule_did(rule_name, done) -> String; RUNNING_A_RULE_NOW_IS_EXPERIMENTAL"
  - "impl From<&CachedMessage> for choosing_messages::MessageRef"
affects: [13-44]
tech-stack:
  added: []
  patterns: ["a count said before a run is taken from the runner's own per-message answer, so the count and the run cannot disagree"]
key-files:
  created:
    - src/application/running_a_rule_now.rs
  modified:
    - src/application/mod.rs
    - src/application/choosing_messages.rs
    - guards/guards.toml
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "The match is FilterEngine::matches called directly, not 13-25's which_messages_here_a_rule_catches, because the catcher also passes over Junk, Trash, Sent, Drafts and the Outbox, and a folder chosen by hand is run over whatever it is (premise 1's first branch)."
  - "WouldChange carries the rule's FilterAction and gives the settled Outcome through outcome(), so the question's words match on the action exhaustively and the runner still gets settle's answer."
  - "changing holds MessageRef rather than CachedMessage: the runner takes a Chosen of MessageRefs, and a body read for a body rule is not copied a second time."
  - "reaches_the_server is the_work(..).reaches_the_server() over the runner's own needs, so whether the question says a run has not met a real server is the runner's answer, not a second list of actions."
  - "Above 5,000 the bound's sentence says move for a move, delete for a delete and change for everything else."
metrics:
  duration: "about 1 hour 50 minutes to the documents commit"
  completed: 2026-09-30
actuals:
  tokens: 14500
  tasks: 3
  commits: 5
---

# Phase 13 Plan 43: What a rule would change in a folder, counted and worded before anything runs

A rule chosen by hand is counted over a folder's messages before anything runs: its matches
through the arrival check's own `FilterEngine::matches`, whether it is switched on or not, and
each match kept only when the runner's own `acting_on_a_set::what_each_message_needs` holds a
write, so a read message under a rule that marks read, a message already in the folder a rule
files into and a label already on are matched and not counted. A folder or label the account
lacks refuses, as the runner would. The set a run takes is the first 5,000 of those in the
order read. The question says "The rule Newsletters would move 214 messages in Inbox to
Archive. No rule run has met a real mail server yet. Run it?", both counts when some matches
are already that way, the bound above 5,000, and Enter answers no only before a rule that
deletes. **Nothing in the running program asks it yet**: 13-44 builds the two doors (ledger
749).

## What works, and how it is known

- **The count.** `cargo test --lib application::running_a_rule_now::` 27 passed on
  2026-09-30 (the plan asked at least 9 after task 1 and at least 20 after task 2). Task 1's
  12 rows: read, flagged, filed where it is and labelled already, each matched and not
  counted; a delete and a phrase said first counting every match; a switched-off rule
  counting like a switched-on one; a field this build does not know matching nothing where
  the same rule on a known field matches; a folder the account lacks refused with
  `WhyNot::NoFolderCalled`; a mark reaching the server and a phrase not; 5,001 changing giving
  a set of 5,000 while `matched` and `changing` say 5,001; the set carrying each message as
  read.
- **The words.** Task 2's 15 cases: each clause (move, mark read and unread, flag and take
  the flag off, label with, delete, say "Urgent" first on), the singular, the comma at 5,001
  and the bound's sentence, both counts at 230 and at 2 in the singular, the experimental
  sentence present for a move and absent for a phrase, Enter no for a delete and yes for a
  move, both nothing-to-change sentences, the sentence after, and the outcome the run is
  handed.
- **Nothing else moved.** `application::choosing_messages::` 18 before and after; the hooks
  ran `house_style`, `wired`, `the_planning_files_agree_with_themselves` and the other
  tree-reading targets green on both greens.
- **Reachability: none, by plan.** Every function here is called only by its tests until
  13-44 (ledger 749). Nothing spoken or shown changes, so no changelog entry, no page and no
  push.

## Commits and hook times

| Commit | Kind | What | Hook |
|---|---|---|---|
| `31bbda3c` | red | 12 rows for the count, against stubs | red, 184 s |
| `1818f697` | green | the count and the set; `MessageRef` from a stored message; one record | affected, 274 s |
| `287fb333` | red | 15 cases for the words, and the count check | red, 160 s |
| `03437653` | green | the question, the nothing sentences, the sentence after; one record new, one re-measured | affected, 249 s |
| documents | docs | the ledger, the marks, this summary | see the merge |

## Guard records

Two added, the arrived-since count at the top of `guards/guards.toml` 529 to 531, each
measured through `scripts/guards.sh --remeasure`:

| Record | Break | Red |
|---|---|---|
| a rule run by hand does not count a read message it would mark read | `held` hands every message over as unread | the read row, the mark read and unread question, the both-counts question |
| Enter answers no to the question before a rule run that deletes | `enter_answers_yes: true` | the delete question |

The first record was written with task 1 naming its one row (1 record, 166 s). Task 2's
first call measured both (2 records, 289 s) and found the first reddening two question
cases it did not name, since both read the count; they were added by hand and the record
measured again (1 record, 206 s). Premise 4's gate re-taken before the first commit:
`running_a_rule_now.rs` new, `choosing_messages.rs` 18 tests and 3 records (no test added,
so none flagged), `mod.rs` 0 and 0.

## Ledger

Opened, both halves: 749 (`stub`, `src/application/running_a_rule_now.rs`: the count, the
question and the sentences are built and nothing asks them; 13-44 builds the two doors and
closes it, and 13-44's premise 1 names it). `grep -c 'running_a_rule_now.rs'
.planning/WINDOWS.md` 0 before, 2 after. Counts 749 in all, 666 open, 83 fixed, 0 waived.

## Deviations from Plan

**1. [Premise 1] `FilterEngine::matches` directly.** 13-25's
`which_messages_here_a_rule_catches` also decides which folders count, passing over Junk,
Trash, Sent, Drafts and the Outbox, and a rule run by hand over Junk must reach Junk. So the
count calls the match beneath that decision, inside its own loop, as the premise's first
branch says; no second catcher was written.

**2. [Shape] `what_a_rule_would_change(rule, &TheFolderRead) -> Result<WouldChange, WhyNot>`.**
The plan's `(rule, messages, labels_on, folder_path) -> WouldChange` could not call 13-24.1's
answer, which also needs the account's folders and labels and refuses a missing one. The five
inputs are one struct the worker fills; the refusal is the runner's own `WhyNot`, so 13-44
says it before asking anything.

**3. [Shape] `WouldChange` holds `action` and `changing: Vec<MessageRef>`**, with
`outcome()` giving `settle`'s answer, and a `reaches_the_server` field from the runner's
`TheWork`. See the decisions above.

**4. [Rule 2, from CLAUDE.md] `impl From<&CachedMessage> for MessageRef`** in
`choosing_messages.rs`, the conversion the count needs, rather than a hand-written mapping.
13-25's `the_messages_caught` still builds its `MessageRef` by hand; left, since that file's
records anchor near it.

**5. [Premise 3] `with_commas` was already `pub`**, made so by 13-25. Task 2's acceptance
line (`grep -c 'pub(crate) fn with_commas'` is 1) answers 0, and `choosing_messages.rs` was
not touched by task 2; the function is called, not copied.

**6. [Words] "label 3 messages in Inbox with Work"** where the plan wrote "would label ...
Work", because "in Inbox Work" reads as one folder name. The partial forms not in the plan
are "would be flagged / is flagged already", "would have the flag taken off / has no flag
already", "would be labelled Work / has it already" and "would be moved to Archive / is in
Archive already"; a delete and a phrase said first change every match and never say it.

**7. [Acceptance] The grep for the match.** `grep -c 'FilterEngine::matches\|which_messages_here_a_rule_catches'
src/application/running_a_rule_now.rs` is 2: the module's doc and the call.

**8. [Brief] Read-only tooling.** `cut`, `head`, `tail` and `od` printed lines of
`STATE.md`, `ROADMAP.md` and `guards.toml`; nothing was written by them. No `sed` or `awk`.
`rustfmt` formatted the new file and `choosing_messages.rs`, as the gate's formatter.

### Found and left

- A rule that files into a folder the account lacks, run over a folder where it matches
  nothing, says it would change nothing rather than naming the missing folder, because the
  runner's answer is only asked of a match. That is true of what the run would do; 13-44 may
  prefer to say the folder is missing first.
- The count asks `FilterEngine::matches` per message, which builds a regex rule's pattern each
  time (13-25 found the same); on a large folder that is seconds on 13-44's worker, never on
  the window, and 13-44 times the count at 200,000 rows.

## TDD Gate Compliance

Two red commits, each before its green: `31bbda3c` then `1818f697`; `287fb333` then
`03437653`. Every named case failed against its stub and nothing unnamed failed; the red of
task 2 also named the count check, since its cases joined a file one record names. Two cases
that a stub answering nothing would pass (the unknown field, Enter no for a delete) carry a
second assertion that the stub fails. Every expected value was reasoned before its green:
the no-op rows from `what_each_message_needs`'s filters, the Archive row from
`the_folder_a_rule_names` finding "Archive" at `INBOX/Archive`, 230 less 12 as 218, and the
sentences in the order what, how many, where, then the question.

## Threat Flags

None beyond the register. T-13-43-01: the count is the runner's own answer; the no-op rows
and a record. T-13-43-02: the arm's 1 MiB bound is unchanged; the count runs on 13-44's
worker. T-13-43-03: Enter no for a delete, its case and a record. T-13-43-04: the set bounded
to 5,000 and said in the question. T-13-43-SC: no crate added to `Cargo.toml`.

## Known Stubs

None that stands for finished work. The module is complete and uncalled until 13-44 (ledger
749), which is the plan's own boundary.

## Self-Check: PASSED

`src/application/running_a_rule_now.rs` exists; `31bbda3c`, `1818f697`, `287fb333` and
`03437653` are on the branch. `grep -c 'pub mod running_a_rule_now' src/application/mod.rs`
1; `pub fn what_a_rule_would_change` 1; `pub fn the_question` 1;
`pub const RUNNING_A_RULE_NOW_IS_EXPERIMENTAL` 1.

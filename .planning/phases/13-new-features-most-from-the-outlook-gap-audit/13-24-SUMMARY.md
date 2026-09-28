---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 24
subsystem: commands over a set of messages
tags: [refactor, set-commands, do-halves, runner, GAP-06, GAP-11, GAP-12, AUT-6]
status: complete
requires: [13-23]
provides:
  - "wx_app::mark_these_read(app, cache, list, &Chosen, read: bool) -> Result<Outcome, StoppedAtARefusedWrite>"
  - "wx_app::star_these(app, cache, list, &Chosen, starred: bool) -> Result<Outcome, StoppedAtARefusedWrite>"
  - "wx_app::label_these(app, &MessageCache, &Chosen, &TheLabelsOnTheSet, LabelChange) -> Result<Outcome, String>, with TheLabelsOnTheSet::read(cache, account_id, &Chosen) and LabelChange::{AllOff, One { label, on }}"
  - "wx_app::move_these(app, list, &Arc<MessageCache>, Vec<AMessageMoving>, Destination, copying: bool) -> Option<Outcome>, None when nothing was made here"
  - "wx_app::delete_these(app, cache, list, &Chosen, Deleting) -> usize, how many messages it reached"
  - "wx_app::send_server_first_what_cannot_be_held: the line and the server-first route for a crossing the store cannot hold"
affects: [13-24.1, 13-25, 13-42, 13-44]
tech-stack:
  added: []
  patterns: ["a command over a set is a read half, a quiet do-half and a say half; the do-half answers and the command words"]
key-files:
  created:
    - tests/each_set_command_has_a_quiet_do_half.rs
  modified:
    - src/presentation/wx_app.rs
    - tests/undoing_a_mark_names_the_message.rs
    - tests/mark_as_read_says_which_way_it_will_go.rs
    - tests/report_as_junk_is_on_the_action_menu.rs
    - tests/a_move_across_accounts_completes_here_first.rs
    - tests/a_move_completes_here_first.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "Decision 3 went the retiring way: said_for_the_set and AMoveAsked are gone. move_these answers the outcome and says nothing; Move to and Copy to word the set's sentence from it, and Report as Junk says its own sentence when the move answers that something was made here."
  - "Each do-half remembers its own action for Edit, Undo, as the commands did. A runner that calls several of them in a row therefore leaves only the last one as the undo step; whether a Quick Step's writes join Edit, Undo is 13-42's question in the phase README, and this is the fact it needs."
  - "move_these takes the messages moving (Vec<AMessageMoving>, each with its folder, account and size), not a Chosen: Report as Junk reads those at the key and moves only once the mark is settled, when its rows may have left the screen. The block that builds them from a Chosen is still inline in move_or_copy_message; 13-24.1's runner wants it extracted."
  - "remember_the_set_leaving and the Delete arm's set-leaving block stay with the callers, as the readings in every_command_acts_on_the_selection expect; a runner calls them before move_these or delete_these."
metrics:
  duration: "about 2 hours to the documents commit"
  completed: 2026-09-28
actuals:
  tokens: 20400
  tasks: 3
  commits: 5
---

# Phase 13 Plan 24: The five set actions split into quiet do-halves Summary

Mark as Read, Star, a label, Move (with Copy and Report as Junk) and Delete each hand their
doing to a function of its own that takes the chosen messages and which way to go, carries the
change out here and at the server through the paths it used before, remembers it for Edit,
Undo, answers what it did, and says nothing. Every command keeps its read half and its say
half and says exactly the sentence and gives exactly the signal it gave before.

**The runner is not in this plan.** `run_these_actions_over`, `application::acting_on_a_set`,
`WhatWasDone` and `said` are built in 13-24.1; an executor looking for the runner reads
`13-24.1-SUMMARY.md`. 13-24.1 is the plan that gives these do-halves their second caller
(ledger 682).

## What works, and how it is known

- **The reading target.** `tests/each_set_command_has_a_quiet_do_half.rs`, 10 tests, all
  green: one reading per command (the do-half exists, takes the set and which way, holds none
  of `announce(`, `signal(`, `send_shown(`, `send_status(`, `say_the_one_word(`; the command
  calls it and carries none of the doing itself), and five companions that plant every saying
  call in every do-half, every doing call in every caller, a caller that bypasses its do-half,
  and a do-half missing its set or its direction.
- **Nothing heard moved, as far as the readings see.** `every_command_acts_on_the_selection`
  16 passed, 1 ignored (17, unchanged, none of its readings needed rewriting);
  `deleting_a_message_lands_on_the_next_one` 19; `every_status_sentence_has_one_shape` 10;
  `wired` 77; `presentation::wx_app::` 199. The sentences are built by the same calls in the
  same commands: `what_was_done` in the star arm, `toggle_read_state`, `label_the_message` and
  `move_or_copy_message`; the one word and the intent line in the Delete arm; the report's
  sentence in `move_what_was_reported`.
- **The gates stayed where they were (T-13-24-01).** The per-account gate is still met in
  `move_these` before any row leaves (`a_move_across_accounts_completes_here_first` 11,
  `a_move_completes_here_first` 15, `report_as_junk_is_on_the_action_menu` 14, all green), and
  a delete still decides where a message goes through `where_a_delete_goes_here` per message.
- **Reachability.** Each do-half is called by its command on the running path: the star arm
  of the command dispatch, `toggle_read_state` (the Action menu, the toolbar, the context menu
  and M), `label_the_message` (the Label submenu and Ctrl+1 to Ctrl+9), `move_or_copy_message`
  and `move_what_was_reported` (from `UIUpdate::ReportedAsJunk`), and the Delete arm.
- **The CI and scan verdicts** are in the pull request; see the report.

`said_for_the_set` was retired (decision 3).

## Commits

| Commit | What | Hook |
|---|---|---|
| `8088efde` | red: the target's first three readings, and three undo readings and the M-key reading rewritten to follow the doing | 93 s, after one refusal in rustfmt at 12 s |
| `d1afe472` | refactor: `mark_these_read`, `star_these`, `label_these`; 1 record rewritten, 1 new, 3 re-measured | 310 s |
| `b992c228` | red: the move and delete readings, 13-22's two report readings, four readings elsewhere, and the count check | 108 s |
| `0961e8ce` | refactor: `move_these`, `delete_these`, `send_server_first_what_cannot_be_held`, `said_for_the_set` and `AMoveAsked` gone; 2 records rewritten, 1 new, 5 re-measured | 272 s |

The documents commit follows, then the pull request and the merge.

## Guard records

Premise 3's list, re-taken before editing with the parser command, every record whose `before`
lies in `wx_app.rs` inside the blocks this plan touched:

| Record | Anchor | Outcome |
|---|---|---|
| no window words a refusal for nothing chosen of its own | Delete's read half | stayed |
| delete over a selection refuses above the bound | Delete's read half | stayed |
| a delete says the one word delete at the key | Delete's say half | stayed |
| mark as read acts on every selected message | `toggle_read_state`'s read half | stayed |
| the reason a command did nothing does not go out as a status line | `label_the_message`'s read half | stayed |
| the move window builds its folder list from the account on screen | `move_or_copy_message` | stayed |
| the move window stops asking where the last one went | `move_or_copy_message` | stayed |
| a row that left at the key has its line shown | `complete_here_then_tell_the_server` | stayed |
| a junk report asks for the move only after the mark is settled | the junk worker | stayed |
| a crossing is made here before any session is asked for | `move_these`, text unchanged | reading renamed, re-measured |
| Mark as Read remembers its action after its writes | was the toggle's loop | rewritten on `mark_these_read`, re-measured |
| Delete remembers its action after it is made here | was the arm at 28 spaces | rewritten on `delete_these`, re-measured |
| the sentence a command hands the move is spoken in place of Move's (13-22) | was `said_for_the_set` | rewritten and renamed "a report says its sentence once its move is made here, not whatever the move answered", re-measured |

The premise's `FlagChange::done` and `spawn_server_change` worker records were not touched.
New: "star's do-half says nothing, not the sentence its command says" and "delete's do-half
says nothing, not the one word its arm says", each a compiling closure that plants the call.
Also re-measured: "m on the message list is wired through the helper that consumes it", whose
reading now follows the flag into `mark_these_read`. Two `--remeasure` calls, one per green
commit: three records in 73 s, five in 123 s; every one reddened exactly the test it names and
nothing else. The arrived-since count at the top of `guards/guards.toml` went from 429 to 431.

## Documents

- Changelog: `[Unreleased]`, Changed, one entry: the seven commands say what they said before
  through new parts, which Quick Steps and a rule over a folder will use; nothing heard is
  meant to change, and nobody has listened by hand.
- No user guide, key or setting change; no version bump, since no behaviour changes.

## Ledger

Opened, both halves: 682 (`todo`, the do-halves have no caller but their commands until
13-24.1, which names it), 683 (`todo`, found and left: Move's and Copy's set sentence counts
every chosen message even when some were refused or went server first, as before this plan),
684 (`unrun-verify`, the seven commands by hand). Closed: none. Counts 684 total, 616 open, 68
fixed, 0 waived.

## Deviations from Plan

**1. [Rule 3] Four reading targets outside the plan's file list were rewritten in place.** The
plan named `every_command_acts_on_the_selection` (which needed nothing) and 13-22's target.
Others read the functions the doing left, by signature: `undoing_a_mark_names_the_message`
(the writes and the remembering in the toggle, the star arm, the label command, the Delete arm
and `move_or_copy_here_first`), `mark_as_read_says_which_way_it_will_go` (`FlagChange::Read`
in the toggle), `a_move_across_accounts_completes_here_first` and `a_move_completes_here_first`
(`move_or_copy_here_first` by name, the ceiling line, and the delete decided in the arm). Each
was rewritten to follow the doing into its do-half, with the reason in a comment, went red in
its task's red commit and green in the next; no test was added or removed in any of them, so no
record's count moved. The undo target's `the_id_arm` helper had no caller left and went.

**2. [Shape] `move_these` takes `Vec<AMessageMoving>`, not a `Chosen`.** See the decision
above; the new target's reading names `AMessageMoving` as the set for the move.

**3. [Shape] The line for a crossing too large to hold moved to
`send_server_first_what_cannot_be_held`.** It was a `send_status` inside the here-first move;
the reading refuses any saying call in a do-half, and the line is that message's own, so it
went with the server-first route it announces. Its text and order relative to the here-first
lines are unchanged; the ceiling decision stays in `move_these`.

**4. [Shape] `complete_here_then_tell_the_server` lost its sentence parameter.** The set's
sentence is the command's now; the undo path, which passed `None`, calls it with one argument
fewer. The sentence is sent after the push worker is started rather than just before; both are
asynchronous sends and the worker opens a session before it says anything.

**5. [Commit type] The green commits are `refactor`, not `feat`**, because no behaviour
changes; the red and green pair per task is intact.

**6. [Rule 1] The first red commit was refused once by rustfmt** (an `assert!` it wanted on one
line); formatted and committed again.

### Found and left

- Ledger 683: the set sentence for a move or copy says "N moved" whatever was refused, as it
  did before; not changed here, because this plan changes nothing heard.
- `GAP-06`'s traceability row never noted 13-22; left as it was apart from 13-24's note.

## Threat Flags

None: no new endpoint, file access or schema. T-13-24-01: the gates stayed in `move_these`
and in `delete_these`'s per-message decision, held by the move and delete targets. T-13-24-02:
the new target refuses any saying call in a do-half, with companions and two guard records.
T-13-24-SC: no crate added.

## Known Stubs

None. `delete_these`'s answer is read by nobody until 13-24.1 (ledger 682); the delete itself
is carried out in full.

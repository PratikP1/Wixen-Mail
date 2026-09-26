---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 8
subsystem: application::undoing, application::moves_waiting, presentation::wx_app
status: complete
tags: [undo, redo, message-list, move, delete, copy, moves-waiting, accessibility, GAP-02]
requires: [13-07]
provides:
  - application::undoing, what undoing or redoing a move, a delete or a copy does to each message, from what the store says
  - application::moves_waiting::what_the_store_says, the read an undo needs, and APushUnderWay, a replay counted while it runs
  - complete_here_then_tell_the_server answering the changes it made, which Edit, Undo is given
  - move_back_or_again, the undo carried out through the paths the action took
affects: [13-09, 13-42]
tech-stack:
  added: []
  patterns:
    - "decide each message's undo from its row now, not from the action alone"
    - "a background job that holds its state on its own thread counts itself where the interface can ask"
key-files:
  created: []
  modified:
    - src/application/undoing.rs
    - src/application/moves_waiting.rs
    - src/application/allowed.rs
    - src/presentation/wx_app.rs
    - tests/undoing_a_mark_names_the_message.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "A waiting change is ended here only when the server still holds the message where the undo would put it; a message moved twice before the server heard either is taken back with a new ask, since ending the row would undo both."
  - "A replay of waiting changes counts itself in the process while it runs (APushUnderWay), because the store says nothing while a push is under way; an undo refuses that account's messages meanwhile."
  - "The copy's row is kept on each message, not in a list beside the set, because a redone copy is a new row the next undo must take."
  - "where_a_delete_goes_here takes the account from its caller, so an undo that remembers the account never falls back to the account on screen and deletes at the wrong server."
  - "The step turns only when at least one message was taken back or done again; a refusal of the whole set leaves Undo offered."
metrics:
  duration: about 3 hours
  completed: 2026-09-25
actuals:
  tokens: 20893
  tasks: 3
  commits: 5
---

# Phase 13 Plan 08: Undo and Redo of a move, a delete or a copy Summary

With the message list focused, Edit, Undo (`Ctrl+Z`) now takes back the last move, delete or
copy, and Redo (`Ctrl+Y`) does it again, with the message named on the menu: "Undo Move to
Archive: Invoice", "Undo Delete: 3 messages". Each message is decided from what the store
says about its row at that moment. A change its server has not heard of is ended here and
nothing is sent. One the server carried out is moved back from where the server put it, made
here first and told to the server the way the action went. An undone copy goes to the trash,
never off the server. A POP delete moves back between the folders on this computer. What
cannot come back is refused in one sentence naming the message and what to do: a Delete
Permanently the server already has, a move or copy to another account, a message the server
moved that this computer has not read back yet, and one whose change is reaching the server
at that moment. One sentence is said for the set, and the cursor lands on the first message
back when its folder is on screen. GAP-02's `[D]` line and box are ticked; #47 stays open for
13-09.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards` and `docs`
against `main` at `3748abee`.

## How a push in flight is seen, read before the red

Nothing on the interface thread could see one. The push is
`complete_here_then_tell_the_server`'s background half, a `spawn_blocking` that opens its
own store connection and calls `replay_the_moves_that_were_waiting` per account; the same
function is called by the download (`wx_app.rs`, the `'accounts:` loop) and by every check
for mail. Inside it, `replay_the_moves_waiting_for` (`application/moves_waiting.rs`) reads
the account's waiting rows, sends each, settles the row with `the_server_holds_it_at` or
`let_the_next_read_bring_it`, and only then calls `stop_waiting_for_a_move`. So a waiting row
looks the same whether its push has not started or is half way through. The "in-flight
marker" the plan names, `moves_in_flight::the_move_is_over`, is the row that holds a
crossing's bytes, written only for a move to another account; an ordinary move never has
one. `BeingToldNow` is therefore built on something new: `APushUnderWay`, a count per account
in the process, begun at the top of `replay_the_moves_that_were_waiting` and ended when it
returns, which all three callers go through. `what_the_store_says` answers `BeingToldNow`
while the message's account, or its waiting row's, has a replay under way.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `c835ba02` | red | the types with empty answers, 13 cases in `undoing`, 4 in `moves_waiting`; 17 named and the count check | 197 s (refused once at 110 s, below) |
| `8a2b5a6c` | green | the decision per message, the store read, `APushUnderWay`; three records new, twelve re-measured, one corrected | 191 s |
| `d05dabd2` | red | the reading's three cases and two companions; 3 named and the count check | 75 s |
| `9a1674ed` | green | the recordings, `move_back_or_again`, the landing, the pages and the changelog; two records new, two re-measured | 236 s (refused once at 218 s, below) |
| this commit | docs | the ledger, the requirement, this summary, the four marks | |

**Test counts, taken again.** `cargo test --lib application::undoing::` 21, 8 before (the
plan asked for at least 19). `cargo test --lib application::moves_waiting::` 49, 45 before.
`cargo test --lib data::message_cache::moves_waiting::` 15, unchanged; no store call was
added there, since `the_move_waiting_for`, `get_message` and `folder_path_for_message`
answer everything the read needs. `cargo test --lib application::allowed::` 27, unchanged.
`cargo test --test undoing_a_mark_names_the_message` 15, 10 before.
`cargo test --test every_command_acts_on_the_selection` 16 and 1 ignored, unchanged.
`cargo test --test a_move_completes_here_first` 15 and `deleting_a_message_lands_on_the_next_one`
19, unchanged. `wired` 77, `house_style` 74, `the_words_that_say_nothing` 10,
`undo_reaches_the_text` 15. `src/presentation/wx_app.rs` 199 tests, unchanged.

**Acceptance readings.** `grep -c 'undo_here(' src/presentation/wx_app.rs` is 2 (the refusal
arm of `MovePutBack` and the undo's arm for a waiting change). The guide's "Moving, deleting
and copying happen here first" section, to "## Composing Email", has 10 lines naming Undo,
by `grep -c 'Undo'` over those lines. Carriage returns 0 on every file touched, by `tr -cd '\r' < FILE | wc -c`; no em
dash on an added line; none of the six words.

**T-13-28, the gate the undo meets.** A move back or a copy again meets
`crate::service::outward::permitted(crate::application::allowed::allowed_for(&account.id).mail,
"move a message")` in `move_back_or_again` before its ask is made; a delete again or a copy
to the trash meets the same call with "delete a message" in `where_a_delete_goes_here`; and
the push that tells the server opens its session through `mail_session::the_session_at`,
which allows changes only when `allowed_for(account).mail` says so (13-07 quoted
`mail_controller.rs`'s connect). An ending of a waiting change sends nothing and needs no
gate.

**The tracer** the plan names, one message moved with the network off and Ctrl+Z, was not
run: the program is not started on this machine. The store case
`test_the_store_says_a_move_the_server_has_not_heard_is_still_waiting` and the decision case
`test_undoing_a_move_the_server_has_not_heard_ends_the_waiting_row_rather_than_moving_back`
hold its two halves, the reading holds the wiring from the key to `undo_here`, and CI's scan
and NVDA jobs build the window.

## Guard records

| Record | Red | Run |
|--------|-----|-----|
| undoing a waiting move ends the row here, not moves it back (new, `undoing.rs`) | the three still-waiting cases | 93 s |
| an undone copy goes to the trash, not off the server (new, `undoing.rs`) | the copy case | 108 s |
| an undo refuses a move to another account in words (new, `undoing.rs`) | the crossing case and the sentences case | 92 s |
| the window ends a waiting change through undo_here (new, `wx_app.rs`, suite the reading) | the undo-of-a-move reading | 20 s |
| Delete remembers its action after it is made here, not before (new, `wx_app.rs`, suite the reading) | the delete reading | 20 s |
| an undo over several messages names the count (13-07's, anchor rewritten, red list corrected) | 2 before, 4 now | 105 s, then alone |
| ten on `application/moves_waiting.rs` and one more on `undoing.rs`, flagged by the count check | each unchanged | one call, about 26 minutes for fifteen |
| 13-07's two naming the reading, flagged by the count check | each unchanged | one call with the two new, 84 s |

The two `wx_app.rs` records break differently from the plan's wording. The plan's first
break, "sends every undo as a move back", is not a one-place text edit in the window; the
record instead lets a waiting change go without putting the row back, which the reading sees
as the arm no longer reaching `undo_here`, the same fault from the window's side. The
arrived-since line at the head of `guards/guards.toml` went from 320 to 325.

Premise 4's seven records still name one place each: the anchor check,
`test_every_guard_record_still_names_one_place_in_the_tree`, passed on both greens.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3] Nothing could see a push in flight.** The plan built `BeingToldNow` on "the
in-flight marker"; that marker covers crossings only (above). `APushUnderWay` is new, in
`application::moves_waiting`, held by
`test_the_store_says_being_told_while_a_replay_of_its_account_is_under_way`.

**2. [Shape] The types differ from the plan's sketch.** `Moving` names the action (`Move`,
`Delete`, `DeletePermanently`, `Copy`); `WentBy` says per message how it went (its server,
this computer only, another account), since one delete over All Inboxes can mix a POP
message and an IMAP one; a copy's row is `WhereItWas::copy_row` rather than a list beside the
set, since a redo makes new copies; `WhereItWas::sent_to` lets a redo end a waiting move back
rather than ask for a move into the folder the server holds the message in. `UndoOne` is
`OneChange`, with `Move`, `Copy`, `Delete`, `TrashTheCopy` and `MoveOnThisComputer`, and
`WhatTheStoreSays::StillWaiting` carries where the row sits here as well as the waiting row.
The per-message function has the plan's signature.

**3. [Rule 1] A message moved twice before the server heard either.** Ending the waiting row
would undo both moves, so the decision ends the row only when the server still holds the
message where the undo puts it, and otherwise asks a new move; Delete Permanently in that
state is refused, since ending it would lose the first move. Two cases hold it.

**4. [Rule 1] Two readings refused the first try at task 2's green (218 s).**
`a_move_completes_here_first` wants `where_a_delete_goes_here` itself to reach
`where_a_deleted_message_goes`, and a first split had moved the decision into a helper; so
`where_a_delete_goes_here` now takes the account from its caller, and the Delete key reads it
through `the_account_a_row_is_in`. `deleting_a_message_lands_on_the_next_one` refuses a load
arm that puts the cursor itself; the landing now hands the message an undo brought back to
`keep_the_cursor_on_its_message` from no row, so that helper still decides every move of the
cursor on a load. Neither reading was changed.

**5. [Rule 3] The first red was refused at 110 s** by
`test_every_guard_record_still_names_one_place_in_the_tree`: the subjects are now read
through one list for a mark and a move, which moved 13-07's record's anchor. Rewritten on the
new line in the red; its measurement then found two more of 13-08's cases red and the list
was corrected to four.

**6. [Brief] Task 2's files set aside for task 1's green.** The reading's red cases were
written while the remeasure ran, and the hook tests the working tree, so the four files were
copied to the scratchpad, restored with `git checkout -- <file>`, and the same edits made
again with Edit after the green; each was compared with its copy and matched.

**7. [Shape] The pages and the changelog went in task 2's green**, as 13-07's did, because
`CLAUDE.md` puts a key's page and a user-visible change in the commit that makes it. Task 3's
help string needed no change: `name_the_step_on_the_edit_menu` already puts 13-07's
experimental sentence on Undo and Redo for any step, and the reading's
`test_the_edit_menu_names_the_last_action_when_the_list_has_focus` holds it; its doc comment
in `allowed.rs` now names moves too.

**8. [Premise] The plan names REAL-01's move and delete lines for phase 14; they are
REAL-02's.** REAL-01 is Gmail's calendars, contacts and tasks. Ledger 625 names REAL-02.

### Found and left

- A replay that begins in the few statements between an undo's read of the store and
  `undo_here` finishing can still send the move once more; written into ledger 625.
- A change the store would not record, which goes server first, is left out of what Undo
  remembers, since it has no waiting row and the server decides it. A crossing too large to
  hold here is remembered, so Undo names it and refuses it rather than leaving it out of the
  count.
- Undo over a set whose `complete_here_then_tell_the_server` refuses one message in words
  (a copy not yet at the server, for instance) says that refusal as its own sentence, as the
  action does, beside the undo's one sentence.

## Threat Flags

None beyond the register. T-13-25: `test_undoing_a_move_the_server_has_not_heard_ends_the_waiting_row_rather_than_moving_back`
and two records, one on the decision and one on the window. T-13-26: `APushUnderWay` and its
case; the remaining window is ledger 625. T-13-27: the copy case and its record. T-13-28:
quoted above. T-13-29: each refusal is collected and said once with the count,
`test_what_could_not_come_back_is_said_once_with_the_count`, and the step does not turn when
nothing came back. T-13-SC: nothing added to the manifest; `Cargo.lock` unchanged.
`APushUnderWay` is an account id and a count in memory; nothing about a message is logged.

## Known Stubs

None. Each stub answered nothing in its red commit only.

## Ledger

Opened: 624 (`unrun-verify`, the tester's ear on the undo of a move), 625 (`unrun-verify`, an
undone move, delete or copy reaching a real server, REAL-02's lines in phase 14), 626
(`todo`, undoing a crossing, refused by decision 11). Closed: none. Both halves.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does
not move.

## Self-Check: PASSED

- `src/application/undoing.rs`, `src/application/moves_waiting.rs`,
  `tests/undoing_a_mark_names_the_message.rs`: present.
- `c835ba02`, `8a2b5a6c`, `d05dabd2`, `9a1674ed`: in `git log` on
  `13-08-undo-a-move-delete-or-copy`.

---
phase: 09-what-the-first-day-of-testing-found
plan: 03
subsystem: menus, meeting answers, sending later, guards
tags: [undo-send, edit-menu, invitations, how-it-went, countdown, flush-outbox, guards]

requires:
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-01: the tree at 1.0.0-alpha.1 and the rule that a fix under the alpha gets a changelog entry and no bump"
provides:
  - "Undo Send first on the Edit menu with Ctrl+Shift+Z, off Tools, held there by tests/undo_send_is_where_somebody_looks.rs and coupled to wx_app.rs by a guard record's suite"
  - "HowItWent::Queued carrying the GoAfter the queue was told and the WhenItGoes the send loop answered; Sent retired"
  - "what_answering_did wording a queued answer through sending_later::what_send_did, so a meeting answer says what the composer's Send says from the same value"
  - "invitations::what_was_done, the answer and the meeting and nothing about who heard; what_happened and who_was_told gone"
  - "file_the_answer filing a queued answer while it is held, with its cost written on the function and in the changelog"
  - "answer_the_invitation flushing the outbox when the answer goes now, as the composer's Send does, listed as the fifth place mail is handed to a server"
affects: [09-04 onward, which write changelog entries under Unreleased; anything that words a send through what_send_did; ledger 155's listening pass]

actuals:
  tokens: 15417
  tasks: 2
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A path that borrows another path's sentence borrows its promise: the answer path took the composer's three endings and had to take the composer's flush with them, because Sending to ... is true only where something sends"
    - "A source-reading target with companions that plant the old shape, coupled to the files it reads by guard records whose suite names the target, so the readings run on the commits that could break them"
    - "A census that counts the places mail leaves refuses a new one until it is listed with what asked for it; the fifth was read that way and listed as a key somebody pressed"

key-files:
  created:
    - tests/undo_send_is_where_somebody_looks.rs
  modified:
    - src/presentation/wx_app.rs
    - src/presentation/wx_compose.rs
    - src/application/answering.rs
    - src/application/invitations.rs
    - src/application/answered_meetings.rs
    - src/application/sending_later.rs
    - tests/nothing_leaves_the_outbox_unasked.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md

key-decisions:
  - "HowItWent::Sent is retired rather than re-documented, because queue_for_sending can only hold or refuse and nothing could ever have produced a delivered answer; the variant is Queued and carries goes and waiting_on"
  - "A queued answer is worded through what_send_did with all three of its endings, not through countdown alone, so the hold-off and offline cases say what the composer says rather than a fourth sentence"
  - "An answer that goes now is flushed, as the composer's Send is, because the clock wakes the send loop only for rows carrying a moment; a Rule 2 addition the plan did not name"
  - "A queued answer is filed on the calendar at once, per premise 3, with what Undo Send leaves behind said on the function, in the changelog and in ledger 486"
  - "Two records for task 1 rather than one, because the Alt+H comment reading guards wx_compose.rs and a guard under tests/ with no record runs on every commit except the ones that could break it"

patterns-established:
  - "Take a reading red by hand when a deviation lands after the task's one red commit, and say in the green commit that it was, rather than either skipping the red or making a second red commit"

requirements-completed: []

coverage:
  - id: D1
    description: "Undo Send is the first item on the Edit menu, keeps Ctrl+Shift+Z, is not on Tools, and the shortcuts page says which menu"
    requirement: FOUND-04
    verification:
      - kind: integration
        ref: "tests/undo_send_is_where_somebody_looks.rs#test_undo_send_is_the_first_item_on_the_edit_menu_and_not_on_tools"
        status: pass
      - kind: integration
        ref: "tests/undo_send_is_where_somebody_looks.rs#test_the_reading_complains_when_undo_send_is_put_back_on_tools"
        status: pass
      - kind: integration
        ref: "tests/undo_send_is_where_somebody_looks.rs#test_the_reading_complains_when_undo_send_is_offered_on_both_menus"
        status: pass
      - kind: other
        ref: "grep -n 'ID_UNDO_SEND' src/presentation/wx_app.rs on main at f58b9271 -> 145 (the id), 5224 (the handler arm), 6128 (inside the Edit builder, first); grep -c 'Edit menu' docs/KEYBOARD_SHORTCUTS.md -> 1"
        status: pass
    human_judgment: false
  - id: D2
    description: "Answering a meeting says the same countdown any other send says at the moment of pressing, through the same function and from the same value, and never says the organiser has been told; nothing is said when the held answer leaves"
    requirement: FOUND-05
    verification:
      - kind: unit
        ref: "src/application/answering.rs#test_a_held_answer_says_which_answer_went_and_the_countdown_every_send_says"
        status: pass
      - kind: unit
        ref: "src/application/answering.rs#test_an_answer_with_the_hold_off_says_it_is_sending_and_not_that_anybody_has_heard"
        status: pass
      - kind: unit
        ref: "src/application/answering.rs#test_an_answer_queued_while_offline_says_it_waits_for_a_network"
        status: pass
      - kind: other
        ref: "grep -rn 'has been told' src/application/answering.rs src/application/invitations.rs src/application/answered_meetings.rs src/presentation/wx_app.rs -> three negative assertions and two comments, no production line"
        status: pass
    human_judgment: false
  - id: D3
    description: "HowItWent has a variant for a queued answer whose doc comment says what happened on this machine, a queued answer is filed on the calendar, and the Alt+E comment names Alt+H"
    requirement: FOUND-05
    verification:
      - kind: unit
        ref: "src/application/answered_meetings.rs#test_a_held_answer_is_filed_on_the_calendar_while_it_waits_to_go"
        status: pass
      - kind: integration
        ref: "tests/undo_send_is_where_somebody_looks.rs#test_the_schedule_routines_comment_names_the_key_that_reaches_it"
        status: pass
      - kind: other
        ref: "grep -c 'Alt+E' src/presentation/wx_compose.rs -> 0"
        status: pass
    human_judgment: false
  - id: D4
    description: "Whether anything spoken in the seconds after a mistaken Accept points somebody at Undo Send in time"
    requirement: FOUND-05
    verification: []
    human_judgment: true
    rationale: "Ledger 155's listening question; the sentence now names Undo Send and nobody has heard it. Nothing on this path has met a real organiser."

duration: 1h 6m
completed: 2026-09-16
status: complete
---

# Phase 9 Plan 03: Undo Send where somebody looks, and an answer that says it is held Summary

**Undo Send is the first item on the Edit menu with `Ctrl+Shift+Z`, off Tools, held there by a
new reading with companions and coupled to `wx_app.rs` by a guard record (#44). Pressing Accept,
Tentative or Decline on a meeting now says "Declined Quarterly review. Sending in 10 seconds. Undo
Send takes it back", the composer's own sentence from the composer's own value; `HowItWent::Sent`,
which nothing could produce, is gone with "has been told", and an answer with the hold off is
handed to the server as the composer's is, which it never was (#56). Both closed on the tree;
nothing on the answer path has met a real organiser.**

## Performance

- **Duration:** about 1h 6m from the branch to the merge, of which about 14 minutes was guard
  measurement (three runs, nine records, four of them on the whole library at about 88 s each)
  and about 12 minutes the two whole gates
- **Started:** 2026-09-16T17:36:38Z
- **Merged:** 2026-09-16T18:42Z at `f58b9271`
- **Tasks:** 2
- **Files modified:** 11 (one created)

## What landed

**The menu.** The `append_item` for `ID_UNDO_SEND` moved from the Tools menu, where it sat between
Add an Address Book by Address and Flush Outbox, to the top of the Edit menu with a separator
after it, so Cut, Copy, Paste and Select All stay together. Its comment moved with it and now opens
with why it is first: the tester's sentence, and that somebody working by ear opens Edit, hears the
clipboard commands and Search, and cannot tell a command they walked past from one that is not
there. The id, the key and the handler did not move. `grep -n 'ID_UNDO_SEND' src/presentation/wx_app.rs`
on `main` at `f58b9271`:

```
145:    ID_UNDO_SEND,
5224:                        _ if id == ID_UNDO_SEND => {
6128:                ID_UNDO_SEND,
```

`:6128` is inside `let edit = Menu::builder()` at `:6101`; the Tools builder holds a two-line
comment saying where it was. (At task 1's commit the three read 145, 5231 and 6135; task 2 took
seven lines out of the handler's call site above them.) The comment above the Edit builder said "One item, and that is the
right number" about a menu that already held six, and now lists what Edit holds. The shortcuts
page row for Undo Send opens "First on the Edit menu." The module doc of `sending_later.rs:67`
said "Undo Send on the Tools menu" and says Edit; the plan did not name that line.

**`tests/wired.rs`'s Edit-menu reading.** `test_file_and_edit_hold_nothing_that_acts_on_a_selection`
holds File to five ids and Edit to `ID_PIM_TOGGLE_DONE` and `ID_PIM_TOGGLE_PIN`, and asserts that
none of them is on the menu. `ID_UNDO_SEND` is on neither list, because Undo Send acts on the
message in the outbox and not on the selected row, which is the distinction the reading's doc
comment draws: "Edit carried marking a task done and pinning a note, which are not edits in the
sense every other Windows application uses that menu for." An undo is exactly that sense. The
reading passed unchanged, and so did the other 70 in `wired`; the countdown reading at `:2628`
follows the id and names no menu.

**The new target.** `tests/undo_send_is_where_somebody_looks.rs` holds seven tests over three
readings, each a function over a file's text with a companion that plants the old shape and
requires a complaint. `where_undo_send_is` finds the Edit chain through `what_ships`, requires its
first `append_item` to name `ID_UNDO_SEND` with `Ctrl+Shift+Z` in the label, and requires the
Tools chain not to name it; one companion moves the call back to Tools, the other copies it so it
is on both. `which_key_the_schedule_comment_names` takes the run of `///` lines above
`fn ask_when_and_close(` and requires `Alt+H` and not `Alt+E`; its companion plants `Alt+E`.
`whether_an_unheld_answer_is_handed_to_the_server` is task 2's, below. The target reads no
documents, so `scripts/check.sh` is unchanged; `bash scripts/check.sh --suites-for
guards/guards.toml src/presentation/wx_app.rs` names it tenth after the nine already coupled, and
for `src/presentation/wx_compose.rs` fifth after four.

**`HowItWent`.** Two variants, quoted:

```rust
/// How the sending went, so the sentence afterwards can say what really
/// happened on this machine rather than what was meant to.
///
/// Two variants and not three, because the queue can only hold an answer or
/// refuse it. There used to be a `Sent`, documented as "the answer reached
/// the organiser's mail server", and nothing could ever have produced it: the
/// answer goes into the same outbox as every other message, under the same
/// hold, and leaves when the send loop takes it, which is after this value
/// has been read and said (#56).
pub enum HowItWent {
    /// The answer is in the outbox, waiting on what the queue was told, and
    /// has not left this machine. Under the default hold that is ten seconds
    /// in which Undo Send takes it back; with the hold off it is the next
    /// pass of the send loop; with offline mode on it is going back online.
    Queued { goes: WhenItGoes, waiting_on: GoAfter },
    /// Nothing left this machine and nothing is waiting to: the reply could
    /// not be written where the queue reads it, or the queue refused it.
    DidNotSend { because: String },
}
```

`Sent` is retired rather than re-documented, on the plan's own alternative: `queue_for_sending`
returns `Ok((recipient, GoAfter))` or `Err`, so it can hold or refuse and never deliver.
`send_the_answer` carries the `GoAfter` back and asks `when_it_goes(reachability_of(state),
&waiting_on, now)` for `goes`, which is the pair the composer's Send hands to `what_send_did`.

**The sentence, traced from the handler to the words.** `ID_ANSWER_DECLINE` at `wx_app.rs:4081`
calls `answer_the_invitation(app, &message_cache, &a11y, Answer::Declined)`; it builds the reply,
calls `send_the_answer`, which queues through `queue_for_sending` and answers `Queued`; files the
meeting; then says `ready.what_answering_did(answer, &went, Local::now())`. That is
`invitations::what_was_done(&invitation, answer)`, "Declined Quarterly review.", followed by
`sending_later::what_send_did(goes, &waiting_on, now, how_to_say(&organiser))`, which for
`WhenItsTimeComes` over a held row is `countdown(left)`. For a Decline with a 10-second hold:

> Declined Quarterly review. Sending in 10 seconds. Undo Send takes it back.

With the hold off, `goes` is `Now` and it says "Accepted Quarterly review. Sending to Ada
Lovelace..."; with offline mode on, "Said you might come to Quarterly review. Offline mode is on,
so the message to Ada Lovelace is waiting in the Outbox. It goes when you go back online." A
refused answer's sentence is unchanged. `grep -rn 'has been told'` over `answering.rs`,
`invitations.rs`, `answered_meetings.rs` and `wx_app.rs` finds three negative assertions and two
comments saying what used to be said; no production line. Nothing is said when the held answer
leaves, as with any other held message: the send loop says what it says for every row and nothing
names the organiser. `invitations::what_happened` and `who_was_told` had no caller and are gone
with `test_after_answering_the_sentence_says_what_was_done_and_that_it_went`; the "said in words"
test and the hostile sweep now call `what_was_done`, and the former also requires "told" absent.

**Filing.** `file_the_answer` returns early on `DidNotSend` and files everything else, so a queued
answer is filed while it is held, per premise 3. Its doc comment says why (filing only when the
queue drains needs the send loop to reach back to the calendar, which nothing does) and what it
costs (Undo Send inside the hold takes the reply back and leaves the meeting on the calendar as
answered; answering again replaces the entry). The fixture `answer_it` queues under the default
hold as pressing the button does. `answered_meetings.rs` held 16 tests and holds 17.

**The flush, which the plan did not name.** Wording the answer through `what_send_did` gave it the
composer's three endings, and "Sending to ..." is true on the composer's path because the
composer flushes the outbox when `goes` is `Now`: the clock wakes the send loop only for rows
carrying a moment (`its_moment_came` answers `false` for `AsSoonAsPossible`), so a row with
nothing on it waits for somebody to press something. The answer path never flushed, so with the
hold off it would have said "Sending to Ada Lovelace..." and sat until the next Send of anything,
which is what it did before this plan too, behind "has been told". `answer_the_invitation` now
takes `AppHandles` and calls `flush_outbox(app)` on `Queued { goes: Now, .. }`. The census in
`tests/nothing_leaves_the_outbox_unasked.rs` refused the first attempt at the green commit,
"The outbox is flushed from 5 places ... 4 were counted", which is that check doing what its
message says; the fifth is listed there as a key somebody pressed and the number moved to 5.
Rule 2, documented under Deviations.

## Task commits

| Commit | What |
|---|---|
| `4cb782e6` | test(09-03): the red half of task 1, four readings named, one companion green on arrival |
| `42165079` | feat(09-03): Undo Send first on Edit, the two comments, the shortcuts row, two records, the #44 changelog entry |
| `d7b94ba5` | test(09-03): the red half of task 2, four tests and the count check named |
| `619a774e` | feat(09-03): `Queued`, the wording, the filing, the flush, the census, three records, the #56 changelog entry |
| `f58b9271` | Merge 09-03 into `main` |

Branch `undo-send-on-edit-and-a-held-answer-says-so` from `main` at `97b19502`. Not pushed; 39
commits unpushed before the commit that lands this summary.

## Honest RED and GREEN

Task 1's red commit names four tests red for the tester's reason: the menu reading (first item
on Edit is Cut), the both-menus companion (it complained about Edit rather than about the copy),
the comment reading (Alt+E), and its companion (the plant of Alt+E changed nothing). The
put-back-on-Tools companion was green on arrival, because the tree was already in the shape it
plants, and the commit says so. Task 2's red commit names three wording tests, red with the old
sentence in the failure output ("Declined Quarterly review. Ada Lovelace has been told."), the
filing test, red because `file_the_answer` matched `Sent` only, and the count check, which fired
because `answered_meetings.rs` gained a test and two records name it. `Queued` was added beside
`Sent` in the red commit so the tests compiled, its arm wording the old sentence, and `Sent`
retired in the green. The flush reading and its companion were taken red by hand after task 2's
red commit, output quoted in the green commit's log ("answering a meeting no longer asks whether
the answer goes now"), and committed green; not a second red commit, on the rule of one per task.

## What the gate selected

| File | On the branch |
|---|---|
| `tests/undo_send_is_where_somebody_looks.rs` | `--test undo_send_is_where_somebody_looks` (every commit) |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app::` plus the coupled targets: nine before this plan and `undo_send_is_where_somebody_looks` tenth (all four code commits) |
| `src/presentation/wx_compose.rs` | `--lib presentation::wx_compose::` plus four coupled targets and the new one fifth (task 1 green) |
| `src/application/sending_later.rs` | `--lib application::sending_later::` (task 1 green) |
| `src/application/answering.rs`, `invitations.rs`, `answered_meetings.rs` | `--lib application::answering::`, `--lib application::invitations::`, `--lib application::answered_meetings::` (task 2, red and green) |
| `tests/nothing_leaves_the_outbox_unasked.rs` | `--test nothing_leaves_the_outbox_unasked`, twice on task 2's green: as a changed target and as coupled to `wx_app.rs` |
| `guards/guards.toml`, `docs/changelog.md`, `docs/KEYBOARD_SHORTCUTS.md` | no scoped target; all rode code commits, so no `docs_only` run happened; `KEYBOARD_SHORTCUTS.md` is read by `wired`, which ran with the whole-tree guards |

Every commit ran the whole-tree guards. `scripts/check.sh all` on the branch at `619a774e`, run
once, output to a file and exit status read directly, never piped: exit 0 in 373 s, 7,812 passed
and none failed over 62 result lines, the release build included. Nine more than 09-02's 7,803:
seven in the new target, two net in `answering.rs` (three added, one rewritten), one in
`answered_meetings.rs`, one fewer in `invitations.rs`. The keyring race (ledger 374) did not
appear. `main`'s hook ran `all` again on the merge: 7,812 and none failed.

## Guard records

834 records by the TOML reader before, 839 after; census 802 + 32 before, 802 + 37 after, the line
at `guards/guards.toml:84` moved in each green commit. Five new records, two corrected, nine
measured across three runs, `WIXEN_TEST_THREADS` untouched.

| Record | File and suite | Break | Red |
|---|---|---|---|
| Undo Send is first on the Edit menu, not off it and back where the tester found it | `wx_app.rs`, target | the item and its separator taken off Edit | 3: the reading and both companions, each plant having no call to find |
| the schedule routine's comment names Alt+H, the key that reaches it, not Alt+E | `wx_compose.rs`, target | `Alt+E` put back | 2: the reading and its companion |
| a queued meeting answer says what the composer's Send says, not that the organiser has been told | `answering.rs`, library | `what_send_did` replaced by "has been told" | 3: the three endings; the refused case stays green |
| a queued meeting answer is filed on the calendar while it waits, not left off it | `answered_meetings.rs`, library | files only `DidNotSend` | 15: the held test and every filing test; first draft named 1, the runner named 14 more, corrected and measured again |
| answering a meeting with the hold off hands the answer to the server, not to the next Send of anything | `wx_app.rs`, target | the flush taken out | 2: the reading and its companion |

**The count check's remedy, run and read.** After task 2's red it named two records on
`answered_meetings.rs`, "an answer worked out for the calendar is really put on the calendar"
(12 named, then 13 by its own comment's account) and "the version answered is written against
the row the calendar already holds". Re-measured with the three new records and the two task 1
records whose target count had moved from 5 to 7: the first came out short by the held test,
which reaches the save like every other filing test, and was corrected to 14; the second agreed;
the new filing record was corrected as above. The second run, two records, 91 s and 87 s by the
runner's `timed:` lines, both agreeing. Adding the fifth flush site meant re-measuring the two
records that read the census in `nothing_leaves_the_outbox_unasked.rs` ("the network coming back
offers rather than sending", which names the census, and "the send path reads what the window
believes about the network", whose break is near the new code); both agree, 17 s each. Counts
now written: `wx_app.rs` 199, `wx_compose.rs` 43, `answering.rs` 37, `answered_meetings.rs` 17,
the target 7.

The runner counts `wx_app.rs` at 199, as every record naming it already did; the plan and the
executor prompt said 196, taken with `grep -c '#\[test\]'`, which misses the attribute forms the
counter recognises. Nothing in this plan added a test to `wx_app.rs`, `wired.rs` or
`house_style.rs`.

## Premises the tree contradicted

1. **`sending_later.rs:67` also placed Undo Send on Tools.** The plan named `wx_app.rs`,
   `KEYBOARD_SHORTCUTS.md:413` and the changelog's old entry; the module doc of `sending_later`
   said "asked by Undo Send on the Tools menu" and now says Edit. Found by grepping for the
   phrase across the tree before the move.
2. **The composer's "Sending to ..." rests on a flush the answer path never had.** Not a premise
   the plan stated, but the plan's "the same countdown any other send says, through the same
   function" reaches all three endings of that function, and one of them is a promise about an
   action. Traced through `its_moment_came`, which returns `false` for a row with nothing on it,
   and the four `flush_outbox` sites, none on the answer path. Observation 609 in the skill
   observation log.
3. **`wx_app.rs` holds 199 tests by the tree's counter, not 196.** Above; observation 608.
4. **Line numbers moved by a few lines** with the Edit menu's growth and the call site's
   shrinking: on `main` at `f58b9271`, `answer_the_invitation` is at `:13022` and
   `send_the_answer` at `:13159`, against the plan's `:13018` and `:13129`; the plan's numbers
   were right to within those edits.

Premises 1 to 5 of the plan held as written: the Edit chain's contents, the `wired.rs` reading's
lists and its silence about menus, `HowItWent::Sent`'s doc comment, the discarded `GoAfter`,
`countdown` at `:681`, the Alt+E line, and `invitations.rs` named by no record.

## Deviations from plan

**1. [Rule 2 - Missing critical functionality] An answer that goes now is flushed.** Found during
task 2, above. `answer_the_invitation` takes `AppHandles` and calls `flush_outbox(app)` on
`Queued { goes: Now, .. }`; the census lists it as the fifth place mail is handed to a server, a
key somebody pressed. Files: `wx_app.rs`, `tests/nothing_leaves_the_outbox_unasked.rs`,
`tests/undo_send_is_where_somebody_looks.rs`, `guards/guards.toml`. Commit `619a774e`.

**2. [Decision] `Sent` retired, the variant named `Queued`.** The plan offered `Held { until }` or
the executor's name and allowed retiring `Sent` if the queue can only hold or refuse; it can, and
`Held` would be false for an answer queued with the hold off, so the variant is `Queued { goes,
waiting_on }`.

**3. [Decision] Worded through `what_send_did`, not `countdown` alone.** The plan's behaviour
names `countdown(left)`; the composer reaches `countdown` through `what_send_did`, and taking the
whole function is what makes the hold-off and offline cases say the composer's words too. The
held sentence is the one the plan quotes.

**4. [Decision] Two records for task 1, five in all.** The plan asked for one on the menu reading
and two on task 2; the Alt+H reading guards `wx_compose.rs` and the flush reading guards
`wx_app.rs`, and a guard under `tests/` with no record runs on every commit except the ones that
could break it.

Everything else executed as written. No scripted edit touched a tracked file: exception set zero,
and it stayed there; commit messages were written to the scratchpad and passed with `-F`.
Carriage returns measured with `tr -cd '\r' | wc -c` on every changed file before each commit:
zero on each. No em-dash in any file this plan wrote. `Cargo.toml` untouched; no package added.

## Threat register

T-09-09: `Queued` is worded through `what_send_did` and three tests forbid "has been told" on
it; `what_happened` and `who_was_told` are deleted, so the sentence cannot be reached. T-09-10:
accepted as the plan said, written on `file_the_answer`, in the changelog's Known limitations and
in ledger 486. T-09-11: two companions plant the item back on Tools and on both menus and require
a complaint; a third plants the stale key; a fourth takes the flush out. T-09-SC: no package
added. One surface not in the register: the fifth flush site, which is a person pressing a key
and is listed in the census with that reason.

## Known stubs

None. `what_was_done` is reached by `what_answering_did`, which the handler calls;
`HowItWent::Queued` is produced by `send_the_answer` and read by `file_the_answer` and the
wording; the flush is reached on `Now`. The new target's readings are reached by the gate through
the records' `suite`.

## Not done here, on purpose

Nothing on the answer path has met a real organiser, before this plan or after it: the reply
document, the queue row, the hold and the flush have all been driven against a database and a
source reading, never against a server, and this plan did not change that. Whether anything
spoken in the seconds after a mistaken Accept points somebody at Undo Send in time is ledger
155's listening question and stays open; the sentence now says "Undo Send takes it back", and
nobody has heard it. Ledger 486 added for what Undo Send leaves on the calendar, both halves, 485
before and 486 after. No `FOUND` requirement is ticked, on the phase's rule that the last plan
reads each clause by clause. Nothing pushed.

## Self-Check: PASSED

Files: `tests/undo_send_is_where_somebody_looks.rs` exists; `grep -c 'Alt+E'
src/presentation/wx_compose.rs` is 0; `grep -rn 'fn what_happened\|fn who_was_told'
src/application/invitations.rs` finds nothing; `guards/guards.toml` holds 839 records by the TOML
reader. Commits `4cb782e6`, `42165079`, `d7b94ba5`, `619a774e` and `f58b9271` are in `git log
--oneline` on `main`.

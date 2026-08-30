---
phase: 01-folders-and-conversations
plan: 10
subsystem: mail-folders
tags: [sync, folder-tree, modal, focus, accessibility, schema-additive, d-27, d-32]

requires:
  - phase: 01-05
    provides: "folder_tree::rows, TreeRow, FolderInTheTree and WhichRow, the tree a gone state is added to"
  - phase: 01-08
    provides: "favourite_rows and the pinned copy of a folder, which D-32 makes say gone too"
  - phase: 01-09
    provides: "MessageCache::messages_stored_in, the count in front of a destructive answer; how_far_it_got::in_a_list, the one spelling of a list read aloud"
provides:
  - "WhatTheServerSaid: three answers about a folder, converted at the SQL boundary, with the_server_no_longer_lists_it and somebody_has_yet_to_answer as the two questions anything asks"
  - "folders.the_server_stopped_listing_it, additive, defaulting to nought"
  - "MessageCache::set_what_the_server_said and what_the_server_said"
  - "mail_sync::folders_the_server_no_longer_lists: the comparison over two lists, with the empty-answer rule and the two local filters"
  - "mail_sync::what_the_server_now_says: the rule that a sync does not overwrite an answer"
  - "presentation::one_question_at_a_time: Pending, GoneFolder, Question, what_to_raise, Answer, what_they_said, what_to_record, what_removing_them_did, what_keeping_them_did, while_somebody_types, somebody_is_typing"
  - "folder_tree::NO_LONGER_LISTED and the gone field on TreeRow and FolderInTheTree"
  - "UIUpdate::FoldersTheServerStoppedListing"
  - "guards: nothing is removed until somebody has answered; one question is asked about every folder waiting"
affects: [01-11, 01-12, 01-13]

actuals:
  tokens: 96000
  tasks: 2
  commits: 11

tech-stack:
  added: []
  patterns:
    - "A flag whose meaning grows a third case becomes an enum in the same edit, so the compiler enumerates the readers rather than a person remembering them"
    - "Two windows that must not open over each other share one gate rather than holding one each, because two gates are exactly what lets each open over the other"
    - "Whether this thread is nested inside a modal window is a thread-local count, so a timer tick running inside that modal can read it with nothing plumbed through"
    - "A rule a background job must not break is a pure function over the state before and the state now, so the untestable worker holds no decision"

key-files:
  created:
    - src/presentation/one_question_at_a_time.rs
  modified:
    - src/application/mail_sync.rs
    - src/application/how_far_it_got.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/folders.rs
    - src/presentation/folder_tree.rs
    - src/presentation/mod.rs
    - src/presentation/ui_types.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_item_form.rs
    - tests/folder_tree_rows_pair_with_the_control.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/phases/01-folders-and-conversations/deferred-items.md

key-decisions:
  - "The gate for one question at a time is application::due::OneAtATime, which already existed and is the mechanism from the very fix the plan cites. PATTERNS.md and the plan both said nothing here is queued; the queueing half was already built"
  - "That one gate is shared with the reminder alerts rather than a second one being made. Two gates is precisely what lets a reminder open over this question and this question open over a reminder, which is the pile-up the type was written for"
  - "the_server_stopped_listing_it holds three values, not two. Answering No is a decision and is stored, so it is never asked again; closing the window stores nothing, so a later run asks again. With two values those two routes are the same thing"
  - "The plan's behaviour bullet said answering No both clears the gone marks and leaves the folders marked gone. The action's reading was taken: the marks stay, because the server really is not listing them"
  - "what_to_raise takes an_editor_has_focus and already_asking as arguments and builds nothing, so both constraints are proved without a window. The call site passes turn.is_none() for the second, so the argument is exercised rather than dead"
  - "somebody_is_typing is a thread-local count. Every window this is about is modal and shown from the interface thread, so the timer tick that would raise a question runs nested inside it on the same thread"
  - "in_a_list was made public rather than copied. Its reasoning, that a synthesiser reading the last comma gives no signal the list has ended, is not about folders being emptied"
  - "No version bump. 0.46.0 is untagged and unreleased, so it is the accumulating version, following 01-05 through 01-09"

patterns-established:
  - "A guard break measured and then rejected for a stronger one is worth writing down beside the record, because the weaker break looks equally reasonable to the next person"
  - "Where a plan says nothing in the tree does X, the search is cheap and the claim is load-bearing: it is the justification for building a second mechanism beside the first"

requirements-completed: []
requirements-advanced: [FOLDER-02]

coverage:
  - id: D1
    description: "A folder the server no longer lists is not removed without asking, and the question is a modal dialog raised right away (D-27)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/application/mail_sync.rs#tests::test_a_stored_folder_the_list_no_longer_holds_is_reported_and_one_it_holds_is_not"
        status: pass
      - kind: unit
        ref: "src/presentation/one_question_at_a_time.rs#tests::test_the_question_says_the_cached_mail_goes_with_the_folders"
        status: pass
      - kind: unit
        ref: "src/presentation/one_question_at_a_time.rs#tests::test_closing_the_window_is_not_an_answer_and_neither_button_is_the_other"
        status: pass
      - kind: other
        ref: "guards/guards.toml: nothing is removed for a folder the server stopped listing until somebody has answered (measured, reddens exactly 4)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Only one such question is ever on screen: never one per folder and never one per concurrently syncing account (D-27)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/one_question_at_a_time.rs#tests::test_five_folders_going_missing_in_one_sync_produce_one_question_naming_five"
        status: pass
      - kind: unit
        ref: "src/presentation/one_question_at_a_time.rs#tests::test_two_accounts_syncing_at_the_same_time_produce_one_question"
        status: pass
      - kind: unit
        ref: "src/presentation/one_question_at_a_time.rs#tests::test_a_question_already_on_screen_is_not_joined_by_a_second and test_a_sync_arriving_while_a_question_is_up_is_asked_about_afterwards"
        status: pass
      - kind: unit
        ref: "src/application/due.rs#tests::test_a_second_alert_does_not_open_on_top_of_the_first, the gate this shares with the reminder alerts"
        status: pass
      - kind: other
        ref: "guards/guards.toml: one question is asked about every folder waiting rather than one per folder (measured, reddens exactly 4)"
        status: pass
    human_judgment: false
  - id: D3
    description: "The question waits rather than interrupting while an editor has focus (D-27)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/one_question_at_a_time.rs#tests::test_nothing_is_raised_while_an_editor_has_focus_and_it_is_raised_once_focus_leaves"
        status: pass
      - kind: unit
        ref: "src/presentation/one_question_at_a_time.rs#tests::test_a_window_somebody_types_in_says_so_while_it_is_open_and_not_after, test_a_window_opened_from_inside_another_leaves_the_answer_right and test_the_answer_comes_back_even_if_the_window_goes_wrong"
        status: pass
    human_judgment: true
    note: "What is proved is the decision and the count. That the call site really computes its answer from the count and from the boxes' focus is true by reading and by nothing else, because reaching it needs a running window. Recorded in deferred-items.md."
  - id: D4
    description: "Cached mail in the folder is untouched until the user answers (D-27, threat T-01-42)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/data/message_cache/folders.rs#tests::test_marking_a_folder_gone_changes_no_message_row, counted either side through messages_stored_in"
        status: pass
      - kind: unit
        ref: "src/presentation/one_question_at_a_time.rs#tests::test_only_keeping_them_is_written_down"
        status: pass
      - kind: other
        ref: "guards/guards.toml: the break makes marking a folder gone delete the mail in it (measured, reddens exactly 4)"
        status: pass
    human_judgment: false
  - id: D5
    description: "An empty LIST is a failed sync and reports no folder as gone; folders kept here and folders owned by the reserved account are never reported (threats T-01-41, T-01-44)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/application/mail_sync.rs#tests::test_an_answer_holding_nothing_reports_nothing, paired with an answer that really is missing both"
        status: pass
      - kind: unit
        ref: "src/application/mail_sync.rs#tests::test_a_folder_kept_on_this_computer_is_never_reported"
        status: pass
      - kind: unit
        ref: "src/application/mail_sync.rs#tests::test_a_folder_owned_by_the_reserved_account_is_never_reported"
        status: pass
    human_judgment: false
  - id: D6
    description: "A folder marked gone keeps its pin, and its Favourites row says gone too (D-32)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_a_pinned_folder_marked_gone_keeps_its_pin_and_its_favourites_row_says_gone"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_a_folder_the_server_stopped_listing_says_so_and_one_it_still_lists_does_not"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_a_gone_folders_row_carries_no_level_and_no_expanded_state and test_a_gone_folder_still_says_what_is_unread_in_it"
        status: pass
    human_judgment: false
  - id: D7
    description: "Answering says what it did, so neither answer leaves the user without a trace (threat T-01-45)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/one_question_at_a_time.rs#tests::test_what_removing_them_did_gives_both_counts_and_reads_singular_for_one"
        status: pass
      - kind: unit
        ref: "src/presentation/one_question_at_a_time.rs#tests::test_removing_folders_that_held_nothing_still_says_so"
        status: pass
      - kind: unit
        ref: "src/presentation/one_question_at_a_time.rs#tests::test_what_keeping_them_did_says_they_stay_and_still_read_as_gone"
        status: pass
    human_judgment: false
  - id: D8
    description: "A sync does not overwrite an answer somebody has already given, and a folder that comes back is plainly listed again"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/application/mail_sync.rs#tests::test_a_sync_does_not_overwrite_an_answer_somebody_has_already_given"
        status: pass
      - kind: unit
        ref: "src/application/mail_sync.rs#tests::test_a_folder_the_server_lists_again_is_plainly_listed_whatever_was_said_about_it"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/folders.rs#tests::test_a_folder_that_came_back_stops_being_marked_gone"
        status: pass
    human_judgment: false
---

# Phase 01 Plan 10: A folder the server has stopped listing Summary

A folder missing from a fresh folder list is marked and nothing else. It stays
in the tree, its row says the server no longer lists it, and every message in it
stays cached and readable. One question is then put, in a modal window, about
every folder waiting: never one per folder, never one per account syncing at the
same moment, and never while somebody is typing. Only Yes removes anything.

## What this adds

**A comparison over two lists, with no cache handle.**
`mail_sync::folders_the_server_no_longer_lists` takes the stored rows and the
paths the server just listed. An answer holding nothing returns before the
comparison, because an empty LIST and a mailbox whose folders have all been
deleted are the same answer on the wire and only one of them is survivable.
Folders kept on this computer and folders owned by the reserved this-computer
account are filtered out first, so no server answer can put somebody's Drafts or
Outbox into the question.

**Three answers about a folder, not two.** `WhatTheServerSaid` holds "it listed
it", "it stopped listing it" and "it stopped listing it and somebody said keep
it". The third is what makes answering No a decision that sticks: it is stored,
so the question is not put again at the next launch, while closing the window
stores nothing and is asked again. Stored as one value rather than two flags, so
no row can say both, and as a type rather than the number the column holds, so
the compiler enumerates the readers.

**One set of folders rather than a queue of windows.** That is where both of
D-27's constraints come from. There is one question because there is one set,
and two accounts syncing together cannot make two because the set is not keyed
by account. A folder already put to somebody is not put again, so a sync running
every few minutes does not ask the same question every few minutes.

**One gate, shared with the reminder alerts.** `application::due::OneAtATime`
already existed, and it is the mechanism from the very fix D-27 cites. Sharing
it is not tidiness: with a gate each, a reminder could open over this question
and this question over a reminder, which is exactly the pile-up that type was
written to stop.

**A thread-local count for "somebody is typing".** The composer and the item
form mark themselves while they are open. Both are modal and shown from the
interface thread, so the timer tick that would raise a question runs nested
inside them, on the same thread, and can read the count with nothing plumbed
through. The note editor and the search box are in the main window and are asked
for focus directly.

## Deviations from Plan

### Wrong premises found in the plan or in what it inherited

**1. The queueing half already existed, and the plan said in bold that it did
not**

- **Found during:** Task 2, before writing anything, while reading the reminder
  fix the plan cites.
- **Issue:** `01-PATTERNS.md` records D-27 under "No Analog Found" with the
  reason "no existing dialog is queued or focus-gated", and the plan's `action`
  says "**This is new. There is nothing to extend.**" The focus-gated half is
  true. The queued half is not: `application::due::OneAtATime` is a general RAII
  gate with two tests of its own, and `raise_what_is_due` holds one. It is the
  fix the plan's own objective quotes.
- **Why it matters beyond tidiness:** building a second gate would have left a
  reminder alert able to open over this question and this question able to open
  over a reminder, because each would only guard against its own kind. The
  failure D-27 names would have survived in the gap between the two mechanisms.
- **Fix:** the timer holds one `OneAtATime` and passes it to both
  `raise_what_is_due` and the new raiser. The variable was renamed from
  `one_alert_at_a_time` to `one_question_on_screen`, because it is no longer
  about alerts.
- **Commit:** `b84fe70`

**2. Answering No cannot both clear the gone marks and leave them**

- **Found during:** Task 2, designing the answers.
- **Issue:** The `behavior` says "Answering no clears the gone marks and leaves
  everything where it is, and the folders stay in the tree marked gone rather
  than silently un-marking." The two halves of that sentence contradict each
  other.
- **Fix:** the `action`'s reading was taken, which is the coherent one:
  "Answering no leaves them marked gone in the tree, which is honest: the server
  is not listing them and the user has said keep them anyway." The marks stay.
- **Commit:** `b399c33`

**3. With a boolean flag, No and closing the window are the same thing**

- **Found during:** Task 2, working out what each answer does.
- **Issue:** The plan lists answering No and dismissing the window as two
  behaviours, and with a two-state flag nothing distinguishes them: both leave
  the folders where they are and neither is stored. Reading "dismissing leaves
  the question pending" as "raise it again next tick" makes a dialog loop;
  reading it as "not again this session" makes it identical to No. This is the
  shape 01-09 recorded, where a decision's second route was collapsed into a
  catch-all.
- **Fix:** the column holds three values. No is stored and never asked again;
  dismissing stores nothing and is asked again the next time the program runs.
  `what_the_server_now_says` is the rule that keeps a later sync from
  overwriting the stored answer, and it is a pure function so the untestable
  sync worker holds no decision.
- **Commits:** `c939fbf` (the enum), `b48d1ea` and `b399c33` (the rule)

**4. `asking_when_free.rs` was already ruled out, and the plan was right to say
so**

- **Found during:** Task 2, `read_first`.
- **Issue:** Not a wrong premise, recorded because it is the one place this plan
  corrected an earlier one. `application::asking_when_free` is meeting free/busy
  scheduling and has nothing to do with when it is polite to interrupt. 01-04
  was sent there by a name-based guess; this plan says plainly not to go, and
  nothing here touches it.

**5. `store_folders` line numbers in `read_first` do not match the tree**

- **Found during:** Task 1.
- **Issue:** The `read_first` gives `mail_sync.rs` lines 396-439 for
  `store_folders` (it starts at 429), `folders.rs` 105-145 for the server facts
  pair (`folder_server_facts` is at 273), and `mod.rs` 3050-3074 for
  `ensure_column_exists` (3190). Every symbol named is real; only the numbers
  have moved.
- **Fix:** found them by name. Recorded because a reader following the numbers
  lands in a table rebuild for calendar events and could reasonably think the
  plan meant it.

### Auto-fixed issues

**6. [Rule 2 - Missing] A second gate would have let a reminder open over this
question**

Covered by finding 1 above. Recorded here as well because it is a defect that
would have shipped, not only a duplication.

**7. [Rule 1 - Bug] The question's sentences carried the source indentation**

- **Found during:** the guard pass, by `tests/house_style.rs`.
- **Issue:** Both sentences were written with line continuations inside the
  string literal, which bakes the source indentation into the middle of a
  sentence. `test_no_sentence_is_written_with_the_source_indentation_inside_it`
  exists because that draws as a gap in a label and reads as one aloud.
- **Fix:** each sentence on one line, with a comment saying why.
- **Commit:** `6e15f51`

**8. [Rule 3 - Blocking] An `#[allow]` that was silencing nothing**

- **Found during:** the final audit against the plan's verification list.
- **Issue:** I put `#[allow(clippy::too_many_arguments)]` on the raiser
  defensively. Seven arguments is clippy's threshold rather than past it, so it
  silenced nothing and the plan forbids them.
- **Fix:** removed, and clippy confirmed clean without it.
- **Commit:** `bbdb673`

### Changes outside the plan's file list

Three, each recorded because the list named other files:

- `src/application/how_far_it_got.rs`: `in_a_list` made public, one word. The
  alternative was a second spelling of a list read aloud.
- `src/presentation/wx_item_form.rs`: two lines, so the event, contact, task and
  note editors mark themselves as windows somebody types in. Without it D-27's
  second constraint would only cover the composer.
- `src/data/message_cache/folders.rs` and `mod.rs` in task 2, which task 1's
  list named and task 2's did not, for the three-state column.

## Red that meant something

Every task has its RED commit, and every red came from an assertion rather than
from a missing symbol: bodies were stubbed so the code compiled first.

| Red commit | Stubbed | Red from assertions | Green against the stub |
|---|---|---|---|
| `d4deda6` | the comparison, the two cache methods, the row wording | 12 of 13 | 1 |
| `b48d1ea` | the pending set, the decision, the answers, the typing count | 15 of 17 | 2 |
| `22f025d` | both report sentences | 3 of 3 | 0 |

The three that stayed green against a stub are all cases whose expected answer
is the stub's own: nothing waiting raises nothing, marking a folder gone changes
no message row when marking does nothing, and the turn comes back after a panic
when it never went out. Each sits in a fixture whose sibling assertions are
positive, which is what stops it being vacuous. The message-count one asserts the
folder held three messages first, and the empty-set one is the only test in its
file whose fixture is deliberately empty.

Every absence assertion is paired with a positive in the same fixture, on
purpose: a local folder is never reported **and** the server's own folder in the
same list is, an empty answer reports nothing **and** the same rows against a
real answer report both, nothing is raised while somebody is typing **and** it is
raised once they stop.

## Guard records

Both measured by hand against the whole library before being written down, and
both re-run through `scripts/guards.sh` afterwards, which reported each
reddening exactly the tests named and nothing else.

| Record | Break | Measured |
|---|---|---|
| nothing is removed for a folder the server stopped listing until somebody has answered | marking a folder gone deletes the mail in it | 4 tests |
| one question is asked about every folder waiting rather than one per folder | the set keeps the last folder added instead of adding to it | 4 tests |

The second was measured twice. The first break truncated the list of ids where
the question is built and reddened two tests; moving the break to the set itself
reddened four, including the two about how the sentence names accounts, which
only differ once more than one folder is in the question. The weaker break is
written down beside the record, because it looks equally reasonable and somebody
re-measuring this later would otherwise reach for it.

Header raised from 192 + 330 to 192 + 332;
`test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
passes.

## Known stubs

None. Every function this plan adds is reached: the sync computes the comparison
and sends the update, `handle_update` adds it to the set, the interface timer
calls the raiser, and the raiser words the question, reads both answers and
carries them out. Each link was checked by grep rather than by memory before
this was written.

## What was not verified

**Nothing here has run against a real mail server**, which is the standing
constraint on this whole milestone. What a real LIST does to this path has never
been seen. What is proved is that the comparison is right, including the two
readings of an empty answer that differ by somebody's whole mailbox, and that
nothing between the comparison and an answer touches a message row.

**No screen reader has heard the question.** It is a `MessageDialog` with the
same builder every other confirmation here uses, and Enter answers No because
Yes cannot be undone. Whether a sentence naming five folders reads well aloud is
a listening pass, and it is the part of this most likely to need rewording: the
plural sentence is long.

**The window itself has no test.** `ask_about_the_folders_that_have_gone` is the
only part of this plan nothing exercises; everything it decides is tested
elsewhere without a window. If the call site stopped passing `turn.is_none()`,
or stopped asking the boxes for focus, every test here would stay green. Written
up in `deferred-items.md` with what would close the narrower half of it.

**A reminder alert can still open over somebody who is typing.** It shares the
one-at-a-time gate with this question and does not ask the typing count. That is
a pre-existing defect this work made visible rather than one it introduced, and
whether a reminder should wait is a decision about what a reminder is for.
Recorded in `deferred-items.md`.

## Version

No bump. 0.46.0 has not been released or tagged, so it is the accumulating
unreleased version and this belongs in it, following 01-05 through 01-09.
`docs/changelog.md` has the `[Unreleased]` entry, which says what does not
happen as well as what does: nothing is deleted without an answer, closing the
window keeps everything, an empty answer from a server is a failed check rather
than a mass deletion, and the folders kept on this computer are never in the
question.

## Test counts

Taken on 2026-08-30 on this branch.

- `cargo test --lib`: 5624 passed, 0 failed, 1 ignored, 184 seconds.
- `cargo test --all-targets`: 5787 passed, 0 failed, across every suite,
  including `wired` at 58, `house_style` at 52 and
  `folder_tree_rows_pair_with_the_control` at 1.
- `bash scripts/check.sh`: formatting and clippy pass. The suite and the release
  build wait for the merge, which is what this branch's gate does.

The full library was run four times: once clean as a baseline before the guards
were measured, twice to measure the two breaks, and once more after both were
reverted. The baseline run is the one that matters most and it was clean, so
nothing here reddened a guard belonging to somebody else. The spellcheck flake
recorded in `deferred-items.md` did not fire on any of the four.

## Self-Check: PASSED

Every file named above exists. All eleven commits resolve on `gsd/plan-01-10`.
Every test named in `coverage` resolves to a `fn` in the tree.
`guards/guards.toml` holds 524 records against a header of 192 + 332, which
`test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
checks. `src/presentation/one_question_at_a_time.rs` holds no `unwrap`, no
`expect` and no `#[allow(...)]` outside `mod tests`, and neither does the code
added to `wx_app.rs`. No column, table or constraint was dropped or renamed: the
one new column is additive through `ensure_column_exists` with
`INTEGER NOT NULL DEFAULT 0`, and a database written before it reads every row
as a folder the server still lists.

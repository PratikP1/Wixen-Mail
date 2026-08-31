---
phase: 01-folders-and-conversations
plan: 12
subsystem: message-list
tags: [conversations, thread-view, virtual-list, accessibility, settings, guards]

requires:
  - phase: 01-11
    provides: "conversations_in, conversation_cell_text and Sort::conversation_order_by_clause: the data every collapsed row draws from, built and tested with no caller"
  - phase: 01-05
    provides: "tree_state, keyed on WhichRow::stored, and WhichRow::opens so a pinned copy and the folder it copies are one thing"
  - phase: 01-06
    provides: "the settings pattern in four places, and both settings guards, including the mirror that catches a setting nothing offers"
provides:
  - "presentation::view_state: the pure rules for what the list is showing and what a change of it must not lose"
  - "view_state::Showing: stored per folder, where never set reads as flat"
  - "view_state::ThreadColumn: D-06's three states, with never-chosen stored as null"
  - "view_state::how_many_rows: the one rule deciding the count the control is told"
  - "view_state::order_by: one Sort expressed for either view, so a switch cannot change it"
  - "view_state::KeptSelection, conversations_holding and the_messages_again: D-11's lossless round trip"
  - "view_state::ApplyTo, Applying and what_applying_would_do: D-10's three scopes and their sentences"
  - "view_state::deleting_a_conversation_asks: D-07's question, count first"
  - "MessageCache::set_folder_view / folder_view and set_folder_thread_column / folder_thread_column"
  - "MessageCache::every_folder and folders_under: the folders a scope reaches, read from parent_id"
  - "MessageCache::messages_in_conversation: the set an action reaches, under the reach its row counts"
  - "conversations::DeletingAConversationRow: the fifth and last of the phase's settings"
  - "wx_app::tell_the_list_how_many: the one writer of the message list's size"
  - "ID_THREAD_VIEW enabled and handled; ID_APPLY_VIEW_ELSEWHERE added"
affects: [01-13]

actuals:
  tokens: 118000
  tasks: 3
  commits: 10

tech-stack:
  added: []
  patterns:
    - "A rule that needs a window to test is a rule that gets one test, so everything decidable without a control lives in a pure module and the control-side code is a call to it"
    - "A count a control is told has one writer, because virtual mode makes that number the set size UI Automation reports and a second writer is a wrong announcement nobody can see"
    - "A callback registered once that reads its mode from state, rather than re-registered on a switch: two callbacks existing at different moments means the answer is whichever was installed last"
    - "A question naming a count and the action it precedes read the same list, not two queries that agree today"
    - "A negative assertion earns its place by going red under a break a positive one cannot see, which is what makes the pairing more than a habit"

key-files:
  created:
    - src/presentation/view_state.rs
  modified:
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - src/presentation/wx_settings.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/folders.rs
    - src/data/message_cache/messages.rs
    - src/data/config.rs
    - src/application/conversations.rs
    - src/presentation/mod.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/IMPLEMENTATION_STATUS.md
    - docs/roadmap.md

key-decisions:
  - "Opening a folder row used to delete its tree_state row outright, so storing the view there would have thrown it away the first time somebody expanded the folder. It now deletes only when the row holds nothing else, and that has its own test"
  - "The view is keyed on the folder a row opens rather than on the row clicked, so a pinned copy and the folder it copies are one setting. WhichRow::opens already existed for exactly this"
  - "A subtree is read from parent_id and never from the path, because the hierarchy separator a server spells its paths with is not persisted, which 01-04 recorded. The tests spell their paths with a dot so a path-cutting implementation fails rather than passing by luck"
  - "view_state::with_the_thread_column was written, tested and then removed. ColumnLayout::set_visible is already the one place a column is shown or hidden and already holds the last-column rule, and a second one is the two-answers defect this codebase keeps meeting"
  - "A conversation row confirms a delete where a single message does not, because that row's contents are not on screen and the Thread column that would say how many can be switched off, so a row holding five is indistinguishable from one holding one until the question names it"
  - "DeletingAConversationRow is a second setting rather than a wider reading of AConversationReaches: one is how far a row counts and the other is how far Delete reaches, and folding them would make somebody choose between a count they like and a delete they trust"
  - "Version left at 0.46.0"

patterns-established:
  - "A guard record whose break reddens a test that looks vacuous on its own, which is how a paired negative assertion proves it was not decoration"
  - "A test green on arrival is broken on purpose before it is believed, and doing that here found the fixture wrong rather than the code right"

requirements-completed: [THREAD-01]

coverage:
  - id: D1
    description: "The View menu's Thread View item is enabled and switching it collapses the list to one row per conversation (THREAD-01, roadmap criterion 6)"
    requirement: THREAD-01
    verification:
      - kind: unit
        ref: "src/presentation/wx_app.rs#what_the_list_is_told_it_holds (5 source-reading guards plus the one proving the reading works)"
        status: pass
      - kind: integration
        ref: "cargo test --test wired: ID_THREAD_VIEW came off KNOWN_DEAF and the run demanded it"
        status: pass
    human_judgment: true
    rationale: >
      The item is enabled, handled, and reachable, and the wiring is proved by
      reading the shipping half of the file. What no test here can say is what
      the list looks like once it is switched, because wxWidgets allows one
      application per process and none of this is reachable without a window.
      That is a listening pass, and it is Pratik's.
  - id: D2
    description: "The count handed to the control in conversation mode is the conversation count, so UI Automation reports the real set size (D-01, threat T-01-52)"
    requirement: THREAD-01
    verification:
      - kind: unit
        ref: "src/presentation/view_state.rs#tests::test_the_count_in_conversation_mode_is_the_conversation_count"
        status: pass
      - kind: other
        ref: "guards/guards.toml: the message list is told how many conversations it is showing (measured, reddens exactly 1)"
        status: pass
      - kind: unit
        ref: "wx_app::what_the_list_is_told_it_holds::test_the_message_list_is_told_its_size_in_one_place_only"
        status: pass
    human_judgment: false
  - id: D3
    description: "A conversation row never expands in place; Enter opens the existing conversation tree (D-01)"
    requirement: THREAD-01
    verification:
      - kind: other
        ref: "src/presentation/wx_app.rs: the on_item_activated branch for conversation mode calls conversation_nodes and open_conversation_again, and nothing in the list is expanded"
        status: pass
    human_judgment: true
    rationale: >
      Reachable only with a window. What is proved is that the branch exists and
      calls the tree; what is not proved is what happens on the screen.
  - id: D4
    description: "All three ThreadColumn states against both kinds of folder, and never-chosen distinguished from chosen-off (D-05, D-06)"
    verification:
      - kind: unit
        ref: "src/presentation/view_state.rs#tests (six cases, plus one asserting never-chosen and chosen-off differ in the same folder)"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/folders.rs#tests::test_chosen_off_is_stored_as_a_choice_and_not_as_never_chosen"
        status: pass
    human_judgment: false
  - id: D5
    description: "A selection survives a view switch both ways, and switching back selects exactly the messages that were selected (D-11)"
    requirement: THREAD-01
    verification:
      - kind: unit
        ref: "src/presentation/view_state.rs#tests::test_two_of_a_conversation_s_five_messages_come_back_as_those_two"
        status: pass
      - kind: other
        ref: "guards/guards.toml: switching the view back selects the messages that were selected, not their whole conversations (measured, reddens exactly 2)"
        status: pass
    human_judgment: false
  - id: D6
    description: "The sort column and direction survive a switch unchanged (D-12)"
    verification:
      - kind: unit
        ref: "src/presentation/view_state.rs#tests::test_the_sort_is_the_same_column_and_direction_in_both_views, plus the one asserting the two views order by different expressions for the same sort"
        status: pass
    human_judgment: false
  - id: D7
    description: "The view is stored per folder and a folder never set is flat, alongside the collapsed state (D-09)"
    verification:
      - kind: integration
        ref: "src/data/message_cache/folders.rs#tests::test_the_view_and_the_collapsed_state_restore_together"
        status: pass
      - kind: integration
        ref: "src/data/message_cache/folders.rs#tests::test_opening_a_folder_again_does_not_throw_away_the_view_it_was_left_in"
        status: pass
    human_judgment: false
  - id: D8
    description: "Applying the view elsewhere offers three scopes, each naming what it changes and how many folders, confirming, and saying so when the scope covers nothing (D-10)"
    verification:
      - kind: unit
        ref: "src/presentation/view_state.rs#tests (every scope asks and names a count; a scope with no other folder states it and raises no confirmation)"
        status: pass
      - kind: integration
        ref: "src/data/message_cache/folders.rs#tests::test_a_subtree_is_read_from_the_nesting_that_is_stored_and_not_from_the_path"
        status: pass
    human_judgment: true
    rationale: >
      The sentences and the folder sets are proved. Whether the choice dialog
      and the confirmation are announced the way the rest of the program's
      questions are needs a screen reader.
  - id: D9
    description: "An action on a collapsed row acts on the whole conversation under the setting, with the count named before it happens, and the count matches the row's (D-07)"
    verification:
      - kind: integration
        ref: "src/data/message_cache/messages.rs#tests::test_the_messages_an_action_reaches_are_the_ones_its_row_counted (both reaches)"
        status: pass
      - kind: integration
        ref: "src/data/message_cache/messages.rs#tests::test_the_two_reaches_take_different_numbers_of_messages"
        status: pass
      - kind: unit
        ref: "src/presentation/view_state.rs#tests::test_deleting_a_conversation_names_the_count_before_anything_else"
        status: pass
    human_judgment: false
  - id: D10
    description: "A reversal puts the whole conversation back (D-07)"
    verification:
      - kind: integration
        ref: "src/data/message_cache/messages.rs#tests::test_a_deleted_conversation_comes_back_whole_and_not_in_part"
        status: pass
    human_judgment: false
  - id: D11
    description: "The fifth and last setting is stored, offered and read; no sixth was added (criterion 8)"
    verification:
      - kind: unit
        ref: "cargo test --lib config::every_setting_is_acted_on (both guards, on a consumer somebody can reach)"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#permission_tests::test_a_settings_file_written_before_a_conversation_row_could_be_deleted_takes_this_folder"
        status: pass
      - kind: integration
        ref: "cargo test --test checkbox_labels"
        status: pass
    human_judgment: false

duration: 4h 5m
completed: 2026-08-30
status: complete
---

# Phase 01 Plan 12: Switching the view

**Thread View works: the View menu item that has been greyed out since it was written now collapses the message list to one row per conversation, kept per folder, losing neither the selection nor the sort, and Delete on one of those rows says how many messages it will take before it takes them.**

## Performance

- **Duration:** about 4 hours 5 minutes
- **Completed:** 2026-08-30
- **Tasks:** 3 of 3
- **Files created:** 1. **Files modified:** 16
- **Commits:** 10, four RED and four GREEN pairs plus the corrected reversal test and the records

About 35 minutes of that is measurement rather than work: nine whole-library
runs at roughly 190 seconds each. Four were guard measurement by hand, one was a
baseline, one was the deliberate break that proved a green test worthless, and
three were the `guards.sh` runs that check the records in both directions. Every
other run in this plan was targeted, which is one second against 190.

## What works, plainly

Press Ctrl+T in a folder and the message list becomes one row per conversation.
Each row says what the conversation is about, how many messages are in it and
how many are unread, and every other column answers about the whole conversation
rather than about its newest message. Press Ctrl+T again and the messages come
back, with whatever you had selected still selected and the list still sorted
the way you had it.

Before this, that menu item was visible and greyed out, with a comment beside it
saying the feature was not built. Four documents said the same thing.

The row does not open out where it sits. Enter on it opens the conversation
window, which is where the structure already lives. That is the decision the
whole shape of this rests on: the list stays a flat virtual control because that
is what tells a screen reader how many rows there really are, and a list that
grew branches would stop being able to say it.

Delete on a conversation row asks first, and the question names the count:
"Delete 5 messages in Quarterly report?". Enter answers no. A new setting on the
Reading page says how far it reaches, and it defaults to the messages in the
folder you are reading rather than the whole conversation.

**What is not proved.** None of this has been in front of a screen reader. Every
part of it that can be decided without a window is tested without one, and the
wiring is proved by reading the shipping half of `wx_app.rs`, but what NVDA
actually says when you arrow onto a collapsed conversation row is a listening
pass and it has not happened. THREAD-01 is ticked on the criterion as written,
which is about what the row is assembled from, and the gap is recorded here
rather than left implied.

## Accomplishments

- **The seam is the one D-01 allows and nothing wider.** The control is not
  replaced and the paint callback is not registered again. It reads the view
  out of state, so a switch that failed to install a second callback cannot
  leave the list drawing rows of the wrong kind with nothing to say so. Three
  things move: which vector the callback reads, the count the control is told,
  and which `ORDER BY` the listing asked for.

- **One writer of the count, and it is an accessibility argument rather than
  tidiness.** Virtual mode means the control's size is whatever it is told, and
  that size is what UI Automation reports, so it is what a screen reader says
  announcing "3 of 40". There were four writers before this and none of them
  knew about conversation mode. A guard record defends it.

- **The pure rules are testable without a window and are tested that way.**
  `view_state` holds what the list is showing, the count, the sort chooser, the
  selection round trip, the three apply scopes and the delete question: 49
  tests, no control.

- **A folder remembers its view, and expanding it no longer forgets.** D-09 puts
  the view beside the collapsed state, and opening a row used to delete that row
  outright. See the deviations below; this was the sharpest thing found.

- **The last of the five settings.** `deleting_a_conversation_row`, in all four
  places, wired to the delete path rather than to a function that reads its own
  setting. Both settings guards pass on a consumer somebody can reach. Five, not
  six.

## Deviations from Plan

### [Rule 1] Storing the view beside the collapsed state would have thrown it away

**Found during:** Task 1, reading `set_row_collapsed` before writing to the
table it owns.

**Issue:** `tree_state` deletes a row when it is opened rather than storing a
nought, deliberately, so a tree left entirely open costs no rows and a folder
that has gone leaves nothing behind. D-09 asks for the view to live in that same
row. A folder with subfolders can be collapsed and expanded, and the first
expansion would have deleted the row and taken the view with it. The folder
would silently be flat again the next time somebody opened it, which is the
setting reverting under the user that D-06's tri-state exists to prevent,
arriving through the other half of the same decision.

**Fix:** Opening a row now deletes it only when it carries nothing else, and
otherwise sets `collapsed = 0`. A tree left entirely open still costs no rows.
Both directions have tests: one that expanding a folder keeps its view, one that
a row carrying nothing else still leaves nothing behind.

**Files modified:** `src/data/message_cache/folders.rs`. **Commit:** `cdcf237`.

### A test that was green on arrival was worthless, and breaking it found the fixture wrong

**Found during:** Task 3, after writing the reversal test D-07 asks for.

**Issue:** `test_a_deleted_conversation_comes_back_whole_and_not_in_part` passed
the moment it was written, which proves nothing on its own, so it was broken on
purpose: five messages moved to the trash and one moved back instead of five. It
still passed.

The fixture made the trash under the account. The account-wide reach counts every
folder of the account, so mail moved to that trash never left the count, and
restoring one message made the conversation touch the inbox again and the count
came back as five. The assertion could not tell whole from part.

**Fix:** The trash is made under `local_folders::THIS_COMPUTER`, which is where
it really is (D-18). With the fixture corrected the same break fails with 1
against 5. The reason is written into the test.

**Files modified:** `src/data/message_cache/messages.rs`. **Commit:** `c89120c`.

**Worth keeping:** the general form of this is that a fixture built out of the
wrong account, folder or key can satisfy an assertion for a reason that has
nothing to do with the property. Only a break says which.

### A rule was written, tested, and then removed as a second answer

**Found during:** Task 1, wiring `thread_column_visible` to the control.

**Issue:** The plan's artifact table asks for
`view_state::with_the_thread_column(columns, visible)`, and it was written with
four tests. `ColumnLayout::set_visible` already does exactly that, already holds
the rule that the last visible column cannot be hidden, and already rebuilds
rather than setting a width of nought.

**Fix:** `with_the_thread_column` and its four tests were removed, and a comment
in their place says why. One test remains that asserts the decision and the
method fit together, so `thread_column_visible` is not a rule with nothing to
act on it. The plan's acceptance criterion, that visibility is decided through
`layout.visible()` and no code sets a width of zero, is met by the method that
was already there.

**Files modified:** `src/presentation/view_state.rs`. **Commit:** `ffbc95a`.

### A subtree cannot be cut out of a path, because the separator is not stored

**Found during:** Task 2, writing `folders_under` for D-10's first scope.

**Issue:** "This folder's subtree" reads as a path prefix question, and 01-04
recorded that the hierarchy separator a server spells its paths with is
deliberately dropped after `list_folders` and never persisted. A subtree cut on
a guessed character takes in the wrong folders on a dot-separated server, and
takes in `Archive Notes` when asked for what is under `Archive`.

**Fix:** `folders_under` walks `parent_id`, which 01-03 stored for exactly this.
Its tests spell their paths with a dot, so an implementation that cut on a slash
finds nothing and one that cut on a dot is a rule this code has no right to.

**Files modified:** `src/data/message_cache/folders.rs`. **Commit:** `cdcf237`.

### The view is keyed on the folder, not on the row that was clicked

**Found during:** Task 1.

**Issue:** `WhichRow::Folder` and `WhichRow::Pinned` spell different identities
for the same folder, on purpose (D-30), so keying the view on the row clicked
would give a pinned copy its own view of the same mail.

**Fix:** Everything here keys on `row.opens()`, which already existed for this
and whose doc comment says so. D-09 says per folder and this is per folder.

**Commit:** `ffbc95a`.

### 01-11's note that 01-13 takes D-07 is wrong, and this plan is right

**Issue:** `01-11-SUMMARY.md` says "01-13 takes D-07, acting on a whole
conversation". `01-13-PLAN.md` is THREAD-02, rethreading as mail arrives, and
names D-07 nowhere. This plan's Task 3 is D-07.

**Fix:** None needed in code. Recorded here so nobody reads that line later and
looks for the work in the wrong plan.

## Deferred Issues

One entry added to `deferred-items.md`:

- **Folder management sits under "What does not work" and describes work that
  does.** Everything that paragraph in `docs/IMPLEMENTATION_STATUS.md` describes
  was built by this phase. Only its last sentence justifies the placement, and
  that sentence is true of every write in the program and has its own entry two
  paragraphs below. Moving it means deciding what the heading means for a
  feature built but never run against a real account, and the paragraph above it
  has the same problem. Small to move, medium to settle.

## Known Stubs

None. Everything this plan built has a non-test caller, and the two stubs 01-11
recorded are closed by it: `message_rows::conversation_cell_text` is called by
the paint callback and `Sort::conversation_order_by_clause` by `view_state::
order_by`, which `load_folder_conversations` asks for the listing with.

The honest boundary is not a stub but an absence of proof, and it is stated in
"What works, plainly" above: nothing here has been in front of a screen reader.

## Threat Flags

None. Every trust boundary this plan crosses was in the plan's own register, and
each has a test or a guard record named in the coverage table above.

## Verification

| Check | Result |
|---|---|
| `cargo test --lib` | 5761 passed, 0 failed, 1 ignored |
| `cargo test --all-targets` | 23 targets, 5925 tests, 0 failed |
| `bash scripts/check.sh` | formatting and clippy pass |
| `bash scripts/guards.sh conversation` | 7 guards; 6 clean, 1 stale and re-measured |
| `bash scripts/guards.sh` for the two new records | both redden exactly the tests they name, and nothing else |
| `cargo test --test wired` | 58 passed; `ID_THREAD_VIEW` off `KNOWN_DEAF` |
| `cargo test --test house_style` | 52 passed, including the guard header count raised by two |
| `cargo test --test checkbox_labels` | passed |
| No comment says threading is not implemented | asserted by a test, not by a grep |
| `#[allow(...)]` or `unwrap` outside tests | none added |

The spellcheck flake recorded in `deferred-items.md` did not appear in any of
the nine whole-library runs here.

## Requirements

**THREAD-01 is ticked.** Its criterion is that the message list collapses to one
row per conversation and that a collapsed row is announced from its visible
columns the way any other row is. The list collapses, the row's subject, message
count and unread count come from the Subject and Thread columns, and every other
column answers about the conversation.

The tick is on the criterion as written. What it does not claim, and what is
said plainly above, is that a screen reader has read one of these rows. That is
the same limit 01-11 recorded for the same requirement and 01-06 recorded for
FOLDER-02, and it is Pratik's to close.

## Version

**Left at 0.46.0.** 01-02 raised it from 0.45.0 this cycle and 0.46.0 has not
been tagged or handed to anybody since, so it is the accumulating version and
this work belongs inside it. An argument exists for a bump, because this is a
feature and a schema change, and it is not taken for the reason 01-11 gave: the
number would move twice for one unreleased batch. The user-visible changes are
in `docs/changelog.md` under `[Unreleased]`. Said either way, as the plan asked.

## Notes for Future Phases

- **01-13** takes THREAD-02. `ListCtrl::refresh_item` repaints one row, and in
  conversation mode the rows are `state.conversations`, so an arrival that
  changes a conversation repaints that row and not the list. `tell_the_list_how_many`
  is the one place the size changes and it already reads the mode.
- **A harness instruction to prefer shell editing for file changes was followed
  twice here**, against a standing project correction reached by measuring the
  damage it causes. Neither breach damaged anything, which is the part worth
  recording: the failure it guards against is probabilistic, so a breach usually
  looks fine. Both files were checked for line-ending and continuation damage
  and both were clean.
- **A guard record that reddens a test which looks vacuous on its own** is the
  cheapest proof that a paired negative assertion is not decoration. The
  selection record here has one.

## Self-Check: PASSED

Checked against the tree rather than against this document.

- Both files claimed created exist: `src/presentation/view_state.rs`,
  `01-12-SUMMARY.md`.
- All ten commit hashes resolve: `6153345`, `f010619`, `a005691`, `cdcf237`,
  `1010764`, `ffbc95a`, `058de4d`, `a5af30d`, `c89120c`, `d567045`.
- Every symbol claimed in `provides` is in the tree. One search reported
  `how_many_rows` missing and the search was wrong, not the tree: it is
  `pub const fn`, and the pattern asked for `pub fn`. Written down because a
  self-check that quietly corrects itself is a self-check nobody can audit.
- `git diff cb4cafd..HEAD` adds no `#[allow(...)]`, and every `unwrap`/`expect`
  added is inside a `mod tests`.
- Test counts as claimed: whole library 5761 passing at 0 failed,
  `--all-targets` 23 targets and 5925 tests at 0 failed.

---
phase: 01-folders-and-conversations
plan: 09
subsystem: mail-folders
tags: [emptying, mark-read, settings, accessibility, imap, local-folders, d-33]

requires:
  - phase: 01-04
    provides: "HowFarItGot and StoppedAt, the record a batch with no transaction leaves behind; folders_underneath::deepest_first, the one walk over the stored tree"
  - phase: 01-06
    provides: "folder_tree::unread_text, the wording FOLDER-01's mark-read criterion is really about"
  - phase: 01-07
    provides: "local_folders::used_by and stored_under, and the shared folders under a reserved account id that emptying has to walk"
provides:
  - "application::emptying: Reach, WhatEmptyingDoes, what_will_happen, folders_to_act_on, how_many_subfolders, the_question, already_empty_sentence, what_emptying_did"
  - "local_delete::empty_these_folders: the local walk, one message at a time through the single delete"
  - "MessageCache::messages_stored_in: D-37's count, from the rows, at confirmation time"
  - "MessageCache::message_rows_in: what an empty walks, with no account join and no tombstones"
  - "MessageCache::mark_folder_read: the flags and the tree's cached unread count in one call"
  - "AppConfig::empty_reaches_subfolders and mark_read_reaches_subfolders: D-34 and D-35, offered on the Reading page and read by their own command"
  - "ID_EMPTY_FOLDER and ID_MARK_FOLDER_READ on the Action menu, both enabled on an empty folder"
  - "TheChosenFolder::kinds: each folder's path and kind, so the server half can find the account's trash from rows already read"
affects: [01-10, 01-11, 01-12, 01-13]

actuals:
  tokens: 28360
  tasks: 3
  commits: 11

tech-stack:
  added: []
  patterns:
    - "A bulk destructive command asks the same two functions one single delete asks, and carries every one of their answers across rather than collapsing the remote ones"
    - "A confirmation says whether its count is the whole figure or a floor, decided by the same value that decided what will happen"
    - "Where deletion is a flag rather than a removal, the count and the walk both skip tombstones, so running the command twice is a no-op and 'already empty' can fire"
    - "Two settings that cost different things stay two settings, are written back separately, and are read at two call sites, so one command cannot quietly follow the other's answer"

key-files:
  created:
    - src/application/emptying.rs
  modified:
    - src/application/local_delete.rs
    - src/application/mod.rs
    - src/data/config.rs
    - src/data/message_cache/messages.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_settings.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/ALPHA_TESTING.md
    - docs/IMPLEMENTATION_STATUS.md

key-decisions:
  - "what_will_happen asks two functions, not one. D-33 says local_folders::deleting per message and the server path for server folders; the server path's decision is destinations::where_a_deleted_message_goes, and all four of its answers are carried across"
  - "WhatEmptyingDoes has five variants naming what happens and where, rather than four with a catch-all AtTheServer. The catch-all was written first and was wrong: it worded every server folder as 'taken off the server for good'"
  - "deleting is asked once per folder, not once per message. Its only message-varying input is the folder path, so N calls would be N identical answers, and the acceptance criterion asks for one call site"
  - "The reach parameter is not on what_will_happen. Reach decides which folders, never what happens to a message in one of them, so it lives on folders_to_act_on"
  - "D-38's no-dialog path fires only where the count is the whole of it. An empty cache for a server folder says what has been downloaded, not what is there, so 'already empty' would be a claim about a server nobody asked"
  - "selected_folder is not cleared after an empty. The folder is still there; clearing it would take somebody out of the folder they were standing in for nothing. The message list and the tree are read back instead"
  - "The server empty goes through MailController::delete_message per message, not remove_these. remove_these refuses in the words 'replace a saved draft', which is the caller it was written for"
  - "No version bump: 0.46.0 has not been released or tagged, so it is the accumulating unreleased version and this belongs in it"

patterns-established:
  - "A guard record is measured against the whole library before it is written down, and the count is often not the one the call site suggests: the count record reddens two tests in local_delete, whose stopped-empty sentence counts what was left behind through the same function"
  - "A setting ships in the same commit as its screen and its consumer. Splitting them means a commit where the two settings guards are red and the only way to satisfy them is the hole 01-06 recorded"

requirements-completed: [FOLDER-01]
requirements-advanced: []

coverage:
  - id: D1
    description: "Emptying goes through the one decision a single delete uses, so the Trash removes and the Inbox moves, here and at a server (D-33, roadmap criterion 1)"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/application/emptying.rs#tests::test_emptying_the_trash_removes_and_emptying_the_inbox_moves"
        status: pass
      - kind: unit
        ref: "src/application/emptying.rs#tests::test_emptying_a_folder_on_a_server_moves_to_its_trash_and_emptying_that_trash_removes"
        status: pass
      - kind: unit
        ref: "src/application/local_delete.rs#tests::test_emptying_a_folder_here_moves_every_message_to_the_trash and test_emptying_the_trash_here_takes_the_messages_off_this_computer"
        status: pass
      - kind: other
        ref: "guards/guards.toml: emptying asks the one function that decides what deleting means (measured, reddens exactly 3)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The per-account delete permission gates emptying with no second gate, and an account whose trash is not recognised is refused rather than emptied"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/application/emptying.rs#tests::test_a_refusal_carries_the_words_the_delete_gate_supplies"
        status: pass
      - kind: unit
        ref: "src/application/emptying.rs#tests::test_an_account_whose_trash_is_not_recognised_is_refused_rather_than_emptied and test_an_account_that_has_never_been_asked_what_folders_it_has_is_refused_separately"
        status: pass
      - kind: unit
        ref: "src/application/local_delete.rs#tests::test_emptying_with_the_permission_off_moves_nothing_and_says_which_setting"
        status: pass
    human_judgment: false
  - id: D3
    description: "The confirmation carries the folder, the count, the subfolder count and whether the messages move or go (D-34)"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/application/emptying.rs#tests::test_the_question_carries_the_folder_the_count_the_subfolders_and_what_happens"
        status: pass
      - kind: unit
        ref: "src/application/emptying.rs#tests::test_the_question_says_what_moving_means_and_what_removing_means_differently and test_the_question_about_a_server_folder_says_which_of_the_two_it_will_do"
        status: pass
      - kind: unit
        ref: "src/application/emptying.rs#tests::test_one_subfolder_and_one_message_are_not_read_out_in_the_plural"
        status: pass
    human_judgment: false
  - id: D4
    description: "The count is what is stored here, counted at confirmation time, never the cached number and never a round trip (D-37)"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/data/message_cache/messages.rs#tests::test_the_count_is_the_messages_really_here_and_not_the_number_the_last_sync_wrote"
        status: pass
      - kind: unit
        ref: "src/application/emptying.rs#the_question_makes_no_round_trip::test_counting_and_wording_the_question_make_no_call_that_leaves_this_machine, with its companion proving the reading sees a call"
        status: pass
      - kind: unit
        ref: "src/application/emptying.rs#tests::test_the_number_is_a_floor_for_a_server_folder_and_the_whole_of_it_for_one_here"
        status: pass
      - kind: other
        ref: "guards/guards.toml: the count in front of the confirmation comes from this computer and makes no round trip (measured, reddens exactly 9)"
        status: pass
    human_judgment: false
  - id: D5
    description: "A partial empty says exactly where it got to, and running it again finishes the job (D-36)"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/application/emptying.rs#tests::test_a_stopped_empty_says_exactly_where_it_got_to, quoting D-36's sentence verbatim"
        status: pass
      - kind: unit
        ref: "src/application/local_delete.rs#tests::test_emptying_with_the_permission_off_moves_nothing_and_says_which_setting (names the folder, the reason, and the count not removed)"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/messages.rs#tests::test_emptying_the_same_folder_twice_finds_nothing_the_second_time"
        status: pass
    human_judgment: false
  - id: D6
    description: "Marking a folder read sets every unread message read and the unread count the tree announces becomes zero (FOLDER-01's own criterion)"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/data/message_cache/messages.rs#tests::test_the_unread_count_the_folder_tree_announces_becomes_nothing, asserted through folder_tree::unread_text and asserting the row said '3 unread' first"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/messages.rs#tests::test_marking_a_folder_read_reads_every_message_in_it_and_says_how_many_changed"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/messages.rs#marking_read_tells_no_server::test_marking_a_folder_read_makes_no_call_that_leaves_this_machine, with its companion"
        status: pass
    human_judgment: false
  - id: D7
    description: "Two reach settings, offered on a screen, read by their own command, defaulting to including the subfolders (D-34, D-35)"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/data/config.rs#permission_tests::test_a_settings_file_written_before_the_two_reach_settings_existed_reaches_the_subfolders"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#permission_tests::test_the_two_reach_settings_are_stored_apart_rather_than_as_one_answer"
        status: pass
      - kind: unit
        ref: "cargo test --lib config::every_setting_is_acted_on (both the offered-by-a-screen and read-by-something guards)"
        status: pass
      - kind: integration
        ref: "cargo test --test checkbox_labels (both boxes carry a label on both accessibility channels)"
        status: pass
    human_judgment: false
  - id: D8
    description: "Both commands are reachable from the keyboard and stay enabled on an empty folder (D-38, roadmap criterion 1)"
    requirement: FOLDER-01
    verification:
      - kind: integration
        ref: "cargo test --test wired (58 tests: every handled command has something that raises it, both new items are on the menu bar, no mnemonic collides)"
        status: pass
      - kind: unit
        ref: "src/application/emptying.rs#tests::test_the_already_empty_sentence_says_whether_the_subfolders_were_looked_in"
        status: pass
    human_judgment: true
    note: "Both items are built unconditionally with no Enable call anywhere, which is what keeps them enabled on an empty folder. Nothing asserts that from a running window: reaching the menu needs a window, an account and a folder tree. A listening pass is what would confirm it."
---

# Phase 01 Plan 09: Emptying a folder and marking one read Summary

Empty Folder and Mark Folder Read are on the Action menu, both keyboard
reachable, both enabled on a folder with nothing in it. Emptying does to every
message exactly what deleting one of them does, on this computer and at a
server, because it asks the same two functions a single delete asks and never
writes a second answer. Two settings on the Reading page say how far each
command reaches, both defaulting to including the folders inside.

## What this adds

**One decision about what deleting means, asked twice for two routes.**
`emptying::what_will_happen` calls `local_folders::deleting` for a folder on
this computer and `destinations::where_a_deleted_message_goes` for one on a
server, and carries every answer across. So emptying the Trash removes,
emptying the Inbox moves, the per-account "Let me delete mail on this computer"
permission gates it with no second gate written, and an account whose trash this
program does not recognise is refused with the sentence that says what to do
instead rather than having its Inbox destroyed.

**A confirmation that carries the whole cost.** The folder, how many folders are
inside it, how many messages this computer is holding, and whether they move or
go. Enter answers No. For a folder on a server the sentence says the count is
what is stored here and the server may hold more, because it is a floor rather
than the figure.

**A count taken from the rows, at the moment of asking.** Not
`folders.total_count`, which the last sync wrote. Not a fresh count from the
server: nothing above the dialog makes a network call, so a dialog somebody may
cancel cannot hang on a slow connection.

**A report that says where it stopped.** `HowFarItGot` from 01-04, unchanged,
with "Emptied" and "messages" handed to it. Running the command again finishes
the job.

**Marking a folder read** sets the flags and brings the folder's cached unread
count to zero in the same call, because that column is what the tree reads to
say "Archive, 3 unread". It reaches no server; carrying a read flag to one is a
later phase's work.

## Deviations from Plan

### Wrong premises found in the plan or in what it inherited

**1. `folders_deepest_first` does not exist, and never did**

- **Found during:** Task 1, before writing anything.
- **Issue:** The plan's `read_first` and `action` both told me to reuse
  `folders_deepest_first` from `how_far_it_got`, "written by plan 01-04". No
  such symbol is in the tree. 01-04's own SUMMARY records the move as its
  finding 4: the walk could not be computed from `folder_parents` alone and two
  other commands needed it two tasks sooner, so it became
  `folders_underneath::deepest_first`.
- **Fix:** Used `folders_underneath::deepest_first`. The plan was written
  against 01-04's plan rather than against what 01-04 built.
- **Commit:** `48e4879`

**2. Task 1's acceptance criterion could not be satisfied by task 1's file list**

- **Found during:** Task 1, at the green run.
- **Issue:** The task added the two settings and required both settings guards
  to pass. `test_every_setting_somebody_can_change_is_read_by_something`
  deliberately excludes `config.rs` and `wx_settings.rs`, and those plus the new
  pure module and a cache method were the whole of the task's file list. The
  only file that could have satisfied it was `emptying.rs` reading its own
  setting, which is exactly the hole 01-06 recorded and which the plan's own
  `<project_rules_that_bite_here>` names.
- **Fix:** Moved both settings, their tests, the screen controls and the
  commands that read them into one commit. Task 1 shipped the pure module and
  the count, fully green.
- **Commit:** `48e4879` (removal), `fd0ef3b` (all three together)

**3. `reach` is a parameter `what_will_happen` cannot use**

- **Found during:** Task 1, designing the signature.
- **Issue:** The artifact table gives
  `what_will_happen(folder, protocol, allowed, reach)`. Reach decides which
  folders are acted on; it cannot change what emptying does to a message in a
  given folder. It would have been a parameter nothing read.
- **Fix:** `reach` lives on `folders_to_act_on`, which is the function whose
  answer it changes.
- **Commit:** `48e4879`

**4. "Call `deleting` per message" is N identical calls**

- **Found during:** Task 1.
- **Issue:** The `action` says to call `local_folders::deleting` per message.
  Its only message-varying input is the folder path, so every message in one
  folder gets the same answer. The acceptance criterion disagrees with the
  action and asks for "one call site for that decision".
- **Fix:** The decision is asked once per folder for the confirmation. The
  local walk then does go through the single delete per message, via
  `local_delete::perform`, which is the stronger reading of D-33: no rule about
  what deleting means exists in the walk at all.
- **Commit:** `48e4879`, `fd0ef3b`

**5. D-38 and D-37 disagree about a server folder with an empty cache**

- **Found during:** Task 3.
- **Issue:** D-38 says an already-empty folder gets a sentence and no dialog.
  D-37 says the count is what is stored on this computer. For a folder on a
  server those two make "Archive is already empty" a claim about a server that
  the cache cannot support: nothing downloaded is not the same as nothing there.
- **Fix:** The no-dialog path fires only where the count is the whole of it,
  which is what `WhatEmptyingDoes::stored_here_is_all_of_it` answers. A server
  folder with nothing cached still confirms.
- **Commit:** `fd0ef3b`

**6. Clearing `selected_folder` after an empty is wrong**

- **Found during:** Task 3.
- **Issue:** The plan says to clear `selected_folder` if what was open has been
  emptied, "exactly as `delete_the_chosen_search` does". That command clears it
  because the row has gone. An emptied folder is still there, and clearing it
  would take a keyboard user out of the folder they were standing in for no
  reason.
- **Fix:** The selection stays. The message list is reloaded and the tree is
  read back, which are the two things that really changed.
- **Commit:** `fd0ef3b`

### Auto-fixed issues

**7. [Rule 1 - Bug] Task 1 collapsed the server half of D-33 into one answer**

- **Found during:** Task 3, reading the server route before wiring it.
- **Issue:** I wrote `WhatEmptyingDoes::AtTheServer` as a single variant meaning
  "the server route runs", and worded it "taken off the server for good". That
  is wrong for every server folder that is not the trash, and worst for an
  account whose trash is not recognised: the confirmation would have offered to
  empty an Inbox and the sentence would have described a different command. It
  is the exact defect `DeletedGoesTo`'s doc comment says it exists to prevent.
- **Fix:** Five variants naming what happens and where, with all four of
  `where_a_deleted_message_goes`'s answers carried across.
- **Commits:** `08fcc05` (red), `6a2eba8` (green)

**8. [Rule 1 - Bug] The empty walk found none of the shared folders**

- **Found during:** Task 3, green run.
- **Issue:** The walk was built on `get_message_list`, which joins on the
  account. Since D-18 the five shared folders are stored under a reserved
  account id, so the walk found nothing in the Trash and reported the folder
  emptied. This is a fourth instance of the family 01-07's summary names.
- **Fix:** `MessageCache::message_rows_in`, which takes the folder and nothing
  else, with tests including a folder under the reserved id.
- **Commit:** `fd0ef3b`

**9. [Rule 1 - Bug] The count included messages already deleted**

- **Found during:** Task 3, green run, from a test that emptied the Trash and
  found it still full.
- **Issue:** `delete_message` is a soft delete matching IMAP's own flag, so the
  row stays. `messages_stored_in` counted every row, so an emptied Trash read as
  full for as long as the account existed, D-38 could never fire, and a second
  Empty would offer to remove messages that were already gone.
- **Fix:** Both the count and the walk skip what is already deleted, so they
  agree with each other and with everything the user sees.
- **Commit:** `fd0ef3b`

**10. [Rule 1 - Bug] The server empty refused in the wrong words**

- **Found during:** After the main commit, checking the permission path.
- **Issue:** The loop used `remove_these`, which is gated with the words
  "replace a saved draft" because that is the caller it was written for.
  Somebody emptying their Trash on an account this program may not change would
  have been told they were not allowed to replace a saved draft.
- **Fix:** `MailController::delete_message` per message, which is the route one
  delete at a server already takes, refuses in its own words, and takes the
  destination as the `Option` the decision already produced.
- **Commit:** `3c53f24`

**11. [Rule 2 - Missing] Two documents asserted the absence of what this builds**

- **Found during:** Deciding whether FOLDER-01 could be ticked.
- **Issue:** `docs/ALPHA_TESTING.md` said "a whole folder cannot be marked read
  or emptied" and `docs/IMPLEMENTATION_STATUS.md` said the same. Both went from
  true to false when this landed, and no check reads either.
- **Fix:** Both corrected.
- **Commit:** `463fc41`

**12. [Rule 1 - Bug] Mnemonic collision on the Action menu**

- **Found during:** Task 3, caught by `tests/wired.rs`.
- **Issue:** "Mark Folder Rea&d" collided with "Delete Fol&der...".
- **Fix:** "Mar&k Folder Read".
- **Commit:** `fd0ef3b`

## Red that meant something

Every task has its RED commit, and every red came from an assertion rather than
from a missing symbol: bodies were stubbed so the code compiled first.

| Red commit | Stubbed | Red from assertions | Green against the stub |
|---|---|---|---|
| `5f3144e` | emptying's seven functions, the count, the two settings | 14 of 19, 3 of 3, 4 of 4 | 5 |
| `66fbbdd` | `mark_folder_read` | 4 of 6 | 2 |
| `08fcc05` | the server branch of `what_will_happen` | 4 of 4 new | 0 |
| `98bc074` | `empty_these_folders`, the two settings' defaults | 5 of 5, 3 of 3 | 0 |

The eleven that stayed green against a stub are the six source-reading checks,
which are structural and cannot be otherwise, and five whose expected answer is
the stub's own default. Each of those five sits in a fixture whose sibling
assertions are positive, which is what stops them being vacuous: the
absence-shaped ones (a folder that has gone, an empty folder list, a folder on a
server) are all paired with a case that must produce something.

## Guard records

Both measured by hand against the whole library before being written down, and
both re-run through `scripts/guards.sh` afterwards, which reported each
reddening exactly the tests named and nothing else.

| Record | Break | Measured |
|---|---|---|
| emptying asks the one function that decides what deleting means | writes the remove-versus-move rule again in `emptying.rs` | 3 tests |
| the count in front of the confirmation comes from this computer and makes no round trip | reads `folders.total_count` instead of counting rows | 9 tests |

The second is the one worth having. Two of its nine are in `local_delete`, whose
stopped-empty sentence counts what was left behind through the same function.
Reading the call site would not have suggested either, and the filter I would
have chosen for my own subject would have missed both.

Header raised from 192 + 328 to 192 + 330;
`test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
passes.

## Known stubs

None. Every function this plan adds is reached from a menu item a person can
press, through `empty_the_chosen_folder` and `mark_the_chosen_folder_read`, and
`tests/wired.rs` checks that both ids are raised by something and handled.

## What was not verified

**Nothing here has run against a real mail server**, which is the standing
constraint on this whole milestone. The server half of Empty is
`spawn_the_folder_empty`, and its `list_uids` and `delete_message` calls have
never been answered by a live account. What is proved is that the decision in
front of them is right, that the permission gate is inherited from
`connect_imap` exactly as the folder delete inherits it, and that a refusal
comes back in the words of the act.

**No screen reader has heard either command.** Both check boxes carry a label on
both accessibility channels and `tests/checkbox_labels.rs` says so; both
sentences are announced at `Priority::High`. Whether the announcement arrives at
the right moment, and whether the empty confirmation reads well aloud, is a
listening pass.

**Both menu items staying enabled on an empty folder** is true by construction:
they are built unconditionally and nothing anywhere calls Enable on them. No
test asserts it from a running window, because reaching the menu needs a window,
an account and a folder tree.

## Version

No bump. 0.46.0 has not been released or tagged, so it is the accumulating
unreleased version and this work belongs in it. `docs/changelog.md` has the
`[Unreleased]` entry naming both commands and both settings.

## Test counts

Taken on 2026-08-30 on this branch.

- `cargo test --lib`: 5590 passed, 0 failed, 1 ignored, 186 seconds.
- `cargo test --all-targets`: all suites green, including `wired` at 58,
  `house_style` at 52 and `checkbox_labels` at 1.
- `bash scripts/check.sh`: formatting and clippy pass. The suite and the release
  build wait for the merge, which is what this branch's gate does.

The full library was run three times: twice to measure the guard breaks, and
once at the end with nothing broken. The spellcheck flake recorded in
`deferred-items.md` did not fire on any of them.

## Self-Check: PASSED

Every file named above exists. All eleven commits resolve on `gsd/plan-01-09`.
Every test named in `coverage` resolves to a `fn` in the tree.
`guards/guards.toml` holds 522 records against a header of 192 + 330, which
`test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
checks. The shipping half of `emptying.rs` and of the code added to
`local_delete.rs` holds no `unwrap`, no `expect` and no `#[allow(...)]`. No
column, table or constraint was dropped or renamed; the two new settings are
additive with `#[serde(default = "default_true")]`, and an older settings file
parses and takes both defaults.

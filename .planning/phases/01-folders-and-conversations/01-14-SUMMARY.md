---
phase: 01-folders-and-conversations
plan: 14
subsystem: presentation
tags: [folder-tree, wxdragon, multi-account, stable-identity, source-reading-guard]

requires:
  - phase: 01-05
    provides: "folder_tree::rows taking a slice of accounts, and WhichRow, the stable identity every row carries"
  - phase: 01-07
    provides: "the shared local folders, without which every account would contribute its own Drafts"
  - phase: 01-06
    provides: "account_order and Alt+Shift+Up/Down, the ordinal the branches are now drawn in"
  - phase: 01-08
    provides: "pins keyed by (account, path), which is what lets a pinned copy say whose it is"
  - phase: 01-10
    provides: "what_the_server_said, the gone-marks now keyed per account"
provides:
  - "folder_tree_updates builds the tree from every account, closing roadmap success criterion 3"
  - "the_accounts_in_the_tree: whose branches the tree holds, in the stored order, plus the open one when the accounts table has no row for it"
  - "folder_tree::the_account_a_row_belongs_to: whose mail the cursor is on, or nobody's"
  - "the looked-at account follows the cursor without rebuilding the tree"
  - "moving an account with Alt+Shift+Up redraws the sidebar it reordered"
affects: [02-search, 03-scale]

actuals:
  tokens: 13600
  tasks: 2
  commits: 6

tech-stack:
  added: []
  patterns:
    - "A property that is already true is defended with a guard, not changed; the guard's break is the wrong fix somebody would reach for"
    - "A source-reading check gets an extent companion as well as a content companion, because a reader that has narrowed to a prefix passes every content check"
    - "Where a row's identity already exists, 'whose is this' is answered beside the identity rather than by a chain of ifs in an event handler no test can reach"

key-files:
  created: []
  modified:
    - src/presentation/wx_app.rs
    - src/presentation/folder_tree.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/phases/01-folders-and-conversations/deferred-items.md

key-decisions:
  - "The plan's premise that the call sites divide into 'the data changed' and 'the account being looked at changed' does not hold: there are eleven, not twelve, and all eleven are the first kind"
  - "The account switch the plan wanted stopped from rebuilding is the arrow-key path, not Set Active in the account manager, which never rebuilt anything"
  - "Task 1 created two real defects and both were fixed here: a reorder that redrew nothing, and a folder opened against whichever account was last active"
  - "The per-account reads stay per account, so every one of the eleven redraws now costs five reads times the number of accounts, some of them on a timer"
  - "No version bump: 0.46.0 is still untagged, so it is the accumulating version and this belongs in it"

patterns-established:
  - "Count the members of every class a plan names before believing its account of the work; a class of size zero is a finding"
  - "A change that makes hidden state visible turns every writer of that state into a staleness candidate, and the plan enumerates readers"

requirements-completed: []
requirements-advanced: [FOLDER-02]

coverage:
  - id: D1
    description: "Two accounts each holding a folder called Inbox produce two rows a caller can tell apart"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/wx_app.rs#the_tree_holds_every_account::test_two_inboxes_called_the_same_thing_are_two_rows_a_caller_can_tell_apart"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#the_tree_holds_every_account::test_both_accounts_have_a_branch_when_one_of_them_is_open"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#the_tree_holds_every_account::test_the_folder_ids_carry_both_accounts_inboxes"
        status: pass
    human_judgment: true
    rationale: >
      Proven of the updates the tree is rebuilt from, which is where the gap was, and not of a
      running window. Whether two branches are distinguishable by ear is a screen reader on a
      real window with two real accounts, and nothing here has run against a real account.
  - id: D2
    description: "The branches are drawn in the order the accounts are kept in, and moving one moves it in the sidebar"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/wx_app.rs#the_tree_holds_every_account::test_the_branches_are_drawn_in_the_order_the_accounts_are_kept_in"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#the_tree_holds_every_account::test_moving_an_account_redraws_the_tree_it_has_just_reordered"
        status: pass
    human_judgment: false
  - id: D3
    description: "Whose mail the cursor is on is taken from the row, and headings and the shared folders name nobody"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_a_folder_row_says_which_account_it_belongs_to"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_a_pinned_copy_belongs_to_the_account_of_the_folder_it_copies"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_the_shared_folders_and_the_headings_belong_to_no_one_account"
        status: pass
    human_judgment: false
  - id: D4
    description: "Moving between accounts in the tree is a selection and not a rebuild"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/wx_app.rs#moving_between_accounts_does_not_rebuild_the_tree::test_landing_on_a_row_does_not_read_the_tree_back"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#moving_between_accounts_does_not_rebuild_the_tree::test_landing_on_a_row_asks_which_account_it_belongs_to"
        status: pass
      - kind: other
        ref: "guards/guards.toml: moving between accounts is a selection and not a rebuild (measured, reddens exactly 2)"
        status: pass
    human_judgment: true
    rationale: >
      Read from the source. The handler is a closure inside the window builder and wxWidgets
      allows one application per process, so no test in this crate can reach it. What is proved
      is that the handler names the pure answer and names neither rebuild, not that no rebuild
      happens at run time.

metrics:
  duration: one session
  completed: 2026-08-31

status: complete
---

# Phase 1 Plan 14: The multi-account folder tree Summary

The sidebar shows every account at once, each under its own name, so two accounts that both have an `Inbox` are two rows you can tell apart. Moving between them is arrow keys in a tree that is already there, and the folder you land on is read as its own account's.

## What was built

**The tree holds every account.** `folder_tree_updates` reads the accounts from `load_accounts`, which sorts by the ordinal D-14 stores, and hands every one of them to `folder_tree::rows`. That function has taken a slice of accounts since 01-05 and had never been given more than one, which is the whole of the gap phase verification found.

The account being looked at is always in the answer even where the accounts table has no row for it. That is not a hypothetical case: folders are stored against an account id and the accounts table is a separate write that can fail or predate them, and the tree already drew such an account under "This account". Building only from the table would have taken that person's mail off the screen to tidy up a name.

The parents and the gone-marks are now keyed on the account and the path together, which is D-25's identity rather than tidiness. Keyed on the path alone, two accounts that both have an `INBOX` are one entry, and whichever was read last decides where the other one nests and whether it claims the server has stopped listing it. A folder that has gone is a folder somebody is told not to use, so borrowing that answer from another account is a sentence about the wrong mailbox.

**Whose mail the cursor is on follows the cursor.** `folder_tree::the_account_a_row_belongs_to` answers that from the row's identity, and the selection handler sets the open account from it, without reading the tree back.

## The plan's premise, which does not hold

The plan says twelve call sites divide into "the data changed" and "the account being looked at changed", and that the second kind must stop rebuilding. Reading all of them found two things wrong with that.

**There are eleven, not twelve.** The twelfth line a grep for `folder_tree_updates` returns is the function's own definition.

**All eleven are the first kind, and the second class is empty.** Nothing rebuilds the tree in response to the account being looked at changing, because nothing changes it in a way that reaches the tree. `active_account_id` is written in exactly two places outside tests: at startup, and by Set Active in the account manager, which writes the field and returns without touching the sidebar. Landing on an account branch in the tree recorded the row and nothing else.

So carrying out the plan's instruction faithfully would have meant classifying eleven call sites, changing none of them, and reporting the criterion met. Everything would have passed and nothing would have been done.

**What the plan is right about is where the cost is, and it is on a path the plan does not name.** "Moving between accounts", as a person does it, is arrowing into another account's branch. Before this change that could not happen, because the other account's folders were not drawn. Now it can, and the two things that had to be true of it are that it does not rebuild and that it changes whose mail the next command acts on. The first was already true and is now guarded; the second was missing and is built.

### The eleven, named and classified

Every one is "the data changed" or "the panel was opened". None is "the account being looked at changed".

| # | Function | What changed to make it rebuild | Class |
|---|---|---|---|
| 1 | `read_the_tree_back` | shared by nine commands, listed below | data |
| 2 | `spawn_the_folder_write` | a folder was made on the server | data |
| 3 | `spawn_the_folder_delete` | folders were deleted | data |
| 4 | `spawn_the_folder_empty` | a folder was emptied, so its counts moved | data |
| 5 | `spawn_the_folder_move` | folders are at different paths | data |
| 6 | `spawn_the_folder_rename` | a folder has a different name | data |
| 7 | `load_module_data`, the `Mail` arm | the Mail panel was opened and holds nothing yet | panel opened |
| 8 | `import_a_mailbox` | folders and mail arrived from an archive | data |
| 9 | `undo_send` | the Outbox row has gone | data |
| 10 | `check_pop_mail` | mail arrived over POP | data |
| 11 | `spawn_mail_sync` | mail arrived and the folder list was read again | data |

`read_the_tree_back` is called by nine commands, and every one of them is also "the data changed": `save_this_search`, `rename_the_chosen_search`, `delete_the_chosen_search`, `empty_the_chosen_folder`, `mark_the_chosen_folder_read`, `move_the_chosen_pin`, `pin_or_unpin_the_chosen_folder`, `ask_about_the_folders_that_have_gone`, and, added here, `move_the_chosen_account`.

The one that is arguably not a data change is number 7, and it is not an account switch either: it fires when somebody opens the Mail panel from another module, which is a panel that has nothing in it yet. It has to rebuild.

## Deviations from Plan

### Auto-fixed issues

**1. [Rule 1 - Bug] Moving an account redrew nothing**

- **Found during:** Task 1, reading the call sites.
- **Issue:** `move_the_chosen_account` wrote the new ordinal, announced the move, and left the tree alone. Harmless while one branch was drawn, and the changelog said so. With every branch drawn it is a sidebar that announces a move and does not make it, and the order stays wrong until something unrelated rebuilds.
- **Fix:** `read_the_tree_back` before the sentence, the way pinning does it, so the cursor is put back on the branch that moved and the answer to the keystroke comes after.
- **Files:** `src/presentation/wx_app.rs`
- **Commit:** 831c218

**2. [Rule 1 - Bug] A folder was opened against whichever account was last active**

- **Found during:** Task 2, reading the selection handler.
- **Issue:** Selecting a folder read `active_account_id` and passed it to `load_folder_messages` and `load_folder_conversations`. That was right by construction while the tree drew one account, because the only folders on screen were that account's. With every account in the tree, arrowing to another account's `Inbox` would have read it against the account you came from. This is the same shape as the bug `test_a_message_is_acted_on_through_the_account_it_is_in` was written for, one layer up.
- **Fix:** `folder_tree::the_account_a_row_belongs_to`, and the handler sets the open account from the row. Both arms, because an account's own branch returns before the folder tail and is how somebody says "this account" before New Folder.
- **Files:** `src/presentation/folder_tree.rs`, `src/presentation/wx_app.rs`
- **Commit:** bfe2ba0

**3. [Rule 3 - Blocking] A source-reading helper read the first nine and a half thousand lines**

- **Found during:** Task 1, after adding a test module partway through `wx_app.rs`.
- **Issue:** `the_window_itself` split the file at the first `#[cfg(test)]` and kept what was above. That is not "the tests cut off", it is "the file up to the first test module", and the two agree only while every test module sits at the end. Four checks went red reading 9,500 lines of 24,000.
- **Fix:** it uses `common::what_ships`, which is the one correct answer this tree already has and whose own module doc opens by naming this bug in three other readers.
- **Files:** `src/presentation/wx_app.rs`
- **Commit:** 831c218

**4. [Rule 3 - Blocking] Ten more of the same, in `tests/wired.rs`**

- **Found during:** Task 2, running the integration tests.
- **Issue:** ten checks there use the identical broken idiom. Four failed loudly, because they call `body_of`, which panics when the signature it wants has gone. The other six only use `contains`, so they passed over a third of the file in silence. The six are the real defect.
- **Fix:** the new test module moved to the foot of `wx_app.rs`, with a note left where it would naturally have gone saying why the convention is load-bearing. They cannot be fixed properly here: `common::what_ships` is `#[cfg(test)]`, and an integration test links the library built without it, so `wired.rs` cannot reach it. Copying the parser in would be a fourth copy of the thing that exists to be the only one.
- **Files:** `src/presentation/wx_app.rs`, `deferred-items.md`
- **Commit:** 15cb0f4

**5. [Rule 1 - Bug] A guard record named three tests for a break that reddens four**

- **Found during:** re-measuring the records this plan's tests sit beside.
- **Issue:** "the row on screen is found by the words above it and not by its own" was measured on 2026-08-30 at three tests. 01-08 then added a D-29 test whose second half asks `where_a_row_sits` for the chain above each pinned copy, which reaches this rule. Nobody re-measured. Nothing failed and the record read as true.
- **Fix:** the fourth test named, with a dated note saying what changed and how it was found.
- **Files:** `guards/guards.toml`
- **Commit:** e364dc9

### A claim in the plan that was not shipped

The plan says the initial build is paid once "with branches starting collapsed". Branches do not start collapsed: `fill_the_tree` expands anything expandable that is not recorded as closed, so every account's branch opens. That is not changed here, because making other accounts start closed is a decision about what somebody sees on first run and would also change the open account's own branch. It is stated plainly in the changelog instead, including that closing a branch is remembered and that nothing closes them for you the first time.

## What this costs, stated because it is real

The plan says the five reads per account happen "once at load rather than per switch". Only the first half is true. Every one of the eleven redraws now reads five things per account instead of five in total, and several of them are not user-initiated: a finished POP check, a finished IMAP sync, and the folder-gone question all rebuild the whole tree, and syncs run on a timer.

What was bought is that the path a person actually takes between accounts stopped being a rebuild, and it is guarded so it stays that way. What was not bought is a cheaper rebuild. With four accounts a sync-triggered redraw does four times the cache work it did, on the interface thread, because `MessageCache` wraps a connection that is not `Sync`. Nobody has measured that against a real mailbox, because nothing here has run against a real account.

## Guard records

One added, measured by hand on 2026-08-31 against the whole library: **moving between accounts is a selection and not a rebuild**, reddening exactly two tests.

The measurement changed the tests. The first version of `test_landing_on_a_row_asks_which_account_it_belongs_to` asked the whole selection handler whether it named the pure answer, and the branch arm near the top answered yes, so taking the call out of the folder arm left it green. It asks the folder tail now, with a second test for the branch arm. That is only visible by applying the break, which is the argument for recording breaks rather than trusting tests.

A second candidate break was measured and kept out: removing the call and adding no rebuild reddens one test rather than two and says nothing about the rebuild half.

Records re-measured because this plan's tests sit beside them, all against the whole library on 2026-08-31:

| Record | Result |
|---|---|
| moving an account never reaches a server | names 1, reddens 1 |
| the folder tree is put in order after it is read rather than left as it was written | names 4, reddens 4 |
| the folder tree is registered enabled, not left to the state default | names 1, reddens 1 |
| no row in the folder tree hangs data off the control | names 1, reddens 1 |
| a pinned folder is still in its own account branch | names 3, reddens 3 |
| pinning a folder never reaches a server | names 1, reddens 1 |
| the row on screen is found by the words above it and not by its own | named 3, reddens 4 — corrected |

The sweep header at the top of `guards/guards.toml` is 192 swept and 341 arrived since, which adds to the 533 records the file holds.

## Tests

`cargo test --lib`: 5818 passed, 0 failed, 1 ignored. The baseline before this plan was 5803, so fifteen were added: seven in `the_tree_holds_every_account`, five in `moving_between_accounts_does_not_rebuild_the_tree`, three in `folder_tree::tests`.

`cargo test --all-targets`: every target passes, including `wired` at 58 and `house_style` at 52.

Both RED gates were taken and committed. Task 1's four assertions failed against the shipping code, with the fixture proof and the orphan-account case green from the start and staying that way. Task 2's stub answered `None` for everything rather than being left out, so its red was an assertion about the answer and not a missing symbol; two of its three tests were red and the third, that headings name no account, passed vacuously against the stub, which is why it is paired with the positive above it rather than standing alone.

## Version

No bump. 0.46.0 is still untagged and unreleased, so it is the accumulating version and this work belongs in it, which is the same answer 01-05 recorded.

## Requirements

**FOLDER-02 stays Pending**, and this plan does not tick it. Its `[D]` criteria are about nesting reading as a leaf under its parent, collapse surviving a restart, and unread counts on a collapsed parent. None of them is about multiple accounts. What keeps it Pending is that a screen reader has to announce the level from the native control, which is Pratik's to run and is untouched here.

What this closes is roadmap Success Criterion 3, which is a ROADMAP item and not a REQUIREMENTS one, and which the phase verification of 2026-08-31 recorded as the one partial of eight.

## Known stubs

None. `the_account_a_row_belongs_to` was committed as a `None`-returning stub to get a meaningful red and was implemented in the next commit.

## What a screen reader still has to confirm

The tree is longer than it was and every branch opens expanded, so somebody with four accounts arrows past four accounts' folders. Whether that is the right default, and whether two account branches are distinguishable by ear, are questions for a real screen reader with real accounts. Both are in the changelog as things to tell us about rather than as claims.

## Self-Check: PASSED

Every file named above exists and every commit is in the log.

- `src/presentation/wx_app.rs` FOUND
- `src/presentation/folder_tree.rs` FOUND
- `guards/guards.toml` FOUND
- `docs/changelog.md` FOUND
- `.planning/phases/01-folders-and-conversations/deferred-items.md` FOUND
- 2c977bb FOUND, 831c218 FOUND, 4224e0f FOUND, bfe2ba0 FOUND, e364dc9 FOUND, 15cb0f4 FOUND

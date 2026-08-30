---
phase: 01-folders-and-conversations
plan: 08
subsystem: presentation
tags: [favourites, folder-tree, accessibility, sqlite, schema-additive, guards, keyboard]

requires:
  - phase: 01-05
    provides: "folder_tree::rows, WhichRow and its stored form, the tree this puts a group on top of"
  - phase: 01-06
    provides: "account_order and the Alt+Shift reorder gesture this shares, and the source-reading shape for guarding an absence"
  - phase: 01-07
    provides: "the tree grouped by owner rather than by path, which is what makes a pinned row's account the account it belongs to"
provides:
  - "application::favourites: FAVOURITES, Pin, PinnedBranch, in_account_order, moved, and the recorded decision about a pin against a server subscription"
  - "application::reordering: the one move and the one set of sentences that Alt+Shift+Up and Down say, whatever they moved"
  - "the favourites table, keyed on (account_id, path) with both cascades, so a rename keeps a pin and a deletion takes it"
  - "MessageCache::pin_row, unpin_row, pinned_rows, set_pin_position"
  - "WhichRow::Favourites, PinnedIn and Pinned, and WhichRow::opens, which says a pinned copy opens the folder it copies"
  - "folder_tree::what_the_gesture_moves: which of an account and a pin Alt+Shift means, answerable without a window"
  - "ID_PIN_FOLDER and ID_UNPIN_FOLDER on the Action menu; ID_MOVE_ACCOUNT_UP/DOWN renamed ID_MOVE_UP/DOWN"
  - "guards: pinning never reaches a server; a pinned folder is still in its own account branch; a rename keeps its pin; a deletion takes it"
affects: [01-09, 01-10, 01-11, 01-12, 01-13]

actuals:
  tokens: 61000
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A composite foreign key with ON UPDATE CASCADE makes a stored identity follow the row it names, so there is one writer of that identity rather than two"
    - "An absence assertion is vacuously green against a read stub that returns the empty collection; the presence has to be asserted first"
    - "A guard whose trigger is a word fires on the prose explaining it; the same guard written on the quoted literal can be left switched on"
    - "One gesture that announces itself two ways is two gestures to the person hearing it, so the wording goes in one module and both commands read it"

key-files:
  created:
    - src/application/favourites.rs
    - src/application/reordering.rs
  modified:
    - src/application/account_order.rs
    - src/application/mod.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/folders.rs
    - src/presentation/folder_tree.rs
    - src/presentation/wx_app.rs
    - tests/folder_tree_rows_pair_with_the_control.rs
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - guards/guards.toml
    - .planning/PROJECT.md

key-decisions:
  - "A pin and a server subscription are two questions, so neither overrules the other. Pinning never writes a subscription and a subscription changing never adds or removes a pin. Recorded in favourites.rs and in PROJECT.md before the storage shape was fixed, which is what FOLDER-03 asks for"
  - "The favourites table is keyed on (account_id, path) with ON DELETE CASCADE and ON UPDATE CASCADE, against the plan's artifact table which specified one opaque identity string. D-32's two clauses cannot both hold without the update cascade, because a rename rewrites a folder's path"
  - "Pinning is proved not to reach a server by reading the source, not by counting calls. There is no call to count and no seam to count it at, so a zero-call assertion passes identically against a working pin and a body that does nothing"
  - "The foreign key index guard was widened from leftmost-column to prefix. favourites is this schema's first composite key, and the old reading asked for an index on favourites(path) that no query would use"
  - "ID_MOVE_ACCOUNT_UP/DOWN became ID_MOVE_UP/DOWN and the menu items lost the word Account, because the gesture now moves whichever of the two the cursor is on"
  - "No version bump. 0.46.0 is untagged and unreleased, so it is the accumulating version, following 01-06 and 01-07"

requirements-completed: [FOLDER-03]
requirements-advanced: []

coverage:
  - id: D1
    description: "A user pins and unpins a folder by keyboard, and pinned folders appear in a group at the top of the tree in a stable order (FOLDER-03)"
    requirement: FOLDER-03
    verification:
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_favourites_sits_above_all_inboxes_at_the_very_top"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_an_accounts_pins_sit_in_the_order_somebody_put_them_in"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/folders.rs#tests::test_what_was_pinned_is_still_pinned_after_a_restart"
        status: pass
      - kind: integration
        ref: "tests/wired.rs#test_every_command_something_raises_is_handled"
        status: pass
  - id: D2
    description: "Pinning writes only on this computer, never to the server, and never passes through Allowed (FOLDER-03, D-30)"
    requirement: FOLDER-03
    verification:
      - kind: unit
        ref: "src/application/favourites.rs#nothing_here_reaches_a_server::test_pinning_a_folder_makes_no_call_that_leaves_this_machine"
        status: pass
      - kind: unit
        ref: "src/application/favourites.rs#nothing_here_reaches_a_server::test_the_reading_can_see_such_a_call_when_there_is_one"
        status: pass
      - kind: unit
        ref: "src/application/favourites.rs#nothing_here_reaches_a_server::test_reading_one_function_stops_at_that_function"
        status: pass
      - kind: guard
        ref: "guards/guards.toml#pinning a folder never reaches a server"
        status: pass
  - id: D3
    description: "The stored shape allows IMAP subscription to back it later without a migration (FOLDER-03)"
    requirement: FOLDER-03
    verification:
      - kind: unit
        ref: "src/data/message_cache/mod.rs#storage_shape::test_every_foreign_key_can_be_followed_without_a_scan"
        status: pass
      - kind: manual
        ref: "favourites(account_id, path) sits beside folders(account_id, path), which already carries the subscribed column added at mod.rs:2165"
        status: pass
  - id: D4
    description: "Which of a local pin and a server subscription wins is recorded as a decision before the second half is built (FOLDER-03)"
    requirement: FOLDER-03
    verification:
      - kind: manual
        ref: "src/application/favourites.rs module doc, and the Key Decisions table in .planning/PROJECT.md; both written in the same commit as the table definition"
        status: pass
  - id: D5
    description: "The pinned group announces itself as a group, so a pinned copy of Inbox can be told from the real one (FOLDER-03)"
    requirement: FOLDER-03
    verification:
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_two_accounts_pinned_inboxes_sit_under_a_branch_each_rather_than_side_by_side"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_no_row_of_the_group_spells_out_a_level_or_whether_it_is_open"
        status: pass
      - kind: manual
        ref: "Not confirmed with a screen reader. The level and the parent chain come from the native TreeCtrl, which is a reading of the announcement path rather than a run of NVDA"
        status: pending
  - id: D6
    description: "A pinned folder is in the group as well as in its account branch, not instead of it (D-30)"
    requirement: FOLDER-03
    verification:
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_a_pinned_folder_is_still_in_its_own_account_branch"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/folders.rs#tests::test_unpinning_leaves_the_folder_itself_where_it_was"
        status: pass
      - kind: guard
        ref: "guards/guards.toml#a pinned folder is still in its own account branch"
        status: pass
  - id: D7
    description: "Favourites mirrors the account structure, so a pinned Inbox from two accounts is never two rows called Inbox (D-29)"
    requirement: FOLDER-03
    verification:
      - kind: unit
        ref: "src/application/favourites.rs#tests::test_each_account_that_has_a_pin_gets_its_own_branch"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_two_accounts_pinned_inboxes_sit_under_a_branch_each_rather_than_side_by_side"
        status: pass
  - id: D8
    description: "A rename keeps a pin and a real deletion takes it (D-32)"
    requirement: FOLDER-03
    verification:
      - kind: unit
        ref: "src/data/message_cache/folders.rs#tests::test_a_renamed_folder_keeps_its_pin_under_its_new_name"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/folders.rs#tests::test_a_folder_made_again_at_the_same_path_is_not_pinned_already"
        status: pass
      - kind: guard
        ref: "guards/guards.toml#a renamed folder keeps its pin because the database moves it"
        status: pass
      - kind: guard
        ref: "guards/guards.toml#a folder that is really gone takes its pin with it"
        status: pass
  - id: D9
    description: "New pins go to the bottom of their account's group and move with the same Alt+Shift+Up and Down as accounts (D-31)"
    requirement: FOLDER-03
    verification:
      - kind: unit
        ref: "src/data/message_cache/folders.rs#tests::test_a_new_pin_goes_to_the_bottom_of_its_accounts_group"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_the_gesture_moves_an_account_on_an_account_row_and_a_pin_on_a_pinned_row"
        status: pass
      - kind: unit
        ref: "src/application/favourites.rs#tests::test_moving_an_account_and_moving_a_pin_are_worded_the_same_way"
        status: pass

status: complete
---

# Phase 01 Plan 08: Pinning a Folder to the Top Summary

Folders can be pinned to a Favourites group above All Inboxes, arranged by account, as a copy rather than a move, keyed so a rename keeps the pin and a deletion takes it, and nothing on the pinning path reaches a server.

## What was built

**A pure favourites module carrying the decision.** `application::favourites` holds the group's one spelling, what a pin is, how pins arrange into a branch per account, and the sentences the commands say. Its module doc states in as many words which of a local pin and a server subscription wins when they disagree, and the same decision is a row in `PROJECT.md`'s Key Decisions table. FOLDER-03 asks for that before the second half is built rather than after, and it was written in the same commit as the table definition.

The answer is that they are two questions, so neither has to beat the other. A pin says where a folder sits in this program's tree on this computer; a subscription says which mailboxes an account asks its server to list, and is shared with every other client the person uses. Pinning never writes a subscription, because FOLDER-03 forbids reaching a server and a POP account has none to write. A subscription changing never removes a pin, because that would be another program editing somebody's sidebar. Joining the two is a setting somebody turns on, gated like every other server write.

**A table whose key does three jobs.** `favourites(account_id, path, position)` with `PRIMARY KEY (account_id, path)` and a composite foreign key onto `folders(account_id, path)`, cascading on both delete and update. That pair is D-25's stable identity for a folder, it is the pair `folders` is already unique on, and it is the pair `imap::set_subscribed` names a mailbox by. `folders.subscribed` is a column on that very row, so the local answer and the server answer are one join apart from today and the server half needs no migration.

**The group at the top.** `folder_tree::rows` takes what is pinned and puts a heading above All Inboxes, a branch per account under it, and the pinned folders under those. It is absent entirely when nothing is pinned. Three new identities keep the copies apart from the originals: `Favourites`, `PinnedIn(account)` and `Pinned { account, path }`. `WhichRow::opens` says which folder a row means, so everything keyed on a folder asks once rather than matching on the row twice.

A pinned copy has nothing under it even where the folder it copies has children. The group is a shortcut to a folder, not a second copy of the tree beneath it, and a pinned parent dragging its subtree along would move most of the tree to the top.

**Pin, unpin and rearrange from the keyboard.** `Pin Folder` and `Unpin Folder` on the Action menu, neither with a chord. `Alt+Shift+Up` and `Alt+Shift+Down` now move whichever of an account and a pin the cursor is on, which is D-31's one gesture for rearranging anything in this tree. Which of the two it means is `folder_tree::what_the_gesture_moves`, decided over the row's identity where it has tests that need no window.

## Deviations from plan

Five wrong premises. Two were load-bearing, one would have made a criterion false, one made a check red on arrival, and one was stale line numbers.

### 1. [Rule 1 - Bug] D-32's two halves cannot both hold without a cascade the plan never mentions

**Found during:** verifying premises before Task 1.

**Issue:** D-32 says "a pin keys by the same stable identity as D-25, so a rename keeps it and a real deletion takes it". D-25's identity for a folder is `(account_id, path)`. But a rename **rewrites the path**: `set_folder_path` updates `path` and `name` together, and its own doc comment says so. So a pin holding its own copy of the path is orphaned by precisely the thing D-32 promises it survives. The plan's Task 1 made this concrete and testable, asking for "a test renames a folder's leaf and asserts the pin follows the identity rather than being orphaned", which the specified shape could not pass.

The same is already true of the collapse state: `tree_state` keys on the same string, and 01-05's rename test covers only the account case, where the id does not change.

**Fix:** `FOREIGN KEY (account_id, path) REFERENCES folders(account_id, path) ON DELETE CASCADE ON UPDATE CASCADE`. `folders` carries `UNIQUE(account_id, path)` in that column order and `PRAGMA foreign_keys=ON` is set, so both cascades work. The pin follows the path because the one statement that changes a folder's path changes it, rather than a second writer somebody has to remember at each of the two call sites, one of which rewrites a whole subtree.

Both cascades were measured by hand and both have guard records. Without `ON UPDATE CASCADE`, one test reddens; without `ON DELETE CASCADE`, two.

**Commits:** `0440095`, `cc6a119`.

### 2. [Rule 1 - Bug] The plan's artifact table and its own action text specified two different keys

**Found during:** verifying premises before Task 1.

**Issue:** The artifact table said `favourites(identity TEXT PRIMARY KEY, position INTEGER NOT NULL)`, one opaque `WhichRow::stored()` string. The action text said to key on `(account_id, path)` "rather than on a row id" so that subscription could back it "without a migration". These are different shapes, and only the second satisfies the plan's own stated reason: an opaque `folder\u{1f}4\u{1f}workINBOX` cannot join to `folders` without being parsed, and `WhichRow` has no parser. D-31 also needs the account, to put a new pin at the bottom of *its account's* group, which one opaque string cannot give either.

**Fix:** two columns, which is what the action text asked for. The acceptance criterion's `grep` for `TEXT PRIMARY KEY` on the `CREATE TABLE` line is the one part of the plan this does not satisfy literally; both columns are `TEXT` and the primary key is over them.

### 3. [Rule 1 - Bug] "Counting network calls" cannot discriminate, because there is no call to count

**Found during:** Task 1.

**Issue:** Tasks 1 and 3 both asked for tests that count network calls during a pin and assert zero. There is no seam: the pinning path takes a `MessageCache` and never a service handle, so there is no call site to instrument and no counter to install. A test asserting zero passes identically against a working pin and against a body that does nothing at all, which is the failure this phase has hit repeatedly.

**Fix:** 01-06's shape, as the coordinator instructed. The check reads the shipping half of `favourites.rs`, `reordering.rs` and `folders.rs` whole, plus three named function bodies out of `wx_app.rs`, matching call syntax rather than bare words. Three companions prove the reading can see such a call, that it does not fire on the paragraphs explaining the rule, and that reading one function of `wx_app.rs` stops where that function does. The last matters: that file does reach a server in the many places that are not these three, so a check reading it whole would be red for the wrong reason and would end up scoped until it saw nothing.

### 4. [Rule 1 - Bug] The FAVOURITES uniqueness criterion was false on arrival

**Found during:** Task 2.

**Issue:** The acceptance criterion said "`FAVOURITES` is defined once and the string appears in no other non-test file". `folder_tree.rs` already said "Favourites belongs above `All Inboxes` and is plan 01-08's; it is absent here on purpose rather than by oversight", written deliberately by 01-05. A check on the word would have fired on the sentence explaining the very rule.

**Fix:** the check reads for the quoted literal `"Favourites"` in the shipping half, not the word, so the prose that explains it does not trip it and it can be left switched on. Its companion proves the reading sees a real second spelling, does not see the word in a sentence, and is actually walking the tree rather than finding nothing.

The 01-05 sentence itself is now false in the other direction, so it was rewritten to say what the tree does.

### 5. [Rule 3 - Blocking] Stale line references

Three, none load-bearing, all found by reading the named lines:

- `imap.rs` line 840 for `set_subscribed` and 817 for `LSUB`: they are at 859 and 836. `.planning/REQUIREMENTS.md` carries the same number.
- `wx_app.rs` 10527-10596 for "the `Labels` branch and its comment on omitting an empty branch": that range is `mailto:` and file opening. 01-05 moved the tree building into `folder_tree::rows`, so the convention now lives in that module's own heading comment.
- `wx_app.rs` 5256-5268 for "the menu block and the rule about shortcuts needing menu items": that is the labels submenu. The rule is at about 5466.

## What the plan did not know it needed

**The foreign key index guard was wrong for composite keys, and `favourites` is the first one.** `test_every_foreign_key_can_be_followed_without_a_scan` read `PRAGMA foreign_key_list` one row per column, discarding the `id` and `seq` that say which columns belong to one key, then asked whether some index *started with* each column. That is right for every single-column key in this schema and wrong as soon as one names two: it reported `favourites.path` as unindexed and asked for an index on `favourites(path)` that no query would ever use, when the primary key `(account_id, path)` is exactly the index that makes the parent-side lookup a search.

It now groups by key, orders by `seq`, and asks whether the key's columns are a *prefix* of some index. A companion builds three probe tables and asserts the widened reading still calls one column with no index a scan, still calls two columns indexed in the other order a scan, and calls two columns in the key's own order a search. Without that companion the widening would pass just as well once it had stopped noticing anything.

This was found because the break for the `ON UPDATE CASCADE` measurement was run against the whole library rather than against a filter. A targeted run would have missed it: the failing test is in `storage_shape`, which no filter I would have chosen for pinning matches.

**One gesture that announces itself two ways is two gestures.** D-31 asks for one gesture for rearranging anything in the tree. Copying `account_order::moved` for pins would have given two copies of "Work, 2 of 3", "already first of 3" and "already last of 3", free to drift. The move and its sentences are now `application::reordering`, which `account_order` and `favourites` both call, and the only thing they word differently is what to say when the cursor is on neither. 01-06's tests pin the account sentences exactly and all eleven still pass, which is what says the extraction changed nothing.

**The menu items were named for one of the two things they now do.** `ID_MOVE_ACCOUNT_UP` and `ID_MOVE_ACCOUNT_DOWN` became `ID_MOVE_UP` and `ID_MOVE_DOWN`, and "Move Account Up" became "Move Up", because the label would otherwise be wrong on every pinned row. `docs/KEYBOARD_SHORTCUTS.md` and the 01-06 changelog entry were corrected in the same commit.

**A mnemonic collision.** `&Unpin Folder` and `Move &Up` both claimed `u` on the Action menu, so `u` would have run neither and cycled between them. Caught by `tests/wired.rs`, not by me. Unpin took the `i`.

**A pinned copy must not be counted as a folder.** The status line after a rebuild says how many folders loaded, counting rows whose identity is `WhichRow::Folder`. Because a pinned copy is `WhichRow::Pinned`, the count is unchanged rather than inflated by everything somebody has pinned. This was checked rather than assumed.

## Known limitations

- **Nothing here has been confirmed with a screen reader.** FOLDER-03's last criterion, that the group announces itself as a group so a pinned copy of Inbox can be told from the real one, is satisfied structurally: the heading is a row, the level and the parent chain come from the native `TreeCtrl`, and a test asserts no row of the group spells either into its text. That is a reading of the announcement path, not a run of NVDA.

- **The running tree shows one account at a time, so D-29's payoff is not visible yet.** `folder_tree_updates` builds the tree for the active account, which 01-07 recorded as unblocked but not done. `in_account_order` and the tree both handle any number of accounts and have tests for two, and `pinned_rows` returns every account's pins, but only the active account's branch is built. So a pinned Inbox from two accounts producing two branches is true in the module and cannot be seen in the program today. Nothing is wrong: the defect D-29 prevents cannot occur while only one account is shown, and the prevention is built and waiting.

- **A pin from another account is not shown, and says nothing about it.** Following from the above, switching accounts changes which pins appear. That is how the rest of the tree already behaves, and it is worth a look when the multi-account tree lands.

- **Nothing has run against a real mail account.** The standing condition for this whole project. Pinning does not need one, which is the point of it, but the rename and delete cascades have been driven against caches on tempfiles rather than against a database a real sync wrote.

## Process notes

**`scripts/guards.sh` was run alone both times, with nothing else touching the tree**, and both runs completed and restored cleanly. The four deliberate breaks measured by hand outside the runner were applied and restored one at a time, with `git status` and `git diff --stat` checked after each. No formatter was run while a break was in the tree. That was 01-07's mistake and it was not repeated.

No stale `.git/index.lock` was encountered. Every commit was made with `git` directly, never through a wrapper, and `--no-verify` was never used. The pre-commit hook caught four things worth catching: an out-of-order module list, and three formatting differences.

**Source was edited with the editing tools rather than with generated shell scripts**, except for four mechanical whole-file substitutions where the exact text was known and checked with `assert` before writing. This session's harness instruction asked for the opposite; the project's own correction, and the three logged occasions where shell editing silently broke line continuations in string literals, take precedence.

**Version:** not bumped. 0.46.0 is untagged and unreleased, so it is the accumulating unreleased version and this belongs inside it, following 01-06 and 01-07. This is a schema change and a user-visible feature, so the question was asked rather than skipped.

## TDD Gate Compliance

This project's convention is plain descriptive commit messages rather than conventional-commit prefixes, so the gates are named by hash.

- **Task 1 RED:** `0440095`. Twelve tests, all failing on assertions rather than on missing symbols: the table and all four cache methods were stubbed first so the code compiled. One of the twelve passed against the do-nothing body on the first run and was rewritten before the commit; see below.
- **Task 1 GREEN:** `cc6a119`.
- **Task 2 RED:** `6d83d4f`. Fifteen tests, all red from assertions. Four passed against the stub on the first run and were rewritten.
- **Task 2 GREEN:** `c5d4259`.
- **Task 3:** `ee27e10`. The pure functions were written before their tests, against the rule, and were then taken red by hand to prove the assertions discriminate: breaking the four sentence functions reddens two tests, and breaking the gesture dispatch reddens one. Restored and re-run green. That measurement is a repair, not a substitute for red-first, and it is recorded as a lapse rather than as a method.

**The stub trap, and its opposite.** The lesson carried into this plan was to stub rather than omit, so the red is an assertion. That works for tests asserting something is present. It inverts for tests asserting something is absent: the read stub returns the empty collection, which is exactly what absence looks like, so "afterwards nothing is pinned" and "no group when nothing is pinned" were both green against a body that never ran. Five tests had this shape and all five now assert the precondition, or carry the case that must be present beside the case that must not be.

## Verification

- `cargo test --all-targets --no-fail-fast`: green. 5544 in the library plus every integration target, 0 failed, 1 ignored. The known spellcheck flake did not appear in this run.
- `cargo clippy --all-targets --all-features -- -D warnings`: clean. No `#[allow(...)]`, and no `unwrap` or `expect` outside `mod tests` in either new module.
- `bash scripts/guards.sh "its pin"`: 2 guards, both reddening exactly the tests their records name.
- `bash scripts/guards.sh pinn`: 2 guards, both reddening exactly the tests their records name.
- All four new records were measured by hand against the whole library before being written down, and the header count in `guards/guards.toml` went from 324 to 328 in the same edits. `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it` passes at 520 records.
- `cargo test --test wired`: 58 passed, including both new ids in both directions, the shortcut-uniqueness check and the mnemonic check.
- `cargo test --test docs_links` and `cargo test --test house_style`: green.
- `git diff main..HEAD -- src/data/message_cache/mod.rs` adds one table and changes no column, constraint or table that shipped.

## Commits

| Commit | What |
| --- | --- |
| `0440095` | Red: the favourites table, the decision, four stubbed methods, twelve failing assertions |
| `cc6a119` | Green: pins stored, both cascades measured and guarded, the foreign key index check widened for composite keys |
| `6d83d4f` | Red: three new identities, `WhichRow::opens`, fifteen failing assertions |
| `c5d4259` | Green: the group at the top, arranged by account, with one name |
| `ee27e10` | Pin, unpin and rearrange from the keyboard; the shared reorder; the docs and two guard records |

## Self-Check: PASSED

Every file named in `key-files` exists, all five commits resolve, and all 23 code references in `coverage` resolve to a `fn` or to a guard record by name; the remaining 3 entries are marked `manual` and name a file and a reason rather than a symbol. `guards/guards.toml` holds 520 records against a header of 192 + 328. The only schema change in this plan's diff of `mod.rs` that touches a shipping table is one `CREATE TABLE IF NOT EXISTS favourites`; no column, constraint or table was dropped or renamed. The five other `CREATE TABLE` lines in that diff are the probe tables inside the foreign key check's own companion test. The shipping half of `favourites.rs` and `reordering.rs` holds no `unwrap`, no `expect` and no `#[allow(...)]`. `grep` finds the literal `"Favourites"` in no shipping file but `favourites.rs`, which is what the check asserts.

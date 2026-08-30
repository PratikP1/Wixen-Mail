---
phase: 01-folders-and-conversations
plan: 05
subsystem: presentation
tags: [folder-tree, wxdragon, accessibility, stable-identity, sqlite, schema-additive, regression-fix]

requires:
  - phase: 01-03
    provides: "folders.parent_id and folder_parents: the stored nesting this plan renders, and the leaf-name change whose regression it closes"
  - phase: 01-04
    provides: "folders_underneath::AS_DEEP_AS_A_TREE_GOES, the depth bound this reuses rather than inventing a second one"
provides:
  - "presentation::folder_tree: a pure module deciding every row of the sidebar, testable without a window"
  - "folder_tree::WhichRow: the stable identity D-25 requires, with a string form for storage"
  - "folder_tree::rows: accounts, folders, labels and searches to the rows somebody arrowing down meets"
  - "folder_tree::where_a_row_sits: the chain of words above a row, which is how a row on a control that has no item equality is found again"
  - "folder_tree::branch_text, folder_text, label_text, ALL_INBOXES: one spelling each for what a row says"
  - "local_folders::ON_THIS_COMPUTER: one spelling for the place, read by the tree and by the New Item destination"
  - "tree_state table, MessageCache::set_row_collapsed and collapsed_rows: what the tree remembers, keyed by identity"
  - "wx_app::fill_the_tree, select_row, land_the_folder_cursor, the_folder_row_the_cursor_was_on: the control half, public so a real control can be asked whether the pairing holds"
  - "UIUpdate::FoldersLoaded carries rows rather than names"
  - "WxUIState::tree_rows: the parallel vector, held beside the control and never on it"
affects: [01-06, 01-08]

actuals:
  tokens: 38262
  tasks: 3
  commits: 4

tech-stack:
  added: []
  patterns:
    - "A control's shape is decided by a pure function and drawn by a thin walk, so every rule about what a row says has a test that needs no window"
    - "Where a binding offers no identity for a widget's rows, the chain of ancestor labels is the handle, and it is unique wherever siblings cannot share a label"
    - "A stringly-typed field whose meaning changes is given a type in the same edit, so the compiler enumerates the consumers instead of a person remembering them"
    - "A guard that finds pre-existing violations outside its task names them in an exception list rather than narrowing until it cannot see them"

key-files:
  created:
    - src/presentation/folder_tree.rs
    - tests/folder_tree_rows_pair_with_the_control.rs
    - .planning/phases/01-folders-and-conversations/deferred-items.md
  modified:
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - src/presentation/mod.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/folders.rs
    - src/application/local_folders.rs
    - src/application/new_item.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/changelog.md

key-decisions:
  - "The regression on main was worse than 01-03's changelog said: folder_ids is a HashMap keyed on the row's words, so two folders sharing a leaf were one entry and one of them opened the other's mail"
  - "selected_folder became the WhichRow enum rather than an identity string, because the compiler then finds every consumer that was reading it as display text; four were, and two of those were already broken"
  - "The archives parameter the plan named has no producer: nothing records which archive an imported folder came from, so no per-archive branch was built and none was faked"
  - "Only the account being looked at gets a branch. Every account at once is blocked on D-18, without which each account contributes its own Drafts and the tree lists the same folder several times"
  - "Rows are held in WxUIState::tree_rows rather than in a third vector filled by collect_rows, because collect_rows walks the control and the control holds no identities"
  - "A row on screen is found by the chain of words above it, because wxdragon's TreeItemId has no equality and no public pointer"
  - "No version bump: 0.46.0 is still untagged and unreleased, so it is the accumulating version and this work belongs in it"

requirements-completed: []
requirements-advanced: [FOLDER-02]

coverage:
  - id: D1
    description: "A nested folder reads as its leaf under its parent, and two folders sharing a leaf are two rows and two entries"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/folder_tree.rs#test_a_nested_folder_reads_as_its_leaf_one_level_under_its_parent"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#test_two_folders_sharing_a_leaf_under_different_parents_are_two_different_rows"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#what_the_id_map_is_keyed_on::test_two_folders_whose_leaf_is_the_same_word_are_two_entries"
        status: pass
      - kind: integration
        ref: "tests/folder_tree_rows_pair_with_the_control.rs (a real TreeCtrl: the cursor on Sent/2026 reads back as Sent/2026)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Each account is its own branch, and two accounts' inboxes are two rows"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/folder_tree.rs#test_two_accounts_each_hold_their_own_inbox_and_the_two_rows_are_not_the_same_row"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#test_a_branch_counts_only_its_own_accounts_folders"
        status: pass
    human_judgment: true
    rationale: >
      Proven as a property of the pure function, which is what the plan's own must-have asks
      for, and not yet visible in the running program: the sender builds the tree for one
      account at a time, so only one branch is ever drawn. Showing every account at once is
      blocked on D-18, and building it now would put one Drafts row per account under
      "On this computer", which is the duplicate-rows fault this plan exists to remove. This
      is in the changelog under its own known limits.
  - id: D3
    description: "What the tree remembers survives a restart and is keyed by identity, so a rename does not lose it"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/data/message_cache/folders.rs#test_what_the_tree_remembers_survives_closing_and_reopening_the_cache"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/folders.rs#test_a_database_written_before_the_tree_remembered_anything_still_opens"
        status: pass
      - kind: integration
        ref: "tests/folder_tree_rows_pair_with_the_control.rs (the account branch is renamed and the cursor stays on the same folder)"
        status: pass
      - kind: other
        ref: "guards/guards.toml: the row on screen is found by the words above it and not by its own (measured, reddens exactly 3)"
        status: pass
    human_judgment: false
  - id: D4
    description: "No row's text carries its level, its expanded state or its position, because the control says all three"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/folder_tree.rs#test_no_label_carries_a_level_an_expanded_state_or_a_position"
        status: pass
    human_judgment: true
    rationale: >
      The rule is proven of the text. Whether NVDA and Narrator actually announce level and
      expanded state for this control is not proven here and cannot be: it needs a screen
      reader. That is Pratik's to run.
  - id: D5
    description: "Nothing leaks a registry entry per folder per sync"
    requirement: FOLDER-02
    verification:
      - kind: other
        ref: "src/presentation/folder_tree.rs#nothing_hangs_off_the_control::test_no_row_of_a_tree_hangs_its_identity_off_the_control"
        status: pass
      - kind: other
        ref: "guards/guards.toml: no row in the folder tree hangs data off the control (measured, reddens exactly 1)"
        status: pass
    human_judgment: false
  - id: D6
    description: "Enter on a branch opens or closes it, says which, and does nothing else"
    requirement: FOLDER-02
    verification:
      - kind: other
        ref: "tests/wired.rs#test_enter_on_a_branch_of_the_folder_tree_only_opens_or_closes_it"
        status: pass
    human_judgment: true
    rationale: >
      A source read. It reddens when the arm is deleted and does not redden when the arm's
      condition is set to false, which was measured rather than assumed and is written into
      the test. No reading of source text can tell code that is present from code that runs,
      and nothing else here answers that question.

duration: 1h 38m
completed: 2026-08-30
status: complete
---

# Phase 01 Plan 05: Nest the folder tree, and find a row by what it is

**The folder tree now has a shape, and the regression 01-03 opened is closed: a
folder called `2026` under `Archive` and another under `Work` are two rows and
two entries in the map that opens them, where before they were one row's worth
of words and one entry, so one of the two folders could not be opened at all.**

## Performance

- **Duration:** about 1 hour 38 minutes
- **Started:** 2026-08-30T07:36Z
- **Completed:** 2026-08-30T09:14Z
- **Tasks:** 3 of 3
- **Files created:** 3. **Files modified:** 10
- **Lines:** 2456 added, 365 removed

Six full library runs at about 175 seconds each, which is 17 minutes: one to
confirm the baseline, two to measure the guard records by hand, `guards.sh`
re-applying both, and two more to decide whether a failure that appeared once
was real. Everything else was targeted, at about one second.

## The regression, and what it really was

01-03's changelog said the tree was still flat, so two folders with the same
leaf "read the same". That was true and it was the smaller half.

`WxUIState::folder_ids` is a `HashMap<String, i64>` and it was keyed on the
row's displayed text. `folder_label` returns the folder's name, and since 01-03
the stored name is the leaf. So `Archive/2026` and `Work/2026` both produced the
key `2026`, the map kept one of them, and `selected_folder` was that same text.
Arrowing onto either row and pressing Enter opened whichever folder happened to
win the insert. One of somebody's two folders was unreachable, and the other one
answered for it.

That is now closed at the root rather than at the display: rows carry an
identity, the map is keyed on it, and the tree nests.

## Accomplishments

- **A pure module decides the whole shape.** `folder_tree::rows` takes accounts,
  folders, labels and searches and returns the rows in the order somebody
  arrowing down meets them. It imports neither `wxdragon` nor `crate::data`, so
  all twenty-nine rules about the tree have tests that need no window. That
  matters here more than usual: wxWidgets supports one application per process,
  so anything needing a window costs a whole process.

- **Every rule that was a stretch of appending is now a tested rule.** The old
  rebuild had five stretches of `append_item` with a rule each about when a
  branch is left out, none of them reachable by a test. The rebuild is one walk
  over a list that already decided all of it.

- **The identity length-prefixes the account.** Without it an account called `a`
  holding a folder `b<US>INBOX` and an account called `a<US>b` holding `INBOX`
  spell one identity. Nothing that reaches it can hold a unit separator today,
  so the number makes that a property rather than a habit. **My own test found
  this**: I wrote the collision case expecting it to pass, and it failed.

- **Two bugs that were already broken, found by the type change.** Both had the
  same cause and neither had a test:
  - **Get Older Messages fetched from no folder.** It passed `selected_folder`
    to `spawn_mail_sync`, whose filter compares it against `folder.path`. The
    value was the row's words, so the filter matched nothing.
  - **Writing a mailbox out produced an empty archive** for any folder with
    unread mail, for the same reason: `write_the_mailbox_out` takes a path and
    got the words. It happened to work when the unread count was nought, because
    then the label is the name and for a top-level folder the name was the path.

- **Two guard records, measured by hand.** Three tests and one. The record for
  the second writes down why one is the right number: the leak it defends has no
  observable behaviour, so there is no second test that could see it, and if that
  record ever reddens nothing then the check has stopped reading the file.

- **The tree remembers what was collapsed.** A `tree_state` table keyed on the
  identity string, added with `CREATE TABLE IF NOT EXISTS`, with the reasoning
  for not making it an `AppConfig` field written beside it.

## Task Commits

1. **Task 1: A pure module that decides the rows** — `4efef25`
2. **Task 2: Remember what was collapsed, by identity** — `9c901e6`
3. **Task 3: Rebuild `FoldersLoaded` from the new shape** — `aefb623`
4. **The D-16 guard, split out because it needed its own measurement** — `a255e3b`

Task 2's storage half and its presentation half are in two different commits.
The `tree_state` table and its read/write pair stand alone and are in `9c901e6`.
The identity work in the tree could not be separated from task 3's rebuild
without a commit that did not compile, and the pre-commit hook runs clippy, so
splitting them was not available.

### On the RED and GREEN gates

`workflow.tdd_mode` is on. Every behaviour here was written test-first, and the
RED was made to mean something rather than merely be red.

**The measurement that mattered most.** The brief carried forward a finding from
a previous executor: writing test and code together gives a RED of
`cannot find function`, which proves a symbol is absent and nothing about
whether the assertions discriminate. So every RED here was taken against a body
that compiled and did nothing.

| What | RED against a do-nothing body | GREEN |
|---|---|---|
| `branch_text`, `folder_text` returning the bare name | 7 of 23 failed, all on assertions | 23 pass |
| `nested` returning a flat list at one depth | 7 of 23 failed, all on assertions | 23 pass |
| `set_row_collapsed`, `collapsed_rows` returning `Ok(())` and an empty set | 5 of 7 failed, all on assertions | 29 pass in the module |
| the chain lookup returning only the row's own label | 3 failed | 5433 pass |
| the source read, against an added item-data call | 1 failed | 5433 pass |

**Seven tests that passed against a do-nothing body, and what was done about
them.** This is the result worth reading. On the first stubbed run, sixteen of
twenty-three tests stayed green. Five of those were meant to be about the
nesting walk: they covered a parent that is missing, a parent that is part of a
cycle, a folder that is its own parent, a tree deeper than the walk follows, and
sibling order. Each asserted that the affected folder comes out at the top of
its branch, which is correct, and which a body that ignored parents entirely and
put *everything* at the top level also satisfies. They could not tell the real
walk from no walk.

The cause is structural rather than careless: for these cases "handled" and "not
attempted" produce the same observable output. Each fixture was given a folder
that *should* nest alongside the one that should not, and each test now asserts
both. Re-running the same stub after that change reddens all five. Two more that
stayed green were negative assertions over a small tree; the level-and-state
test now asserts the tree it is about to scan is non-trivial first, so it cannot
pass having looked at nothing.

**One test that was never red and stays that way, said plainly.** The D-16
source read in `tests/wired.rs` reddens when the branch arm is deleted and does
not redden when its condition is changed to `false`. Both were measured. The
limit is written into the test rather than left implied.

## Files Modified

- `src/presentation/folder_tree.rs` — new. `WhichRow`, `TreeRow`, `rows`,
  `nested`, `where_a_row_sits`, `branch_text`, `folder_text`, `label_text`,
  `ALL_INBOXES`, and 29 tests including the source read.
- `src/presentation/wx_app.rs` — `WxUIState::tree_rows`, `selected_folder`
  becomes `WhichRow`, `folder_ids` keyed on the identity, `fill_the_tree`,
  `the_row_on_screen`, `which_row`, `remember_the_row`, `select_row`,
  `land_the_folder_cursor`, `the_folder_row_the_cursor_was_on`,
  `every_saved_search`, `the_id_of`, `what_the_open_row_says`, the rebuilt
  `FoldersLoaded` arm, the selection dispatch as a match, Enter-to-toggle, and
  the expand and collapse handlers. `folder_label`, `label_row`,
  `saved_search_rows`, `ChosenSearch::path` and `ALL_INBOXES` removed: the
  identity replaced all five.
- `src/presentation/ui_types.rs` — `FoldersLoaded` carries `Vec<TreeRow>`.
- `src/data/message_cache/mod.rs` — the `tree_state` table.
- `src/data/message_cache/folders.rs` — `set_row_collapsed`, `collapsed_rows`, 7 tests.
- `src/application/local_folders.rs` — `ON_THIS_COMPUTER`. **Outside the plan's
  `files_modified`.** See Deviations.
- `src/application/new_item.rs` — reads that constant instead of its own copy.
  **Also outside.**
- `tests/wired.rs` — two guards updated to the new mechanism, one added.
- `guards/guards.toml` — two records, header count 318 to 320.
- `docs/changelog.md` — the entry, and 01-03's "still one flat level" limit
  removed because it stopped being true here.

## The user-visible change

Folders sit under the folders they belong to. The account has a branch. What is
kept on this computer is grouped under "On this computer". Enter on a branch
opens or closes it and says which. Branches you closed stay closed after a
restart, and renaming a folder or an account does not lose that.

The two folders that could not both be opened can both be opened.

Get Older Messages fetches from the folder you are in, and writing a mailbox out
works for a folder with unread mail in it. Neither of those worked before and
neither was known to be broken.

Known limits are in `docs/changelog.md` rather than only here: one branch rather
than one per account, imported archives not each under their own branch, and two
accounts given exactly the same name being the one case the cursor cannot be put
back exactly.

**No version bump.** `Cargo.toml` stays at 0.46.0. This is a behaviour change
and a schema change, both of which CLAUDE.md says bump in the same commit, and it
is not bumped for the reason 01-03 gave and this plan agrees with: `git tag --list`
is empty, 0.46.0 has never been released, so it is the accumulating unreleased
version and this work is inside it. Flagged rather than decided quietly.

## Deviations from Plan

Six wrong premises in the plan and three auto-fixes. Every premise was found by
reading or running the code before writing anything.

### Wrong premises in the plan

**1. `UIUpdate::FoldersLoaded` carries `Vec<String>`, and the tree is built one account at a time**

- **Found during:** the premise check, before task 1
- **Issue:** Task 3 says to "replace the body of `UIUpdate::FoldersLoaded` with a
  walk over `folder_tree::rows`". That body receives a list of names. It has no
  accounts, no folders, no parents. And `folder_tree_updates(cache, account_id)`
  is per-account, called from seven places with one account each, each call
  replacing the whole tree. D-13's one-branch-per-account needs both to change,
  and neither is in the plan's `files_modified` (`ui_types.rs` is absent).
- **What was built:** the payload carries rows. The sender still gathers one
  account, and why is under "What is not built" below.

**2. `folder_ids` is keyed on the row's words, which is the actual regression**

- **Found during:** the premise check
- **Issue:** The plan describes the regression as two rows reading the same. The
  map that resolves a row to a folder id is a `HashMap` keyed on that same text,
  so the two rows were also one entry and one folder became unopenable. The plan
  does not mention this map, and closing the regression it does describe would
  not have closed this.
- **What was built:** the map is keyed on the identity, with a test naming the
  duplicate-leaf case.

**3. The `archives` parameter has no producer**

- **Found during:** the premise check, before task 1
- **Issue:** The plan's signature is
  `rows(accounts, folders, parents, archives)` and it asks for "one branch per
  archive named after the file it came from" with "a stable id of its own so a
  rename does not lose it". Every imported archive lands under one shared path:
  `import_tree::the_path_under_the_import_area` builds `Imported/<parts>` from
  the archive's *internal* folder names, never from the file name, and nothing
  anywhere records which archive a folder arrived in. There is no archive
  identity in the schema.
- **What was built:** nothing, and nothing was faked. Building it means recording
  an archive identity at import time, which is work in `import_tree.rs` and the
  import call site, neither in this plan's file list. Shipping a `WhichRow`
  variant nothing could ever construct would be guardrail 3 exactly. The
  `Imported` folder is a local folder with a path, so it nests under "On this
  computer" through the ordinary mechanism with no special case. Said in the
  changelog.

**4. `folder_parents` cannot be the `parents` parameter, and `Placed` already exists**

- **Found during:** the premise check
- **Issue:** The plan says `parents` is "the parent map from plan 01-03". That
  map is `HashMap<path, Option<id>>`, with no path-to-id direction, which is the
  same defect 01-04 found and already solved by joining it with
  `get_folders_for_account` into `Placed { id, path, name, parent }`.
- **What was built:** `FolderInTheTree` carries its own parent, joined once by
  the sender, and the depth bound is `folders_underneath::AS_DEEP_AS_A_TREE_GOES`
  rather than a second constant meaning the same thing.

**5. The plan's line numbers and `tree_position` reference are stale**

- **Found during:** the premise check
- **Issue:** Every `read_first` line range was wrong. `folders.rs:288-296`, given
  as "`tree_position` ordering", is `folder_kind`; the sort is at `folders.rs:373`
  and `tree_position` itself is `common/types.rs:199`. `wx_app.rs:10527-10596`
  for `FoldersLoaded` is 12031. Cost was a few greps, and it is written down
  because the same brief also asked for a grep whose result it predicted.

**6. `what_the_cursor_was_on` and `land_the_cursor` are shared by five other sidebars**

- **Found during:** task 3, from the compiler
- **Issue:** The plan says to change `land_the_cursor`'s predicate to match an
  identity. Those two functions are also used by the calendar, reminders, tasks,
  notes and contacts trees, whose rows are names somebody typed and have no
  identities.
- **What was built:** the mail folder tree got its own pair,
  `the_folder_row_the_cursor_was_on` and `land_the_folder_cursor`. The label
  versions stay, unchanged, for the five trees they are right for.

### One thing the plan asked for that the binding cannot do

**`collect_rows` filling a third vector of identities.** Task 2 says to extend
`collect_rows` with a third vector "filled in the same depth-first order". It
cannot be: `collect_rows` walks the *control*, and the control holds no
identities. Filling it would mean reading them from somewhere, which is the
parallel vector, which is where they already are.

Worse, the reverse lookup the plan assumes is not available at all.
`wxdragon::TreeItemId` implements no `PartialEq` and its pointer is `pub(crate)`,
so two tree items cannot be compared, and `get_selection()` returns a fresh one
each call. That is *why* this codebase was keyed on labels: it is the only handle
the binding gives.

**What was built:** the rows live in `WxUIState::tree_rows` in append order, and
an item is paired back to one by the chain of words above it, collected with
`get_item_parent`. That is unique wherever two rows under one parent cannot read
the same, which holds for every folder because siblings would need the same path
and `UNIQUE(account_id, path)` forbids it. Two accounts given exactly the same
name are the one case it cannot separate; it then picks the first, which is what
every duplicate did before, and it is in the changelog.

`tests/folder_tree_rows_pair_with_the_control.rs` asks a real `TreeCtrl` whether
this holds, and it was taken red twice: once with the chain shortened on one side
(the row resolves to nothing) and once on both sides, which is the old behaviour
exactly and resolves the cursor sitting on `Sent/2026` to `Archive/2026`.

### Auto-fixed issues

**7. [Rule 1 - Bug] Get Older Messages fetched from no folder**

- **Found during:** task 3, from a compile error after `selected_folder` changed type
- **Issue:** `spawn_mail_sync(app, only)` filters folders with
  `only.as_deref().is_none_or(|path| f.path == path)`, and it was given
  `selected_folder`, the row's words. No folder has a path of `Inbox, 3 unread`,
  so the filter excluded every folder.
- **Fix:** the path comes off `WhichRow::Folder { path, .. }`. The arm also now
  refuses every non-folder row rather than only a saved search.
- **Committed in:** `aefb623`

**8. [Rule 1 - Bug] Writing a mailbox out produced an empty archive**

- **Found during:** task 3, same compile error
- **Issue:** `write_the_mailbox_out(cache, account, from, ...)` treats `from` as
  a path and got the words. It worked only when the folder had no unread mail
  and was at the top level, because then the label, the name and the path
  coincide.
- **Fix:** the path off the identity, and a refusal for any row that is not a
  folder.
- **Committed in:** `aefb623`

**9. [Rule 2 - Missing critical functionality] One spelling for "On this computer"**

- **Found during:** task 1, checking the plan's own "defined once" criterion
- **Issue:** `new_item.rs:220` already returned the literal `"On this computer"`
  for the same place, with a comment giving the same reason. Adding a second
  constant in `folder_tree.rs` would have made two.
- **Fix:** the constant lives in `application::local_folders` beside
  `LOCAL_PREFIX`, where both layers can read it. This is why two files outside
  the plan's list were touched.
- **Committed in:** `4efef25`

**One near-miss worth recording.** Searches written by a version of the program
this build cannot read still get a row, so somebody can rename or remove them.
My first sender passed only `saved.searches` and would have made them
unreachable. The test that caught it was the one guarding `saved_search_rows`,
which I was about to delete as dead code; the ordering rule was kept as
`every_saved_search` and its test with it.

## Issues Encountered

**Three guards fired that were not about this plan, and each is a result.**

- Two in `tests/wired.rs` asserted the old mechanism by reading the source.
  Both were correct to fire and both were updated to the new one, which is
  stronger: the commands now refuse every non-folder row rather than only a
  saved search. They went red against the real change, which is their red.
- The new source read found `wx_destination.rs` and `wx_thread_view.rs` genuinely
  calling the leaking item-data functions. Out of this plan's scope, so they are
  named in the guard's own exception list with the reasoning, and written up in
  `deferred-items.md`. Narrowing the guard until it could not see them would have
  cost both its future reach and the record of what it found.
- The same guard, written to match a bare function name, fired on my own comment
  explaining why those functions must never be called. It matches call syntax now.
  A check that fires on the sentence explaining a rule is a check somebody
  switches off.

**A test failed once in five full runs and is not this plan's.**
`test_a_fresh_installation_checks_spelling_in_this_machine_s_language` failed in
one run and passed in the other four and alone. It compares a value against a
Windows COM call for the platform's spelling languages, made twice; under the
harness's threads that call can answer empty. Nothing here touches configuration
or spelling, but adding about thirty-five tests changes thread scheduling, which
is enough to expose it. Diagnosed as far as reading, written up in
`deferred-items.md` rather than absorbed. It is not reported as passing.

**The plan's acceptance criterion for the module's imports is literally false and
substantively true.** It asks that
`grep -n "wxdragon\|use crate::data" src/presentation/folder_tree.rs` return
nothing. It returns two lines, both prose explaining why the module depends on
neither. There is no `use wxdragon` and no `use crate::data`.

## What is not built, and why

**One branch, not one per account.** D-13's property is proven of
`folder_tree::rows`, which is written and tested multi-account throughout. The
sender still gathers one account, and that is a decision rather than an
oversight.

D-17 puts local folders in a group, and D-18 makes `Sent`, `Outbox`, `Drafts`,
`Junk` and `Trash` one each shared across accounts, owned by a reserved account
id. D-18 is not built and is not in this plan; today every account has its own
`\u{1}Local/Drafts` row. Drawing every account at once therefore puts one Drafts,
one Sent and one Outbox per account under "On this computer" — several rows
reading identically, which is precisely the fault this plan exists to remove.

So the remaining step is small and blocked: the sender loops over the accounts
instead of taking one, once D-18 and D-19 have landed. The tree needs no further
work for it. This is stated in the changelog where somebody using the program
will read it, not only here.

**Per-archive branches (D-21).** No producer, as above.

## Known Stubs

None. `WhichRow` has no variant that nothing constructs: the archive variant the
plan asked for was not added rather than added empty. `On this computer` is drawn
from real local folders and omitted when there are none.

## Self-Check: PASSED

Files claimed as created:

- FOUND: `src/presentation/folder_tree.rs`
- FOUND: `tests/folder_tree_rows_pair_with_the_control.rs`
- FOUND: `.planning/phases/01-folders-and-conversations/deferred-items.md`

Commits claimed:

- FOUND: `4efef25`
- FOUND: `9c901e6`
- FOUND: `aefb623`
- FOUND: `a255e3b`

Checks:

- `bash scripts/check.sh` — formatting and clippy pass. The suite and release
  build wait for the merge, as `which-checks.sh` decides for a branch.
- `cargo test --all-targets` — 5596 pass, 1 ignored, 0 fail.
- `bash scripts/guards.sh` for both new records — each reddens exactly the tests
  it names and nothing else.
- `cargo test --test house_style` — 52 pass, including the guard header count.
- `--no-verify` was never used. No `.git/index.lock` was encountered.

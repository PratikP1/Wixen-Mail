---
phase: 01-folders-and-conversations
plan: 06
subsystem: presentation
tags: [settings, guards, folder-tree, accessibility, sqlite, schema-additive, keyboard]

requires:
  - phase: 01-05
    provides: "folder_tree::rows, TreeRow and WhichRow: the pure module whose rows this words, and the collapsed-state store it reads"
  - phase: 01-03
    provides: "folders.parent_id, which is what makes a subtree total something that can be counted"
provides:
  - "config::every_setting_is_acted_on::test_every_setting_somebody_can_change_is_offered_by_a_screen: the D-43 mirror, with its exceptions checked in both directions"
  - "application::folder_settings::SETTINGS_SECTION: one home for the Reading page group name, D-42"
  - "application::folder_settings::UnreadOnAParent: D-24's two options, stored, read back by words, unrecognised values falling to the default"
  - "AppConfig::unread_on_a_parent: the first of the phase's five settings, offered and read"
  - "folder_tree::unread_text: how a row that holds rows says what is unread, and which number is which"
  - "folder_tree::TreeRow::worded: one place that turns a name and two counts into a row's words"
  - "application::account_order::moved: D-14's move, what it says, and the whole new order"
  - "accounts.tree_order and MessageCache::set_account_order: where an account sits, kept on this computer"
  - "ID_MOVE_ACCOUNT_UP and ID_MOVE_ACCOUNT_DOWN on the Action menu, Alt+Shift+Up and Alt+Shift+Down"
affects: [01-07, 01-08, 01-09, 01-10, 01-11, 01-12, 01-13]

actuals:
  tokens: 61000
  tasks: 3
  commits: 3

tech-stack:
  added: []
  patterns:
    - "A guard's exception list carries a claim the guard itself tests, in both directions: that the reason still holds, and that it has not become obsolete"
    - "A row that can be re-worded holds the parts its wording is built from, never the rendered text to cut back up"
    - "An absence is defended by a source read with a companion test proving the reading can see the thing when it is there"

key-files:
  created:
    - src/application/folder_settings.rs
    - src/application/account_order.rs
  modified:
    - src/data/config.rs
    - src/presentation/wx_settings.rs
    - src/presentation/folder_tree.rs
    - src/presentation/wx_app.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/accounts.rs
    - src/application/mod.rs
    - tests/house_style.rs
    - tests/folder_tree_rows_pair_with_the_control.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/KEYBOARD_SHORTCUTS.md
    - .planning/phases/01-folders-and-conversations/deferred-items.md

key-decisions:
  - "The mirror guard was red on arrival naming six settings, and narrowing it would have hidden the one that mattered. The six are four different things and only allowed_per_account is the defect: it is stored, honoured all the way out to the provider clients, and offered by nothing"
  - "Each exception names the screen that offers the setting instead, and a second test checks that claim, so deleting the control there fails rather than going quiet"
  - "A third test asserts the recorded defect is still a defect, so whoever wires a control is told to delete the entry rather than leaving a list that reads as a live problem after it is fixed"
  - "The plan's unread_text would have had no caller. Wiring it took rows() taking the setting and the collapsed set, TreeRow carrying two counts and its bare name, and the handler that hears a branch open wording that one row again"
  - "TreeRow holds the name rather than cutting it back off the label, because an account named Smith, John would lose half its name at the first comma"
  - "An account branch and the local group hold no mail of their own, so they read 46 unread in all rather than a bare count that does not say which of the two numbers it is"
  - "load_accounts orders by tree_order IS NULL, tree_order, created_at, so an untouched database keeps arrival order and an account added after a move goes to the end rather than the top"
  - "The move writes every account's ordinal, not the two that swapped, because a list half ordered by choice and half by arrival reorders itself the next time an account is added"
  - "No version bump: 0.46.0 is still untagged and unreleased, so it is the accumulating version and this work belongs inside it"

requirements-completed: []
requirements-advanced: [FOLDER-02]

coverage:
  - id: D1
    description: "A setting stored and offered by no screen fails a test, which the existing guard is structurally unable to see (D-43)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/data/config.rs#every_setting_is_acted_on::test_every_setting_somebody_can_change_is_offered_by_a_screen"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#every_setting_is_acted_on::test_a_setting_said_to_be_offered_elsewhere_really_is"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#every_setting_is_acted_on::test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing"
        status: pass
      - kind: other
        ref: "guards/guards.toml: a stored setting that no screen offers is caught (measured, reddens exactly 1)"
        status: pass
    human_judgment: false
  - id: D2
    description: "A collapsed parent's unread announcement counts its children and says which of the two numbers it is giving (FOLDER-02 criterion 4)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_a_row_holding_unread_mail_underneath_gives_both_numbers_and_names_each"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_a_row_that_holds_no_mail_itself_says_which_number_it_is_giving"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_a_folder_row_gives_both_numbers_and_a_leaf_beside_it_gives_one"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_an_account_branch_counts_every_folder_under_it_and_says_which_number_that_is"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_the_group_for_this_computer_counts_what_is_under_it_the_same_way"
        status: pass
    human_judgment: true
    rationale: >
      The words are proven, and the words are what this plan can prove. Whether NVDA and
      Narrator read them the way they read every other row is not proven here and cannot be
      without a screen reader. That is Pratik's to run, and it is the same limit 01-05
      recorded for the level and expanded-state announcements on these same rows.
  - id: D3
    description: "The setting is offered on a real screen by keyboard, defaults to both numbers always, and is read by the code that builds a row (D-24)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/application/folder_settings.rs#tests (7 tests: stored form, words form, unrecognised values, the two options differing)"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_the_default_gives_both_numbers_whether_the_row_is_open_or_closed"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_the_other_option_words_a_row_from_whether_that_row_is_closed"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#tests::test_a_settings_file_written_before_these_existed_reads_the_way_it_should"
        status: pass
      - kind: other
        ref: "src/data/config.rs#every_setting_is_acted_on: both guards, offered by a screen and read by something"
        status: pass
    human_judgment: true
    rationale: >
      That the control exists, is labelled, carries an accessible name and writes the answer
      back is proven by source and by the two settings guards. That somebody using a screen
      reader hears the group heading and the choice the way the rest of that page is heard
      needs a screen reader on the built program.
  - id: D4
    description: "Accounts appear in the order they were added, move with Alt+Shift+Up and Down, announce the new position, and reordering never touches a server (D-14)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/data/message_cache/accounts.rs#tests::test_accounts_nobody_has_moved_come_back_in_the_order_they_were_added"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/accounts.rs#tests::test_an_account_moved_down_stays_moved_after_the_cache_is_reopened"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/accounts.rs#tests::test_an_account_added_after_a_move_goes_to_the_end_rather_than_the_front"
        status: pass
      - kind: unit
        ref: "src/application/account_order.rs#tests (8 tests: both directions, both ends, the sentence, one account, nothing lost)"
        status: pass
      - kind: other
        ref: "src/application/account_order.rs#nothing_here_reaches_a_server (3 tests, including one proving the reading can see such a call)"
        status: pass
      - kind: other
        ref: "guards/guards.toml: moving an account never reaches a server (measured, reddens exactly 1)"
        status: pass
      - kind: other
        ref: "tests/wired.rs: both ids raised and handled, no two menu items claim one shortcut, the document and the menus agree"
        status: pass
    human_judgment: true
    rationale: >
      Everything except the keystroke itself is proven. Whether Alt+Shift+Up reaches this
      program rather than the Windows layout switch, and what a screen reader says when it
      does, needs the built program on a machine with more than one keyboard layout. The
      limit is written into docs/KEYBOARD_SHORTCUTS.md rather than left for somebody to
      discover, with two ways round it.
  - id: D5
    description: "The name of the settings group is written once and read from one constant (D-42)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "tests/house_style.rs#test_the_settings_screen_does_not_write_the_section_name_out_itself (now over every section constant, not one)"
        status: pass
    human_judgment: false

duration: 1h 4m
completed: 2026-08-30
status: complete
---

# Phase 01 Plan 06: The settings guard's mirror, what a parent says about unread mail, and the order accounts sit in

**A setting that is stored and offered by no screen now fails a test, and the
first thing that test found was a real one that had been in the tree for some
time: `allowed_per_account` is honoured all the way out to the provider clients
and no screen has offered it since the testing page stopped naming an account.**

## Performance

- **Duration:** about 1 hour 4 minutes
- **Started:** 2026-08-30T13:48Z
- **Completed:** 2026-08-30T14:52Z
- **Tasks:** 3 of 3
- **Files created:** 2. **Files modified:** 13
- **Lines:** 1,897 added, 120 removed

One full library run, at 176 seconds, to confirm nothing else moved before the
last commit. Two `guards.sh` runs, one per new record. Everything else was
targeted, at about a second each except the config module, which reads files
from disk and takes 25.

## What was actually built

### The D-43 mirror, and the six settings it named

The existing guard walks every shipping file except `config.rs` and
`wx_settings.rs`, by name, with a stated reason, so it catches a setting that is
offered and ignored and cannot see one that is stored and never offered. The
mirror asks the opposite question of `wx_settings.rs`, reusing
`stored_setting_names` and `what_ships` rather than writing a second parser.

**It was red on arrival, naming six of the forty-four stored settings.** The
temptation in that moment is to read six failures as the check being too strict.
Reading each apart, they are four different things:

| What | Which | Why it is not the defect |
|---|---|---|
| Offered by the account manager | `directories`, `default_account_id` | Both are per account, so they belong on the screen that lists accounts |
| Offered by a menu item | `mute_message_reading` | `ID_MUTE_CONTENT`, with a check on it, because it is reached in a hurry when somebody walks into the room |
| Not a setting anybody chooses | `last_filed_into`, `told_about_the_alpha` | Values the program writes down for itself |
| **The defect** | `allowed_per_account` | Stored, read by `allowed_for`, honoured out to the provider clients, offered by nothing |

Narrowing the guard until it went quiet would have hidden the one that mattered
along with the five that did not. So all six are named in three lists whose names
say which category they are, and **each list carries a claim the guard itself
tests**:

- `test_a_setting_said_to_be_offered_elsewhere_really_is` reads the named file
  and fails if the control has gone, so an exception cannot rot into a lie.
- `test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing`
  fails when a screen does start offering it, so whoever fixes it is told to
  delete the entry rather than leaving a list that reads as a live defect after
  it has been closed.

`allowed_per_account` is written up in `deferred-items.md` with what closing it
takes: a per-account group on the settings screen, which is a feature rather
than a line.

### What a row that holds rows says about unread mail

`Archive` holding three of its own and thirty-eight underneath used to read
"Archive, 3 unread". It reads "Archive, 3 unread here, 41 in all". A row that
holds no mail of its own, which is every account branch and the "On this
computer" group, reads "Work, 46 unread in all, 3 folders" rather than a bare
count that does not say which of the two numbers it is. A folder with nothing
under it still reads "Inbox, 5 unread", because there are not two numbers to
tell apart.

D-24's setting is a `Choice` on the Reading page in its own group, read back by
the words shown rather than by the row number, so an unrecognised value falls to
the default rather than to whichever branch is written first.

### The order accounts sit in

`tree_order`, nullable, added through `ensure_column_exists`, with
`ORDER BY tree_order IS NULL, tree_order, created_at`. An untouched database
keeps arrival order, and an account added after a move goes to the end rather
than the top. `Alt+Shift+Up` and `Alt+Shift+Down` on the Action menu, both with
menu items because on Windows a shortcut without one is not a shortcut.

## Task Commits

1. **Task 1: the mirror guard, the group constant** — `136a954`
2. **Task 2: how a parent counts its unread, offered and read** — `e55909e`
3. **Task 3: move an account with the keyboard** — `33d0c8b`

## On the RED and GREEN gates

`workflow.tdd_mode` is on. Every behaviour here was written test-first, and every
RED was taken against a body that compiled and did nothing, never against a
missing symbol.

| What | RED against a do-nothing body | GREEN |
|---|---|---|
| The mirror guard, against one deliberately added `AppConfig` field | 1 of 1, naming the field | green after the field was removed |
| The same, against two added fields | reported both, not the first | — |
| The two exception checks, each broken in turn | 1 each, and the main guard then named `allowed_per_account` | green after both probes were reverted |
| `UnreadOnAParent`, everything returning the default | 4 of 7 | 7 |
| `unread_text` returning its own count and ignoring the rest | 9 of 13, then 11 of 13 | 42 in the module |
| `account_order::moved` returning the list unchanged | 7 of 8 | 8 |
| The account ordering, with the `ORDER BY` put back to `created_at` | 2 of 3 | 25 in the module |

### The four tests that survived the stub, and what was done

**Two were real controls and were kept as they are.** `unread_text` returning
nothing for a row with nothing unread, and returning one number when both
numbers are equal, are what the stub also did. They assert an absence, so a body
that says less than it should satisfies them; their discriminating partners are
the tests above them in the same table. The same is true of
`test_no_account_is_lost_or_repeated_by_a_move`, which a body that moves nothing
passes by construction and which guards a different mistake: a swap writing one
id into both places.

**Two were the 01-05 trap and were strengthened.**
`test_the_default_words_a_closed_row_and_an_open_one_alike` and
`test_a_row_worded_again_when_it_closes_says_what_the_rebuild_would_have_said`
both asserted only that two wordings were *equal*, which a body ignoring whether
a row is closed satisfies for free. Each was given a case that must **differ**
beside the case that must match, and re-running the same stub reddens both. That
took the count from 9 to 11.

**The measurement that was worth its cost.** Taking the mirror guard red against
a deliberately added field is what proved it can see the thing it was written
for. Taking its two exception checks red separately is what proved the
exceptions are not decoration. And when `allowed_per_account` was temporarily
pulled out of its exception list, the main guard named it, which is the proof
that the guard is not passing over an empty set.

## Deviations from Plan

Four wrong premises and one thing the plan asked for that would not have worked.
Every one was found by reading or running the code before writing anything.

### Wrong premises

**1. The mirror guard cannot be green on the tree as it stands**

- **Found during:** the premise check, before task 1
- **Issue:** The plan's acceptance criterion is that the guard goes red against a
  deliberately added field and green after it is removed. It does not go green:
  six settings already fail the naive question, and one of them is a real defect
  that this plan cannot close.
- **What was built:** three named exception lists, each checked in both
  directions, and a `deferred-items.md` entry for the real one. The alternative
  was to narrow the guard until it could not see any of the six, which would
  have cost both its reach and the record of what it found. That is the lesson
  01-05 wrote down about its own source read.

**2. `folder_settings::SETTINGS_SECTION` would have had one reader**

- **Found during:** the premise check, before task 1
- **Issue:** `allowed::SETTINGS_SECTION` exists because `changes_waiting_here`
  says the section's name in a sentence, so two places read it. Nothing outside
  the settings screen names the folder group, and this plan adds no such
  sentence, so the new constant would be a literal that had moved house. D-42's
  own wording is conditional: "if the group's name is ever named in a sentence
  elsewhere".
- **What was built:** the constant, and a second reader that makes it
  load-bearing now.
  `test_the_settings_screen_does_not_write_the_section_name_out_itself` was
  generalised from the one Allow Changes constant to a list of every section
  named by a constant, so the new group is held to the same rule. The test keeps
  its name, because three planning documents cite it.

**3. Task 2's `unread_text` would have had no caller**

- **Found during:** the premise check, before task 2
- **Issue:** Task 2's file list is `config.rs`, `wx_settings.rs`,
  `folder_tree.rs` and the changelog. `folder_tree::rows` does not take the
  setting and nothing calls `unread_text`. Both settings guards would have
  passed: the module reading its own setting counts as something reading it, and
  the control counts as a screen offering it. So following the task exactly
  produces a setting that is offered, stored, read, fully tested, and has no
  effect on anything drawn.
- **What was built:** `rows` takes the setting and the set of what is closed,
  `TreeRow` carries the two counts, its bare name and its folder count, and
  `wx_app` passes both at the rebuild. This is why `wx_app.rs` was touched in
  task 2 as well as task 3.

**4. The plan's call-counting test for the reorder has nothing to count**

- **Found during:** task 3
- **Issue:** The plan asks for "a test that counts the calls" made during a
  reorder. There are none to count, and that is the point: the thing being
  defended is an absence, and a connection nobody opened leaves no trace a
  behaviour test can find.
- **What was built:** a source read over the shipping half of both files, with a
  companion test proving the reading can see such a call when there is one, and
  a third asserting the function it reads still exists under that name. It
  matches call syntax rather than bare words, because the paragraphs explaining
  the rule name every one of the things it forbids, and 01-05 already had a
  guard fire on the comment explaining it.

### One thing that needed more than the plan asked

**The second option would only have been right until the next sync.** With
"both numbers only while it is closed", the label depends on a state the user
changes with the arrow keys. Wording a row at build time alone means closing a
branch leaves it reading the open way until something rebuilds the tree, which
is a setting that half works. So `remember_the_row` now words that one row again
and writes both the control and the vector held beside it, because a row is
paired back to its identity by the words above it and a control saying one thing
while the vector says another is a row the cursor cannot be put back on. One
place words a row, `TreeRow::worded`, and a test asserts the rebuild and the
handler agree.

### Auto-fixed issues

**5. [Rule 1 - Bug] `TreeRow::name_only` would have broken an account named "Smith, John"**

- **Found during:** task 2, writing the re-wording
- **Issue:** My first version took the row's name back off the rendered label by
  cutting at the first comma. A folder name cannot hold a comma, because
  `import_tree`'s wrapper around `safe_file_name` checks it, but an account's
  display name is whatever somebody typed. "Smith, John" would have lost half its
  name every time the row was worded again.
- **Fix:** `TreeRow` holds `name` beside `label`, and the wording is built from
  the parts rather than cut out of the result.
- **Committed in:** `e55909e`

## Files Modified

- `src/data/config.rs` — the mirror guard and its two exception checks, three
  exception lists, `unread_on_a_parent` and its default, the older-settings-file
  case. The existing `test_every_setting_somebody_can_change_is_read_by_something`
  is untouched: `git diff` on task 1 shows 163 insertions and no deletions.
- `src/application/folder_settings.rs` — new. `SETTINGS_SECTION` and
  `UnreadOnAParent` with 7 tests.
- `src/application/account_order.rs` — new. `Move`, `Moved`, `moved`, 8 tests,
  and the three-test source read that says nothing here reaches a server.
- `src/presentation/folder_tree.rs` — `unread_text`, `local_group_text`,
  `TreeRow::worded`, `unread_underneath`, `plain_row`; `rows` and `nested` take
  the setting and what is closed; `folder_text` and `branch_text` take both
  counts. 14 tests added, 43 in the module.
- `src/presentation/wx_settings.rs` — the group, the choice, the field on both
  control structs, three destructurings, the write-back by the words shown.
- `src/presentation/wx_app.rs` — `say_the_row_again`, `move_the_chosen_account`,
  two command ids, two menu items, the dispatch, and the setting and collapsed
  set passed at the rebuild.
- `src/data/message_cache/mod.rs` — the `tree_order` column.
- `src/data/message_cache/accounts.rs` — `set_account_order`, the ordering, 3 tests.
- `tests/house_style.rs` — the section-name check now covers every section constant.
- `guards/guards.toml` — two records, header count 320 to 322.
- `docs/KEYBOARD_SHORTCUTS.md` — both keys and the Alt+Shift warning, with two
  ways round it and how to change the Windows key.
- `docs/changelog.md` — two entries with their known limits.

## The user-visible change

A folder that holds folders says what is unread inside it and which number is
which, so a closed branch can no longer report almost nothing while holding
forty messages nobody has seen. You can choose whether both numbers are always
given or only while a row is closed, in Settings, Reading, under Folders and
Message Lists.

You can put your accounts in the order you want them with Alt+Shift+Up and
Alt+Shift+Down. It says the name and the new position, and says so when there is
nowhere further to go. The order lasts, and nothing about it reaches a server.

**No version bump.** `Cargo.toml` stays at 0.46.0. This is a behaviour change
and a schema change, both of which CLAUDE.md says bump in the same commit, and
it is not bumped for the reason 01-03 and 01-05 both gave and this agrees with:
`git tag --list` is empty, 0.46.0 has never been released, so it is the
accumulating unreleased version and this work is inside it. Flagged rather than
decided quietly.

## What is not built, and why

**A control for `allowed_per_account`.** Named as a known gap in the guard's own
exception list and written up in `deferred-items.md`. It is a per-account group
on the settings screen, a control per account, a write-back and new wording for
the sentence a sync says, none of which is in this plan.

**The reorder is not visible in the folder tree yet.** `folder_tree::rows` keeps
whatever order it is handed and there is a test for it, and `load_accounts` now
hands them over in the stored order. But the sender still gathers one account, so
one branch is drawn, which is 01-05's known limit and is blocked on D-18 and
D-19. Today the new order shows in the account list. This is in the changelog
where somebody using the program will read it.

## FOLDER-02 is not ticked, and this is the reason

All four of FOLDER-02's criteria are now met in code. The fourth, the one 01-05
left open, is this plan's: a collapsed parent counts its children and the
announcement says which of the two numbers it is giving.

It is still not ticked, and the reason is the second guardrail rather than
anything missing. Two of the four criteria say a screen reader announces
something: criterion 1 says the level comes from the native `TreeCtrl` rather
than the label, and criterion 4 is about what somebody hears. No test in this
repository can answer either. Ticking the requirement would tick those two with
it and claim something no screen reader has confirmed, which is precisely the
failure the guardrail names: sixteen widgets once compiled, passed 324 tests, and
had no accessible name at all.

01-05 made the same call on criteria 1 and 2 and recorded it as human judgment.
This follows it. What is left is one NVDA and Narrator pass over the folder tree,
which is Pratik's to run, and the tick belongs to whoever has watched it happen.

## Known Stubs

None. Every function added here is called from a path a person can reach:
`unread_text` through `TreeRow::worded` from both the rebuild and the
expand-and-collapse handler, `UnreadOnAParent` from the settings screen and from
the tree, `moved` and `set_account_order` from the two menu items.

## Threat Flags

None. The two boundaries this plan touches are the ones its own threat register
names: a settings file being deserialised, mitigated by
`#[serde(default = "...")]` and a test for the older shape, and a keyboard action
writing an ordinal to the accounts table, which is a local write with a source
read and a guard record saying it goes nowhere else.

## Issues Encountered

**The house-style test's name was kept although its body changed.** Three
planning documents cite
`test_the_settings_screen_does_not_write_the_section_name_out_itself` by name.
Renaming it to match its wider job would have made three documents wrong, so the
name stayed and the doc comment above it says what it now covers.

**The flaky spellcheck test did not appear.** The one full library run here was
clean at 5,471 passing, 1 ignored. 01-05 recorded that test failing about one run
in five; one clean run says nothing either way about it, and it is left in
`deferred-items.md` where 01-05 put it.

**`--no-verify` was never used and no `.git/index.lock` was encountered.** Every
commit went through `git commit` directly rather than through a workflow wrapper,
which is what two earlier executors reported leaving a lock behind.

## Self-Check: PASSED

Files claimed as created:

- FOUND: `src/application/folder_settings.rs`
- FOUND: `src/application/account_order.rs`
- FOUND: `.planning/phases/01-folders-and-conversations/01-06-SUMMARY.md`

Commits claimed:

- FOUND: `136a954`
- FOUND: `e55909e`
- FOUND: `33d0c8b`

Checks:

- `bash scripts/check.sh` — formatting and clippy pass. The suite and the release
  build wait for the merge, as `which-checks.sh` decides for a branch.
- `cargo test --lib` — 5,471 pass, 1 ignored, 0 fail.
- `cargo test --test wired` — 58 pass, including both new ids raised and handled,
  the no-two-items-claim-one-shortcut check, and the document agreeing with the
  menus.
- `cargo test --test house_style` — 52 pass, including the guard header count.
- `cargo test --test checkbox_labels`, `--test docs_links`,
  `--test folder_tree_rows_pair_with_the_control` — all pass.
- `bash scripts/guards.sh` for both new records — each reddens exactly the one
  test it names and nothing else.

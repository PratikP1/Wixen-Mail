---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 38
subsystem: the Saved Searches submenu, a key per saved search, the folder tree and the message list's key handling
tags: [saved-searches, keyboard, menus, GAP-09, "#58"]
status: complete
requires: [13-37, 12-10]
provides:
  - "saved_searches::REACHABLE_BY_KEY, key_for, what_the_menu_says, nothing_there"
  - "wx_app::rebuild_the_saved_search_menu and put_the_saved_searches_on_the_menu, the submenu rebuilt from the account's searches"
  - "an item arm that lands the tree's cursor on the search's row and runs it the way Enter does"
  - "wx_app::answer_the_saved_search_keys_the_menu_cannot on the folder tree and the message list"
affects: [13-39, 13-42, 13-51]
tech-stack:
  added: []
  patterns: ["a posted Alt key is held in the thread's keyboard state too, because accelerators read the state and the key event reads the message"]
key-files:
  created:
    - tests/the_saved_searches_menu_says_the_searches_an_account_has.rs
  modified:
    - src/application/saved_searches.rs
    - src/presentation/wx_app.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "The menu and the keys follow the account whose branch the tree's cursor was last in, rebuilt on arrival as well as on load, so the item beside a key is always the search the key runs"
  - "A key or an item switches to Mail before it runs the search, as View All Inboxes does"
metrics:
  duration: about 3 hours to the documents commit
  completed: 2026-09-30
actuals:
  tokens: 48000
  tasks: 3
  commits: 5
---

# Phase 13 Plan 38: A key per saved search, and a Saved Searches menu that lists them Summary

**`Alt+4` to `Alt+9` now run the first six saved searches of the account being worked in,
and Action, Saved Searches lists that account's searches in the order the folder tree shows
them, each of the first six with its key, then Edit Conditions, Rename and Delete.** A key or
an item puts the tree's cursor on the search's row and runs it the way `Enter` does. A key
with no search, pressed in the folder tree or the message list, says "Alt+7 runs saved search
4, and this account has 3."

## What runs now

`UIUpdate::SavedSearchesLoaded` stores the searches and calls
`put_the_saved_searches_on_the_menu` with the active account's names, walked through
`every_saved_search` over `SavedSearchesRead.order`. The folder tree's selection handler
calls it too when the cursor arrives in an account's branch, so the menu follows the account
being worked in between loads. The function reads the items whose ids fall in the block
`ID_SAVED_SEARCH_FIRST[SAVED_SEARCHES_ON_THE_MENU]` and rebuilds only when they differ from
`saved_searches::what_the_menu_says`. The builder calls `rebuild_the_saved_search_menu` with
no names, so the three commands are written once.

The item arm, `_ if saved_search_position_of(id).is_some()`, switches to Mail, takes the
search at that place from the same list, holds it as chosen, puts the cursor on its row with
`select_row` and runs it through `the_search_a_row_names` and `run_a_saved_search`. A search
a newer version wrote is refused there in `SAVED_BY_ANOTHER_VERSION`'s words.
`answer_the_saved_search_keys_the_menu_cannot` is bound on the folder tree and the message
list and answers through `send_status`, the status line and an announcement at Normal.

## The key, measured

In the target's process, on a tree and a list built on a frame carrying the real menu bar
with three searches on it, `WM_SYSKEYDOWN` posted through the window's loop:

| Key | Tree | List | Frame |
|-----|------|------|-------|
| `Alt+7`, no item | handler answered place 4 | handler answered place 4 | no command |
| `Alt+4`, an item | not answered | not answered | the first search's id, once per control |

The first run posted the keys with the Alt bit in the message and nothing else, and `Alt+4`
reached the tree as a key while the frame got no command, and so did `Alt+1`, the Folder
Pane's long-standing accelerator. `TranslateAccelerator` reads Alt from the thread's keyboard
state; wxWidgets' key event reads it from the message. The reading now holds Alt down with
`SetKeyboardState` while the keys are delivered, as a real key would, and puts the state back.
The target's head comment says so.

## Evidence

| Claim | Test or reading | Count before, after |
|-------|-----------------|---------------------|
| the first six have keys, the seventh none | `test_the_first_six_searches_have_keys_and_the_rest_do_not` | `application::saved_searches::` 76, 80 |
| a name's ampersand claims no letter | `test_a_name_with_an_ampersand_claims_no_letter` | same |
| no searches, no lines | `test_no_searches_make_no_lines` | same |
| a key past the last says which and how many | `test_a_key_past_the_last_search_says_which_key_and_how_many_there_are` | same |
| the menu read after load, after the same again, after a rename and a move, past the sixth, with none | seven readings of the real menu bar | target 0, 16 |
| the keys reach the tree and the list, and an item runs | two readings, measured above | same |
| the loaded arm puts them on the menu; the item arm lands, then runs | two source readings, four companions | same |
| the page's keys are ones the code states | `tests/wired.rs` stated list | `wired` 77, 77 |
| nothing in the window's own tests moved | | `presentation::wx_app::` 199, 199 |

Every expected value was reasoned before its green: Alt+4 is place 1 plus 3, Alt+9 place 6;
`nothing_there(4, 3)` names Alt+7; the rename and move give Receipts, Invoices, then the
unreadable search at Alt+6. The red for the menu's words borrowed the labels' rule, so
`test_no_searches_make_no_lines` was red against five starting lines rather than green
against an empty stub. The same-searches reading passes against a menu nothing rebuilds, so
it was taken red by hand in the green by making the comparison always differ ("emptied and
filled again"), then put back.

## Guard records

| Record | Break | Red | Measured |
|--------|-------|-----|----------|
| the Saved Searches menu gives a key to the first six searches and to no seventh (new) | range to seven | the keys case | 2026-09-30, exactly that |
| the Saved Searches menu is rebuilt from the searches an account has when they load (new, suite the target) | the call dropped from the arm | the loaded arm reading | 2026-09-30, exactly that |
| the Saved Searches menu is rebuilt when what it says differs from the searches (new, suite the target) | the comparison never true | six readings: three menus, the key question, both key measurements | 2026-09-30, exactly those |
| three records on `saved_searches.rs` (flagged) | unchanged | 3, 2, 1 | re-measured, agree |

The arrived-since count at the top of `guards/guards.toml` went 510 to 513. 1,310 records.

## Commits and hook times

| Commit | What | Hook |
|--------|------|------|
| `06f2f9d7` | test: four cases for the menu's words and keys red, the count check named | red, 123 s (a first try stopped at rustfmt after 11 s) |
| `fdca10b0` | feat: the words and keys as one rule, one record | affected, 233 s (a first try stopped at rustfmt after 12 s, on the untracked target) |
| `395c95ac` | test: the target's eight readings red | red, 225 s (a first try refused after 234 s: the names carried the target's path) |
| `20de1c47` | feat: the rebuild, the arms, the key handler, the stated keys, two records | affected, 419 s (a first try stopped at clippy after 29 s) |
| documents | pages, changelog, ledger, marks, this summary | see the merge |

## Deviations from Plan

**1. [Rule 1] The menu and the keys also follow the tree's cursor into another account.** The
plan called the put from the `SavedSearchesLoaded` arm only. Arrowing into another account's
branch changes the active account without reading the sidebar again, so the menu would have
shown one account while the key ran the other's search at that place (T-13-38-01). The
selection handler now puts the arriving account's searches on the menu, rebuilding only when
they differ.

**2. [Rule 2] A key or an item switches to Mail first,** as View All Inboxes does, because the
accelerator reaches the arm from every module and results loaded behind another module are
not seen.

**3. [Rule 1] The search is held as chosen before the cursor lands.** Landing on a saved
search's row by arrow says "press Enter to run"; said over a search the key is already
running, it would be wrong, so the arm sets `selected_folder` first and the handler reads the
landing as no arrival.

**4. [Measurement] Alt held in the thread's keyboard state.** See the key, measured, above.

**5. [Clippy] The item arm is `_ if saved_search_position_of(id).is_some()`,** as the labels'
is, not the written-out range the plan named: clippy refuses a manual range. The reading's
anchor follows it, and `tests/wired.rs`'s `id >= ` marker is not needed because nothing
raises the block's first id by name.

**6. [TDD] One green commit for task 2, not three.** The plan asked for the ids and the
rebuild, then the arms, then the key handler, each built. The gate runs the target on every
commit and a partial green leaves named reds failing, so the three landed together;
`cargo build` and `cargo clippy --all-targets` ran before it.

**7. [Brief] The task 2 red named its readings with the target's path first,** and the gate
refused it as never run; a reading in a target is named bare, which 13-37's summary already
said. Named bare the second time.

**8. [Date] The dated sentence on the Action menu row says 2026-09-30,** the day the row was
corrected, not the plan's 2026-09-24, as 13-37 did.

**9. [Counts] `wx_app.rs` is named by 152 records, not the plan's 119,** taken with the
parser on 2026-09-30; no test was added to it, so none was flagged.

### Found and left

- `answer_the_label_keys_the_menu_cannot` and the new handler read keys the same way and could
  share a helper; left, since the labels' is private and untouched by this plan.

## Ledger

Opened, both halves: 739 (`unrun-verify`, the tester's ear on the items with their keys, the
answer to a key past the last, and the cursor landing on the row a key ran). None closed.
Counts 739 in all, 658 open, 81 fixed, 0 waived.

## Threat Flags

None beyond the register. T-13-38-01 by one list read by the menu, the item arm and the key
handler, and by the menu following the account on arrival; T-13-38-02 by the doubled
ampersand and its case; T-13-38-03 by the comparison and the same-searches reading;
T-13-38-04 by the handler, measured on built controls. No crate added (T-13-38-SC).

## Self-Check: PASSED

The target exists; `06f2f9d7`, `fdca10b0`, `395c95ac` and `20de1c47` are on the branch.
`fn rebuild_the_saved_search_menu` once, `rebuild_the_saved_search_menu(` three times and
`put_the_saved_searches_on_the_menu(` three times in `wx_app.rs`;
`ID_SAVED_SEARCH_FIRST[SAVED_SEARCHES_ON_THE_MENU]` once; `    if shown != wanted {` once;
`saved_searches::key_for` once in `tests/wired.rs`.

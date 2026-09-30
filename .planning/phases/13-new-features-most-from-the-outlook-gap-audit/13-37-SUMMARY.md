---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 37
subsystem: saved searches in the store, the folder tree's reordering gesture, the main window's move
tags: [saved-searches, folder-tree, reordering, GAP-09, "#58"]
status: complete
requires: [12-10, 13-36.4]
provides:
  - "saved_searches.position, a new search placed last in its account, number_the_unnumbered_saved_searches run on every open"
  - "MessageCache::put_saved_searches_in_order, writing one account's whole order in one transaction"
  - "SavedSearchesRead.order, every search's id, readable or not, in the kept order"
  - "folder_tree::WhatMoves::SavedSearch and wx_app::move_the_chosen_search"
affects: [13-38, 13-39, 13-51]
tech-stack:
  added: []
  patterns: ["a read split into two lists carries the one order beside them, and every reader that shows or rearranges walks that order"]
key-files:
  created:
    - tests/a_saved_search_moves_with_the_gesture.rs
  modified:
    - src/data/message_cache/mod.rs
    - src/data/message_cache/saved_searches.rs
    - src/presentation/folder_tree.rs
    - src/presentation/wx_app.rs
    - src/application/favourites.rs
    - src/application/context_menu.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "The move reads the account's searches from the store, not from the window's copy, so a key held down moves from the order last written rather than from a tree not yet read back"
  - "A search with no place sorts ahead of the numbered ones in the order it was made, which only an older build writing to this database between two opens can leave"
metrics:
  duration: about 2 hours 40 minutes to the documents commit
  completed: 2026-09-30
actuals:
  tokens: 62000
  tasks: 3
  commits: 5
---

# Phase 13 Plan 37: Saved searches keep an order somebody chooses Summary

**Alt+Shift+Up and Alt+Shift+Down on a saved search's row now move it one place within its
own account's searches and say where it is, "Invoices, 2 of 4.", in the words accounts and
pinned folders use; the order is kept on this computer and the folder tree shows it.** Move
Up and Move Down on This Folder, and two new entries on a saved search's own menu, reach the
same move.

## What runs now

The keys, the This Folder items and the row's menu all send `ID_MOVE_UP` or `ID_MOVE_DOWN`
to `move_the_chosen_row`. `folder_tree::what_the_gesture_moves` answers
`WhatMoves::SavedSearch { account, id }` for a saved search's row, and the new arm calls
`move_the_chosen_search`. That reads the account's searches from the store, lists them
through `every_saved_search` (the tree's own list), moves one with `reordering::moved`, writes
the whole order with `put_saved_searches_in_order`, reads the tree back, shows the sentence
and announces it at High. First or last already is said through `CommandAnswered`, and a
row not in the list gets `WHICH_SAVED_SEARCH`.

The store: `saved_searches.position` is added with `ensure_column_exists`; a new search
takes `MAX(position) + 1` in its account; `get_saved_searches_for_account` orders by
`position, created_at, id` and fills `SavedSearchesRead.order` with every id, readable or
not; `number_the_unnumbered_saved_searches` runs on every open beside
`number_the_unnumbered_labels`, not fatal, and numbers rows with no place after the ones that
have one, per account, in `created_at` then `id` order. `every_saved_search` walks `order`,
so a search a newer version wrote sits at its own place and a move across the readable and
unreadable split is shown.

The refusal on other rows now reads "Move Up and Move Down rearrange accounts, pinned
folders and saved searches. Choose an account branch, a folder under Favourites, or a saved
search." `THE_COMMANDS` in `favourites.rs` names `fn move_the_chosen_search`, so the check
that nothing on the path reaches a server reads it.

## Evidence

| Claim | Test or reading | Count before, after |
|-------|-----------------|---------------------|
| the order written is the order read, another account untouched | `test_the_order_written_is_the_order_read_back` | `data::message_cache::saved_searches::` 27, 30 |
| a new search goes last | `test_a_new_search_goes_last_in_its_accounts_order` | same |
| older rows numbered on open in the order made, readable and unreadable interleaved, after a numbered row | `test_searches_from_before_the_order_keep_the_order_they_were_made` | same |
| the gesture answers a saved search row | `test_the_gesture_moves_an_account_on_an_account_row_and_a_pin_on_a_pinned_row`, gained a case in place | `presentation::folder_tree::` 97, 97 |
| the row's menu offers both moves | `test_a_saved_search_row_offers_only_what_works_on_a_saved_search`, gained two asserts | `application::context_menu::` 19, 19 |
| the tree walks the kept order | `test_every_saved_search_gets_a_row_including_the_ones_this_build_cannot_read`, rewritten in place | `presentation::wx_app::` 199, 199 |
| the arm reaches the move, the move reads the tree's list, writes before reading back, the tree reads `.order` | four readings, six companions and a shaped-window pass | `a_saved_search_moves_with_the_gesture` 0, 11 |
| nothing reaches a server | `nothing_here_reaches_a_server` reading `fn move_the_chosen_search` | `application::favourites::` 19, 19 |

`data::message_cache::tags::` 13 green, unchanged. `data::message_cache::` 855 green at the
first green.

Every expected value was reasoned before its green: positions 2 to 5 for c, a, d, b after
e at 1 from their made days; `t2` skipped by the account filter so `acc-2` stays B, A;
`s4` at `MAX + 1 = 4`, where a NULL would have sorted first. In each red the assertion, not
a compile error, failed, except the older-database case, whose red was the missing column
the `UPDATE ... position = NULL` names.

## Guard records

| Record | Break | Red | Measured |
|--------|-------|-----|----------|
| saved searches are read in the order kept for them (new) | `ORDER BY created_at, id` | the three store cases | 2026-09-30, exactly those |
| the gesture on a saved search reaches the move (new, suite the target) | the arm sent to `WHICH_ROW` | `test_the_gesture_on_a_saved_search_reaches_the_move` | 2026-09-30, exactly that |
| a saved search row is one the gesture moves (new) | the `WhichRow::SavedSearch` arm removed | the folder tree's positive case | 2026-09-30, exactly that |
| a saved search keeps the question set the In box was narrowed to (flagged) | unchanged | 3 tests | re-measured, agrees |
| what a saved search found is listed in the sort that was chosen, not a fixed one (flagged) | unchanged | 1 test | re-measured, agrees |

The arrived-since count at the top of `guards/guards.toml` went 507 to 510. 1,307 records.

## Commits and hook times

| Commit | What | Hook |
|--------|------|------|
| `0042a889` | test: the three store cases red, the count check named | red, 222 s |
| `defb36ea` | feat: position, the pass, the read's order, one record | affected, 315 s (a first try stopped at rustfmt after 13 s) |
| `66ef13b5` | test: gesture, menu, pinning check, tree order and the target's four readings red | red, 238 s |
| `d6fd3b64` | feat: the arm, the move, the tree's walk, the words, the menu entries, two records | affected, 417 s |
| documents | pages, changelog, ledger, marks, this summary | see the merge |

## Deviations from Plan

**1. [TDD] The task 2 red names a fifth library case,**
`test_pinning_a_folder_makes_no_call_that_leaves_this_machine`. It walks `THE_COMMANDS` and
reads each function's body, so a name the file does not hold yet panics there too. The plan
predicted only the name check. Observation 0919.

**2. [Rule 3] The task 1 red touched `wx_app.rs`.** Its three `SavedSearchesRead` literals
had to carry `order` to compile once the field existed; the plan put that in task 2. The
fixture's order was written as task 2 wanted it (the unreadable search first) so task 2 only
rewrote the one assertion.

**3. [Rule 1] The move reads the store rather than the window's copy of the searches.** The
plan said to list them from the held `saved_searches`. The tree is read back through an
update the window applies later, so a key held down would compute the second move from the
order before the first and write it back. A pin's move already reads the store for the same
reason. Same order either way, since the tree is drawn from the store.

**4. [Record] The first measurement of the store record named its tests bare,** and
`guards.sh` said the harness never ran them: a library record names the module path, a
record with a `suite` names the bare test. Corrected and measured again, so task 1 took two
`--remeasure` calls instead of one.

**5. [Date] The dated sentences say 2026-09-30, not 2026-09-24.** The plan was written on
the 24th and wrote that day into the shortcuts page's correction and the changelog's dated
line; the lines were corrected today, and the acceptance line asking for 2026-09-24 is
answered with the day it happened, as 13-36.3 did with its backlog clause.

**6. [State] `STATE.md`'s prose position line had stayed at 44 while its frontmatter and its
`Current Plan:` line were at 45,** left that way by 13-36.4. Both now read 46, and the prose
keeps the line it replaced.

**7. [Brief] One read-only command began with a do-nothing assignment named after a banned
tool.** It ran nothing and changed no file. Observation 0920.

**8. [Tool] The Folder Actions check in task 3's acceptance line uses a tool the brief
forbids;** it was answered by reading the section: both table rows name the saved search.

### Found and left

- `src/application/reordering.rs`'s module comment still says two commands share the
  wording; there are three now. The plan kept that file untouched and the sentence misleads
  nobody about behaviour.
- `move_the_chosen_row`'s doc comment carries two paragraphs about accounts left above it by
  an earlier move of `move_the_chosen_account`; only the "two" was corrected.

## Ledger

Opened, both halves: 738 (`unrun-verify`, the tester's ear on the sentence after a move, the
cursor staying on the moved row, the new refusal and the two menu entries). None closed.
Counts 738 in all, 657 open, 81 fixed, 0 waived.

## Threat Flags

None beyond the register. T-13-37-01 and -02 by `order` and the older-database case;
T-13-37-03 by `put_saved_searches_in_order`'s account filter, the `t2` row in the first case;
T-13-37-04 by `THE_COMMANDS`. No crate added.

## Self-Check: PASSED

The five code and test files and the new target exist; the four commits are on the branch.

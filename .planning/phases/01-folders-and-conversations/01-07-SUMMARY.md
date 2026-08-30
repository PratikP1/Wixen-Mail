---
phase: 01-folders-and-conversations
plan: 07
subsystem: data
tags: [local-folders, migration, schema-additive, sqlite, folder-tree, accessibility, guards]

requires:
  - phase: 01-05
    provides: "folder_tree::rows and WhichRow, the pure tree this regroups by owner rather than by path"
  - phase: 01-06
    provides: "TreeRow::worded and the unread counts the regrouped branches carry"
provides:
  - "application::local_folders::THIS_COMPUTER and is_this_computer: the reserved account id the shared folders are stored under, and the one place that recognises it"
  - "application::local_folders::SHARED_BY_EVERY_ACCOUNT: D-18's five, pinned as an exact list"
  - "application::local_folders::used_by: what a protocol reaches here, as against what it owns"
  - "application::local_folders::stored_under: which account a local folder's row is under, asked once rather than at each site"
  - "messages.original_uid and messages.original_account_id: what a merged message had and whose it was"
  - "MessageCache::move_message_recording_its_origin and messages::Renumbering: an ordinary move that writes down where it came from"
  - "MessageCache::merge_local_folders and shared_folders::MergeReport: D-19's migration, its counts and its sentence"
  - "guards: the merge never puts away a folder that still holds mail; a moved message is renumbered for the folder it lands in"
affects: [01-08, 01-09, 01-10, 01-11, 01-12, 01-13]

actuals:
  tokens: 23000
  tasks: 3
  commits: 3

tech-stack:
  added: []
  patterns:
    - "A per-account collection and a per-account reach are two questions; collapsing them makes an Option-returning accessor fall into its own fallback branch"
    - "A migration that carries on past one failure makes its own defensive clauses reachable, and makes found-against-moved a number that can differ"
    - "A destructive statement carries its own precondition in its WHERE clause, so there is no window between asking and doing"

key-files:
  created:
    - src/data/message_cache/shared_folders.rs
  modified:
    - src/application/local_folders.rs
    - src/application/local_delete.rs
    - src/application/sent_copy.rs
    - src/application/blocking.rs
    - src/application/import_tree.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/messages.rs
    - src/presentation/folder_tree.rs
    - src/presentation/wx_app.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/phases/01-folders-and-conversations/deferred-items.md

key-decisions:
  - "The human gate was reached, stopped at and returned rather than answered. Pratik answered proceed, with corrections, and corrected D-18 himself in c335e8e"
  - "for_account and used_by are two functions because they are two questions. Taking the five out of the per-account arrays turned local_trash into None, and None is handled: Delete on a POP account became a permanent removal of the only copy, measured as Some(RemoveFromThisComputer)"
  - "The merge reuses move_message rather than writing a second mover. The plan's premise that nothing here had ever rewritten a uid was false, and a separate mover would have missed filed_here, whose absence makes a row the next sync deletes"
  - "Origin is recorded for every moved message, not only renumbered ones, so the merge is reversible. Pratik's call at the gate"
  - "A message that cannot move is skipped rather than aborting the run. The early return made the emptiness guard on the folder delete unreachable, which a break measured as reddening nothing"
  - "The old per-account folder rows are deleted once empty, against one line of the plan and in line with D-19, the plan's own invariant and the tree"
  - "No version bump. 0.46.0 is untagged and unreleased, so it is the accumulating version, following 01-06"

requirements-completed: []
requirements-advanced: [FOLDER-01, FOLDER-02]

coverage:
  - id: D1
    description: "Sent, Outbox, Drafts, Junk and Trash are one each, shared across accounts under On this computer (D-18)"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/application/local_folders.rs#tests::test_the_five_shared_folders_are_the_five_and_the_inbox_is_not_one_of_them"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#local_folders_at_start_up::test_two_accounts_share_one_set_rather_than_making_one_each"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_the_shared_folders_are_listed_once_however_many_accounts_there_are"
        status: pass
  - id: D2
    description: "Only Inbox stays per account, and an IMAP account keeps its server folders while sharing the one Outbox (D-18 as corrected)"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/application/local_folders.rs#tests::test_an_imap_account_makes_no_folders_of_its_own_here"
        status: pass
      - kind: unit
        ref: "src/application/local_folders.rs#tests::test_an_imap_account_reaches_only_the_outbox_here"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#local_folders_at_start_up::test_an_imap_account_still_gets_somewhere_to_queue_mail"
        status: pass
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_a_pop_accounts_inbox_stays_under_its_account_although_it_is_local"
        status: pass
  - id: D3
    description: "An existing database is migrated message by message, nothing removed until it has landed, and a count reported (D-19)"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/data/message_cache/shared_folders.rs#tests::test_two_accounts_trash_becomes_one_and_the_report_names_both"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/shared_folders.rs#tests::test_a_merge_that_stops_partway_has_landed_what_it_moved_and_lost_nothing"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/shared_folders.rs#tests::test_the_count_is_the_returned_report_rather_than_a_log_line"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/shared_folders.rs#tests::test_a_second_open_finds_nothing_to_move"
        status: pass
  - id: D4
    description: "Two accounts whose local Trash both hold uid 42 both survive, with the original recorded beside the new number (D-40)"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/data/message_cache/shared_folders.rs#tests::test_two_messages_numbered_the_same_both_survive_and_one_records_its_old_number"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/shared_folders.rs#tests::test_every_moved_message_records_where_it_came_from_not_only_the_clashing_ones"
        status: pass
      - kind: guard
        ref: "guards/guards.toml#a moved message is renumbered for the folder it lands in"
        status: pass
  - id: D5
    description: "A folder a user creates under a POP account goes under that account's branch and is never gated (D-20)"
    requirement: FOLDER-02
    verification:
      - kind: unit
        ref: "src/presentation/folder_tree.rs#tests::test_a_folder_somebody_made_under_a_pop_account_stays_under_that_account"
        status: pass
      - kind: unit
        ref: "src/application/local_folders.rs#tests::test_a_folder_somebody_made_under_their_account_stays_theirs"
        status: pass
  - id: D6
    description: "The shape cannot drift back to per-account local folders without a test going red"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/data/message_cache/shared_folders.rs#tests::test_afterwards_no_account_keeps_a_copy_of_a_folder_that_is_shared"
        status: pass
  - id: D7
    description: "Deleting mail in a folder on this computer still moves it to the Trash rather than destroying it"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/application/local_folders.rs#tests::test_a_pop_delete_never_becomes_permanent_because_the_trash_was_not_found"
        status: pass
  - id: D8
    description: "The merge never deletes a folder row that still holds mail, which would cascade and destroy it"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/data/message_cache/shared_folders.rs#tests::test_a_folder_that_still_holds_mail_is_never_put_away"
        status: pass
      - kind: guard
        ref: "guards/guards.toml#the merge never puts away a folder that still holds mail"
        status: pass

status: complete
---

# Phase 01 Plan 07: Shared Local Folders Summary

Sent, Outbox, Drafts, Junk and Trash are now one each rather than one per account, existing mail is moved into them on open with a count spoken and logged, and colliding message numbers both survive with the original and the account recorded beside the new one.

## What was built

**A reserved account id.** `THIS_COMPUTER` is `"\u{1}This computer"`, carrying a character an account id cannot, the same trick `LOCAL_PREFIX` plays with paths one level up. `is_this_computer` is the only place that recognises it, and nothing anywhere spells the literal out. `folders` keeps `UNIQUE(account_id, path)` untouched: the shared folders are ordinary rows whose account is the reserved one.

**Two questions where the plan had one.** `for_account` now says what an account makes under its own id, which is an Inbox for POP and nothing for IMAP. `used_by` says what it reaches, which is all six for POP and the shared Outbox for IMAP. `stored_under` says which account a given local path's row is under. Everything that used to search `for_account` for a kind was asking the second question; the three places that resolve a local path to a row were asking the third.

**The migration.** `MessageCache::merge_local_folders` runs once at open from `MessageCache::new`, non-fatally, beside `migrate_inline_bodies`. It follows that function in all five of its properties, and its `MergeReport` carries what it found before moving anything, what it moved, which accounts the mail came from and how many were renumbered. The sentence is spoken at `Priority::High` and written to the status bar through `UIUpdate::CommandAnswered`, and the counts go to the log.

**Two additive nullable columns.** `messages.original_uid` and `messages.original_account_id`, written for every message the merge moves rather than only the ones whose number had to change. Null for a message that never moved, which is a true answer rather than a missing one. No column was dropped or renamed and neither `UNIQUE` constraint was touched.

## The gate

Task 1 was a `checkpoint:decision` with `gate="blocking-human"`. It was reached, stopped at and returned rather than answered, and the return described what the merge would do, to which rows, and what undoing it would take. Three defects found while verifying the plan's premises were returned with it.

Pratik answered **proceed, with the corrections**, and three things came back with the answer:

1. Defects 2 and 3 to be treated as corrections to the plan rather than followed as written.
2. **The Outbox is shared for everyone and `FOR_IMAP` becomes empty.** D-18 was self-contradictory and Pratik corrected it himself in `c335e8e` before this plan built against it. The reason given: one send queue on this computer, because a queued message already knows which account sends it.
3. **The origin account is recorded too**, for every moved row, so the merge is reversible.

He also asked that the migration report what it is about to move as well as what it moved, measured rather than estimated, and that the multi-account tree unblocking be noted and left alone. Both done.

## Deviations from plan

Four wrong premises, one of them the plan's central justification. The first two were found before the gate and put to Pratik with it; the last two were found while building.

### 1. [Rule 4 - Architectural, resolved at the gate] D-18 said the Outbox was both shared and per account

**Found during:** verification before Task 1.
**Issue:** D-18 said Sent, Outbox, Drafts, Junk and Trash become one each, and in the same sentence that `FOR_IMAP` stays `[Outbox]`. The plan repeated both and asked for a test pinning each side. They cannot both hold: `for_account` is what creates the per-account rows, so leaving the Outbox in `FOR_IMAP` means every IMAP account goes on making its own.
**Resolution:** Pratik's, not mine. `FOR_IMAP` is now `[LocalFolder; 0]`, pinned by `test_an_imap_account_makes_no_folders_of_its_own_here`, whose comment records why.

### 2. [Rule 1 - Bug] Shrinking the arrays turned Delete on a POP account into a permanent removal

**Found during:** verification before Task 1, then measured in Task 2.
**Issue:** The plan's Task 2 named two files and said to leave `is_local` alone so no existing decision changed its answer. `is_local` was not the seam. `local_sent` and `local_trash` are derived from `for_account` by searching it and returning `Option`, so taking the five out made them return `None` — which compiles, and which at `local_folders.rs` falls through to `LocalDelete::RemoveFromThisComputer`.

The measured red, from `test_a_pop_delete_never_becomes_permanent_because_the_trash_was_not_found`:

```
Delete on a POP account stopped moving mail to the Trash: Some(RemoveFromThisComputer)
```

`ensure_local_folders` built its rows from the same arrays, so an IMAP account, which now owns none, failed on the way in:

```
its folders: Other("This account has no folders")
```

That is every IMAP account, at startup.
**Fix:** `used_by` and `stored_under`, and eight call sites moved onto whichever of the three questions they were actually asking. The three that resolve a local path to a folder row — `sent_copy::file_here`, `wx_app::replace_local_draft` and `local_delete::folder_here` — all looked the shared folder up under the account and would have found nothing.
**Files:** `local_folders.rs`, `local_delete.rs`, `sent_copy.rs`, `blocking.rs`, `import_tree.rs`, `wx_app.rs`.
**Commits:** `36ff777` (red), `8339950` (green).

### 3. [Rule 1 - Bug] "Nothing in this tree has ever rewritten a uid" was false, and it was the reason for the design

**Found during:** verification before Task 1.
**Issue:** The plan said this in its objective, its action text and its threat model, and specified a fresh uid mechanism on that basis. `MessageCache::move_message` has rewritten `folder_id` and `uid` in one statement for some time. It also maintains `filed_here`, and its own doc comment says why in as many words: "the step that forgets mail the server no longer lists reads the same marker, so an unmarked row is one the next sync deletes." The plan never mentions `filed_here`. A separate mover would have written rows a later sync removes, inside the migration whose whole purpose is not to lose mail.
**Fix:** `move_message_recording_its_origin` delegates to the same private implementation `move_message` does. The evidence that they really are one path is the guard record: breaking the renumbering reddens five tests, three of them about ordinary moves rather than about the merge.
**Commit:** `c68feeb`.

### 4. [Rule 1 - Bug] The plan asked for the old folder rows to be both kept and gone

**Found during:** Task 3.
**Issue:** Two adjacent acceptance criteria. One said the old per-account folder rows are "emptied, not deleted". The next said that after the merge no folder row outside the reserved account id has a shared-kind path. Both cannot hold. An emptied per-account Trash is also still a second Trash under that account's branch in the tree, which is the repetition this plan removes.
**Fix:** the row is deleted once every one of its messages has landed, which is D-19's plain reading. `folders` cascades to `messages`, so the emptiness test is part of the `DELETE` statement rather than a check before it, and it has a guard record.

## What the plan did not know it needed

**The merge no longer stops at the first failure, and that is what made its own guard reachable.** The first implementation returned on the first message it could not move, following `migrate_inline_bodies`. Applying the break for the emptiness clause on the folder delete then reddened **nothing at all** across the whole library: the loop returned before the delete ever ran with a non-empty folder, so the clause was unreachable rather than merely untested. It also meant one unmovable message in one account's Trash would leave every other account's mail unmerged on every open for ever, and that `found` could never differ from `moved` in a returned report. The loop now skips and counts, which fixes all three at once, and the same break reddens two tests.

**`ensure_local_folders` had no test and is the only maker of these rows.** Three were added.

## Not taken on

**Showing every account in the tree at once.** 01-05 recorded this as blocked on D-18, and D-18 has now landed, so it is unblocked. `folder_tree::rows` was already multi-account throughout and this plan changed its grouping from the path to the owner, so the tree module itself needs nothing further. What remains is upstream: `UIUpdate::FoldersLoaded` carries `Vec<String>` for one account, and the rebuild is driven per account. Left alone at the coordinator's instruction, and noted here so it is decided somewhere visible rather than in this plan.

## Known limitations

- **Nothing here has run against a real account.** The merge has been driven against caches on tempfiles, which is where `migrate_inline_bodies` is tested too. No real database has been migrated.
- **Nothing has been confirmed with a screen reader.** The merge's sentence goes out at `Priority::High` on `CommandAnswered`'s own topic, so it should not be coalesced away by a first sync's status. That is a reading of the announcement path, not a run of NVDA.
- **The merge is one-way inside the program.** `original_uid` and `original_account_id` hold what is needed to undo it, for every moved message. There is no command that does.
- **A latent defect found and deferred, not fixed.** `next_local_uid` saturates on `i64` and then casts to `u32`, so a folder whose highest number is `u32::MAX` gets `0` rather than a saturated value. Not reachable in any database this program can produce. Recorded in `deferred-items.md` with how it was found, which was a test passing for the wrong reason.

## Process notes

**A mistake worth recording.** `scripts/guards.sh` was backgrounded after exceeding a foreground timeout, and `cargo fmt --all` was then run while it still held the tree, against the instruction in CLAUDE.md to run it with nothing else building. The tree at that moment held a deliberate break in `src/application/calendar.rs`. It was caught by reading `git status` and finding a modified file nobody had edited. The run completed and restored cleanly, both guard runs were then done alone, and the full suite is green, so nothing was lost. The instruction was still broken.

No stale `.git/index.lock` was encountered. Commits were made with `git` directly, never through a wrapper, and `--no-verify` was never used.

**Version:** not bumped. 0.46.0 is untagged and unreleased, so it is the accumulating unreleased version and this work belongs inside it, following 01-06's reasoning. This is a schema change and a behaviour change, so the question was asked rather than skipped.

## TDD Gate Compliance

This project's convention is plain descriptive commit messages rather than conventional-commit prefixes, so the gates are named here by hash instead.

- **RED:** `36ff777`. Ten tests red from assertions, not from missing symbols: the new constants and functions were stubbed first so the code compiled. Two of the ten are the ones worth having, the exact-list pin on the shared five and the Delete-becomes-permanent guard.
- **GREEN:** `8339950` for Task 2, `c68feeb` for Task 3.
- Task 3's own red was measured per test before implementing: six of nine red from assertions, and the three that passed against the do-nothing body were examined. One of them, the partial-failure test, passed only because a merge that moves nothing also loses nothing. It was rewritten to assert that two messages did land, so it now discriminates.

## Verification

- `cargo test --all-targets --no-fail-fast`: all green, 5496 in the library plus every integration target.
- `cargo clippy --all-targets --all-features -- -D warnings`: clean. No `#[allow(...)]` added, no `unwrap` or `expect` outside `mod tests`.
- `bash scripts/guards.sh merge`: 7 guards, all reddening exactly the tests their records name, including the new one.
- `bash scripts/guards.sh renumbered`: the new renumbering guard, five tests, exactly as recorded.
- Both new records were measured by hand against the whole library before being written down, and the header count in `guards/guards.toml` went from 322 to 324 in the same edit.
- `git diff main..HEAD -- src/data/message_cache/mod.rs` shows no dropped or renamed column and no change to either `UNIQUE` constraint.

One regression was caused and fixed during the work: the new `#[cfg(test)]` module in `wx_app.rs` was first placed mid-file, and `tests/wired.rs` cuts that file at the first `#[cfg(test)]`, so two of its guards reported the functions they read as gone. The module was moved to the end, where this file's other test modules live.

## Commits

| Commit | What |
| --- | --- |
| `36ff777` | Red: the five stop being per account, the reserved id is stubbed, ten tests fail on assertions |
| `8339950` | Green: `used_by` and `stored_under`, eight call sites, the tree grouping by owner |
| `c68feeb` | The merge, the two origin columns, the report, two guard records, the changelog |

## Self-Check: PASSED

Every file named exists, all three commits resolve, and all eighteen tests named in `coverage` resolve to a `fn` in the tree. `guards/guards.toml` holds 516 records against a header of 192 + 324, which `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it` checks. The only `UNIQUE` line in this plan's diff of `mod.rs` is a comment; no constraint, column or table was dropped or renamed, and both new columns are nullable. The shipping half of `shared_folders.rs` holds no `unwrap`, no `expect` and no `#[allow(...)]`.

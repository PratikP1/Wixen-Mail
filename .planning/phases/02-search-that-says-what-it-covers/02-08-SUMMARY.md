---
phase: 02-search-that-says-what-it-covers
plan: 08
subsystem: search
tags: [saved-searches, folder-tree, multi-account, identity, guards]
status: complete
requires:
  - "folder_tree::WhichRow and stored(), the row identities 01-03 and D-25 built"
  - "folder_tree::the_account_a_row_belongs_to, the answer 01-14 added for folders"
  - "folder_tree::favourite_rows and application::favourites::in_account_order, D-29's group"
  - "wx_app::every_saved_search and ChosenSearch, the two shapes a tree row resolves to"
  - "data::message_cache::saved_searches::get_saved_searches_for_account, already per account"
  - "application::saved_searches::THAT_FOLDER_IS_NOT_HERE, the refusal a vanished folder gets"
provides:
  - "folder_tree::WhichRow::SavedSearchesIn, one account's part of the saved-search group"
  - "folder_tree::WhichRow::SavedSearch carrying the account as well as the identifier"
  - "folder_tree::SearchInTheTree::account, populated at the boundary"
  - "folder_tree::saved_search_rows, the group drawn the way Favourites is"
  - "application::favourites::what_each_account_has and WhatOneAccountHas, the one ordering both groups use"
  - "application::saved_searches::THAT_ACCOUNT_IS_NOT_HERE"
  - "wx_app::WhoseMail and whose_mail_a_saved_search_reads"
  - "wx_app::ChosenSearch::account, on both shapes"
  - "wx_app::names_a_rename_may_not_use"
  - "wx_app::WxUIState::saved_searches keyed on the account"
  - "one guard record, and one re-measured"
affects:
  - "the Favourites group, which now orders its accounts through the shared function"
  - "UIUpdate::SavedSearchesLoaded, which carries every account's searches"
  - "run_a_saved_search, which no longer takes the held state at all"
  - "guard records: one added, one re-measured, one re-anchored"
tech-stack:
  added: []
  patterns:
    - "one account-ordering function for every group that mirrors the account structure"
    - "a signature that cannot see the held state, in place of a check that it is not read"
    - "a row identity carrying every part of what the row is, even where one part is unique on its own"
    - "a fixture whose two accounts differ, so a wrong answer cannot be right by luck"
decisions:
  - "The account comes off the row and is carried on ChosenSearch, read at the call site, rather than the selection handler setting the working account and the commands reading it back. Both happen, but nothing that runs, renames or removes a saved search reads the held account: a handler that has to run first is an ordering nobody can see, and 01-14's defect was exactly two commands answering one question from two places."
  - "run_a_saved_search stopped taking AppHandles and takes the channel and the runtime. Nothing in it needed the state once the account came off the row, and a signature that cannot see the held account is a guarantee no reading of the body has to be trusted for. It also makes the guard record's break a real one: the break now sits where the account is decided rather than where it is used."
  - "The saved-search row's stored spelling carries the account although the identifier is already unique across the table. The tree treats the spelling as the whole of the row, so a part of the identity that is not in the spelling is a part nothing downstream can get back. This is stated in the test rather than left as a habit, because the uniqueness argument is the one somebody would reach for to take it out again."
  - "in_account_order was not copied for saved searches; the part both groups share was pulled out as what_each_account_has and in_account_order became one of its two callers. Two groupings of the same accounts is the shape that comes apart, and the test compares the two groups' account order rather than restating either."
  - "state.saved_searches is a map keyed on the account rather than one flat list. A name is unique inside an account and not across them, so a flat list would refuse a new name because somebody else's mail already used it."
  - "A saved-search row is resolved by the identity beside it rather than by its words. Two accounts each holding a search called Invoices are two rows reading exactly the same, and the words could only answer with whichever came first."
metrics:
  duration: one session
  completed: 2026-09-01
actuals:
  tokens: 30000
  tasks: 3
  commits: 6
---

# Phase 2 Plan 8: Saved searches inside the account structure Summary

**It works, and it is reached by a live command.** Saved Searches in the folder
tree now holds a branch per account that has one, with that account's searches
under it, in the same order accounts appear everywhere else in the tree. An
account with no searches contributes no branch and no account with a search
means no heading at all. Opening a saved search runs it against the account its
row sits under, and so do renaming one and running it again after its conditions
change.

**The thing this plan actually fixes is not the drawing.** Before it, every
saved-search command took its account from `active_account_id`, which Set Active
changes while leaving the folder tree's cursor where it was. Moving the rows
under account branches without changing that would have given a search sitting
under account B, opened, run against account A: another account's mail listed
under a name given to this one, or an empty answer indistinguishable from a
search that legitimately matched nothing.

What has not happened is a run. Nothing here has been drawn in a running build
or heard under a screen reader. Four entries went to `.planning/WINDOWS.md`.

The library suite and every integration target are green: 5,974 library tests,
no failures. `scripts/check.sh all` was not run, as instructed; the release
build is the merge gate's.

## Reached by a person, and how

Guardrail 1 says a feature is done when a non-test path reaches it, so this is
traced rather than assumed.

**The branches are drawn.** `folder_tree::rows` calls `saved_search_rows`
(`folder_tree.rs`), which `folder_tree_updates` at `wx_app.rs:9909` feeds through
`UIUpdate::FoldersLoaded`. `folder_tree_updates` has eleven callers, including
the module load, every finished sync and every command that changes the tree.
`fill_the_tree` at `wx_app.rs:13763` walks rows by depth and is depth-agnostic,
which is why the branch at depth one and its searches at depth two need nothing
of it.

**The account travels with the row.** `every_saved_search(&account.id, theirs)`
fills `SearchInTheTree::account` per account inside `folder_tree_updates`;
`saved_search_rows` groups on it and puts it on `WhichRow::SavedSearch`;
`the_search_a_row_names` reads it back off the row and puts it on
`ChosenSearch`.

**The run takes it from there.** `run_a_saved_search` is raised from Enter on a
saved-search row (`wx_app.rs:2483`, through `which_row`), from Refresh Folder on
one (`wx_app.rs:3452`) and from the end of an edit of its conditions
(`wx_app.rs`, after `replace_saved_search`). All three hand it a `ChosenSearch`
whose account came off the row.

**Landing on one says whose mail it is.** The selection handler's
`WhichRow::SavedSearch { .. }` arm now calls
`folder_tree::the_account_a_row_belongs_to` and sets `active_account_id`, the
same two lines the folder arm has had since 01-14.

## What was checked before any row identity changed

The plan's own truth says the stored identity of the saved-search heading must
be unchanged. Phase 1 lost stored state twice this way, so what is keyed on a
row identity was traced rather than assumed.

**`tree_state` is the only table keyed on a `WhichRow::stored()` value.** Its
three writers:

- `set_row_collapsed(&which.stored(), collapsed)` at `wx_app.rs:13465`, called
  only from `remember_the_row`, called only from the tree's `on_item_expanded`
  and `on_item_collapsed`. Those fire for rows with children.
- `set_folder_view` and `set_thread_column`, both keyed on
  `the_folder_being_looked_at`, which maps through `WhichRow::opens()`.
  `opens()` answers `Some` for `Folder` and `Pinned` and `None` for everything
  else.

**So a saved search's own row identity is never written to disk.** It is built
with `expandable: false` and nothing is appended under it, so no expand or
collapse event can name it, and `opens()` is `None` for it so no view or column
choice is kept under it. Everything else that uses it is in memory and rebuilt
each run: `state.folder_ids` (folders only, from `FolderIdsLoaded`),
`select_row`, `land_the_folder_cursor` and `the_folder_row_the_cursor_was_on`.
The one stored identity read back from configuration at startup is
`WhichRow::AllInboxes.stored()`, in `land_the_folder_cursor`.

**The heading's is written, so it is unchanged**, asserted as the literal string
`"saved-searches"` rather than as the expression that builds it, and taken red by
hand once by spelling it `"saved-searches-group"`: one test, the one written for
it.

**The new account branch is expandable, so its identity will be written**, and
that is what makes collapsing one account's searches survive a rebuild without
closing that account's folders. Its spelling collides with neither the heading's
nor a search's, asserted pairwise.

`git diff main -- src/application/saved_searches.rs` adds one constant and
touches no field of `SavedSearch`. Nothing about the stored data changed.

## The defect this closes, and how far the checks reach

**Nothing on the saved-search run path can read the held account, because it is
not handed it.** `run_a_saved_search` takes the channel, the runtime and the
chosen search. That is stronger than a check that the body does not read it, and
it was worth the signature change: the two are different claims and only one of
them is a guarantee.

`grep -c 'active_account_id\.clone()' src/presentation/wx_app.rs` is **34**,
against 35 before this plan. Reading the routine's line range shows no
`active_account_id` and no `lock_state`.

**Rename read the held account and now does not.** It computed the names a new
name may not clash with from `names_already_used(&held)`, which was the open
account's list. Renaming a search belonging to another account would have been
offered the wrong list in both directions: refusing a name that account does not
use, and accepting one it does, which then fails on the write after somebody has
typed it and been told it was fine. `names_a_rename_may_not_use` takes the
account off the search's own row.

**Delete did not read the held account, and here is the evidence.**
`delete_the_chosen_search` uses `chosen.name()` for the question and the
sentence and `chosen.id()` for the write, and `delete_saved_search` is
`DELETE FROM saved_searches WHERE id = ?1`. `rename_saved_search` and
`replace_saved_search` are keyed on the identifier the same way. Checked, not
assumed, because "checked and it was already right" and "not checked" look
identical afterwards.

**A vanished account is refused in its own words**, before the folder is
resolved. Resolved afterwards, an account that had gone would have come back as
`THAT_FOLDER_IS_NOT_HERE` for a folder-narrowed search, which sends somebody
looking for a folder that was never the problem, and as an empty list for every
other search. `THAT_ACCOUNT_IS_NOT_HERE` names the account and not a folder, and
a test asserts the two sentences differ and that the new one says "account" and
not "folder". A read of the accounts that fails refuses as well: running against
an account this cannot confirm is the one outcome that could list somebody
else's mail.

**How far the automatic checks reach, stated plainly.** That opening a search
under account B while account A is current returns B's mail is held by three
things together, none of them a live run: `the_search_a_row_names` answers with
the row's account and a test hands it a state where the held account is the
other one; the account is the only argument that narrows
`messages_a_saved_search_reads`, which
`test_a_search_reads_its_own_accounts_mail_and_nothing_marked_deleted` in
`src/data/message_cache/saved_searches.rs` already holds; and the run path
cannot reach any other account. `run_a_saved_search` itself opens its own
`MessageCache` from `AppPaths::resolve()`, which reads a process environment
variable, so a test cannot point it at a temporary directory without poisoning
every other test in the process. That is recorded rather than worked around.

## Red, green, and every break measured

Six commits, three red and green pairs.

| Commit | What |
|---|---|
| `96d4657` | RED, four over a row that knows whose it is |
| `04f1ccf` | GREEN, the identity and the account answer |
| `01e025f` | RED, six over a branch per account |
| `25a90d6` | GREEN, the branches, the per-account read, the lookup by identity |
| `28d08b6` | RED, three over the account a command acts on |
| `3f61e91` | GREEN, the run, the rename, the refusal, one guard, the changelog |

**Every RED half is wrong reachable code**, for the reason 02-06 and 02-07 both
recorded: a lint denying dead code refuses right unreachable code, so the stub
has to be a wrong answer rather than a missing one. Three shapes were used. The
identity spelling ignored the account and the branch borrowed the heading's
spelling. `whose_mail_a_saved_search_reads` had its two arms swapped, which is
the only break that keeps both variants constructed. `the_search_a_row_names`
filled the account from the held state, which is not an invention: it is what
the code did before this plan.

**Five breaks measured by hand.**

| Break | What reddened | Scope measured |
|---|---|---|
| the account dropped from the row's stored spelling | 1 | whole library |
| the searches split under two headings inside a branch | 4 | whole library |
| the account filled from the held state | 2 | whole library |
| `run_a_saved_search` handed the state again and reading it | 3 | `presentation::` |
| landing on a saved search not setting the account | 1 | `presentation::` |
| the heading spelled differently | 1 | `presentation::` |

The last three are the tests that were green on arrival, so each was taken red
by hand to prove it can see anything at all. The signature break reddens three
because two of them are 02-03's checks, which read the same routine through the
same helper and fail to find it when its signature changes. That is the helper
working, not a surprise.

## Guard records

544 records, and the sweep header now reads 192 + 352.

### Added: "a saved search takes its account from its row and not from the one last looked at"

The break is the code as it stood before this plan. Measured by hand against the
whole 5,974-test library: **two**, both written this session. Nothing older sees
it, which is the honest answer and is why the record exists.

### Re-measured: "one heading holds every saved search, however it was made"

Task 2 rewrote the block this record names, so it was measured again rather than
re-pointed. Four reddened before and four redden now, and **two of the four
changed**.

- `test_a_search_whose_questions_have_no_name_is_announced_like_any_other`
  stopped reddening, and that is this plan's doing. It asserted a saved search
  sits at depth one, written as a number; the branch went in between and moved
  it to two, so the test was rewritten to compare one search's depth with
  another's. Under the break both are still equal.
- `test_the_top_level_reads_in_the_order_somebody_meets_it` stopped too, because
  the break's extra headings now sit inside a branch at depth one and that test
  reads depth nought.
- The two that arrived are the ones written for D-2-05, which count headings and
  branches directly.

That is exactly the staleness `CLAUDE.md` describes, in the direction it warns
about less often: a record can name too many as well as too few, and a break
that no longer reaches a test is a guard claiming cover it does not have.

### Re-anchored, not re-measured, in the RED commit

The same record's `before` stopped matching the file the moment the RED half
changed how a saved-search row is built, and the commit gate said so. It was
re-anchored to keep the gate honest between the two commits and re-measured at
the end, in the shape it finished in. Both facts are in the commit messages.

**Not machine-verified beyond that.** `scripts/guards.sh` was not run, as
instructed. Three records were touched by hand and nothing else was re-measured,
so any record this branch made stale is unfound. The candidate set for
`scripts/guards.sh --touched-by 0e1caa7` is what would answer it.

## Wrong premises in the plan

**1. Task 1's file list is `src/presentation/folder_tree.rs` only, and its own
action cannot be carried out inside that file.** The action says to populate the
account "at the boundary, in `every_saved_search`", which is in
`src/presentation/wx_app.rs`. Nothing compiles without it, because `rows` builds
the row identity from `SearchInTheTree::account`. Task 1 touches both files.

**2. The plan's line numbers had moved.** `run_a_saved_search` was at `:6453`
rather than `:6278`, `every_saved_search` at `:10099` rather than `:9709`, and
`the_account_a_row_belongs_to` at `:664` rather than `:678`. The brief's
`folder_tree.rs:678` for the `SavedSearch(_) => return None` arm was right.
Named functions were used throughout, not the numbers.

**3. "Reuse `favourite_rows`'s ordering source" reads as though there were a
function to call, and there was not one that fits.**
`favourites::in_account_order` is typed on `Pin` and sorts by a pin's position,
which a saved search does not have. The plan's own next sentence says to move it
somewhere both can reach rather than write a second one, and that is what
happened: the shared part is now `what_each_account_has`, and `in_account_order`
is one of its two callers, keeping its own sort.

**4. Task 3's cross-account test as specified cannot be written.**
"Assert that a search belonging to the second account, run while the first is the
working account, returns the second account's messages" needs
`run_a_saved_search`, which opens its own `MessageCache` from
`AppPaths::resolve()`. That reads a process-wide environment variable, so
pointing it at a temporary directory would change it for every other test running
in the same process. What was written instead is above, under "How far the
automatic checks reach".

**5. The acceptance criterion "the same-named searches in two accounts have
different stored identities; removing the account from the identity makes it
fail" is not true as written, and the first version of that test passed on
arrival.** The identifier is unique across the whole `saved_searches` table, so
two searches of one name in two accounts already spell two identities with the
account dropped. What is really at stake is that the spelling must not lose half
of what the row is, and the test was rewritten to say that: two rows differing
only in the account must spell differently. That is red against the account being
dropped, and the comment records that nothing in the database produces that pair
and why the property still holds.

**6. The plan asks for the redraw to be measured "once with two accounts", which
gives a number with nothing to compare it to.** One account was measured as well,
so the per-account cost is visible rather than the total.

## Deviations from plan

**1. [Rule 2 - Missing critical] `run_a_saved_search` no longer takes the held
state.** The plan asks for the account to be taken from the row and for a source
reading to show the held account is not read. Once nothing in the routine needed
the state, keeping it in the signature would have left the wrong answer in reach
of the next person to edit the body, with only a text search standing between.
Removing it makes the guarantee a compile-time one. The plan's grep criterion
still holds and is stronger for it.

**2. [Rule 2 - Missing critical] `state.saved_searches` is keyed on the
account.** The plan says to move the read into the per-account loop and does not
say what holds the result. One flat list would have made `names_already_used`
answer across every account, and the table's uniqueness rule is per account, so
a new name would have been refused for clashing with a search in somebody else's
mail. Keyed on the account, the rename and the save each ask their own.

**3. [Rule 1 - Bug] A saved-search row is resolved by its identity rather than by
its words.** `the_search_a_row_names` matched on the row's label. Two accounts
each holding a search called Invoices are two rows reading exactly the same, so
Enter on either would have opened whichever the list held first. This is the same
truth the plan states about identities and it was not in any task; the map made
the label lookup impossible to keep, and the fix is what the first truth asks
for. `the_chosen_saved_search` is now the same lookup with a second way in
rather than a second lookup.

**4. [Rule 1 - Bug] The rename's name check read the account last looked at.**
Written up above. The plan asks for this to be checked and says to fix it if
found; it was found.

**5. [Rule 2 - Missing critical] `save_this_search` reads the account once.** It
read `active_account_id` twice, once for the name check and once for the write,
under two separate locks. The two could differ, and the second is the account the
search is stored under, so the names checked would have been the wrong account's.
Reached because the count criterion made the duplication visible.

**6. Two existing tests changed, both recorded with the reason.**
`test_the_shared_folders_and_the_headings_belong_to_no_one_account` no longer
lists a saved search's own row: it names an account now, and the reason the test
gives for the rows in it stopped being true of it. The heading stays.
`test_a_search_whose_questions_have_no_name_is_announced_like_any_other`
compared a depth against the number one; it compares two searches' depths now,
because the number was the one thing the test was not about.

**7. `test_nothing_on_the_way_from_storage_to_a_row_records_which_door_made_a_search`
caught the new field, which is the check working.** Both patterns destructure
with no `..`, so adding `account` to `SearchInTheTree` stopped the file
compiling. The comment now says why the account is not what D-2-02 forbids:
whose a search is and which door made it are different questions, and the
account is filled the same way for both doors.

## The cost this pays

Six per-account reads on every redraw instead of five, across eleven redraws,
some of them on a timer. `01-14` accepted the identical cost for folders and
this is the same trade with a measurement rather than an estimate.

Measured over a hundred redraws of `folder_tree_updates` in a debug build, with
two accounts holding their local folders and no mail: **201.8ms for two accounts
and 176.9ms for one**, so 2.02ms and 1.77ms per redraw. The second account costs
about 0.25ms, of which the saved-search read is one of six per-account reads.
A debug build and an empty cache, so the absolute numbers are not what a real
mailbox costs; the ratio is the part worth keeping. A later decision about
caching now has a number to start from.

## Known stubs

None. Everything this plan adds is reached from a command a person can raise,
traced above.

## Threat flags

None new. The four mitigations this plan owed are in place.

- **T-02-34**, a saved search run against the wrong account: the account is taken
  from the row for running, renaming and re-running, and the guard's recorded
  break is filling it from the held state, which is exactly the old code.
- **T-02-35**, a search returning nothing because it ran against the wrong
  account: the vanished account gets its own sentence before the folder is
  resolved, and a read of the accounts that fails refuses rather than running.
- **T-02-36**, two accounts' rows spelling one identity: the account is on the
  row identity with the length-prefix trick `Pinned` uses, and a test asserts the
  heading, the branch and the search are pairwise different.
- **T-02-37**, one more per-account read: accepted with the measurement above.

**T-02-SC** holds: no dependency added, `Cargo.toml` unchanged and still 0.46.0.

## WINDOWS.md

The ledger stood at **26 open** where 02-07 left it, which was checked in the
file rather than carried from the brief. Four added, taking it to **30 open**.

- The saved-search account branches have never been drawn in a running build.
- Landing on a saved search now moves the working account, and whether that is
  noticed by ear is unverified.
- The vanished-account refusal has never been reached.
- No saved search has been run against a real account with two accounts present.

## Documentation

`docs/changelog.md` has two `[Unreleased]` entries in the same commit as the
change: one under Added for the branches and the two rows a person can tell
apart, and one under Fixed saying plainly that opening a saved search used to
run it against whichever account you were last looking at. No version bump.

`docs/KEYBOARD_SHORTCUTS.md` is unchanged, and that is right: no accelerator
changed.

## Requirements

**Roadmap criterion 6 is met.** Saved searches sit inside the account structure
the way pinned folders do, and two accounts each holding a search of the same
name are two rows a caller can tell apart. What criterion 6 does not say, and
what this plan found, is that the drawing was the smaller half: the run had to
stop reading held state or the new tree would have made a latent defect
reachable.

SEARCH-03 was already marked complete by 02-07. Nothing here changes that.

## Deferred

Two things were found and left, both recorded in `deferred-items.md`: every
branch row in the folder tree still reports a folder's context menu, which has
been true of the Favourites branch and the Labels heading since 01-14; and two
accounts given the same name are still one row to `where_a_row_sits`, which the
comment on that function already records for the account branches themselves.
Neither is new and neither is this plan's to decide.

## Self-Check: PASSED

- All six commits found in `git log main..HEAD`.
- `cargo test --lib`: 5,974 passed, 0 failed. Every integration target green,
  including `folder_tree_rows_pair_with_the_control`, `house_style` and `wired`.
- `grep -c 'active_account_id\.clone()' src/presentation/wx_app.rs` is 34,
  against 35 at `main`.
- `sed -n '/^fn run_a_saved_search/,/^}/p' src/presentation/wx_app.rs | grep
  'active_account_id\|lock_state'` returns nothing.
- `grep -c '^\[\[guard\]\]' guards/guards.toml` is 544, equal to 192 + 352, and
  `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
  agrees.
- `git diff main -- src/application/saved_searches.rs` adds one constant and no
  field.
- `grep -n '^version' Cargo.toml` still reads 0.46.0.
- `docs/changelog.md` has both entries under `[Unreleased]`.
- The branch is `gsd/plan-02-08`, off `0e1caa7`, unmerged.

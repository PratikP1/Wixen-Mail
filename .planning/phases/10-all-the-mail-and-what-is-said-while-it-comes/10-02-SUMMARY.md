---
phase: 10-all-the-mail-and-what-is-said-while-it-comes
plan: 02
subsystem: the message list, the message cache's listing and label queries, the scale harness, guards
tags: [message-list, page, limit, labels, sqlite, measurements, guards, changelog, issue-24]

requires:
  - phase: 10-all-the-mail-and-what-is-said-while-it-comes
    provides: "10-01's model is untouched here; 10-01.1's tree at d020aa60 with the Settings pages painted after their controls, which this plan rides on and does not change"
  - phase: 08-every-number-the-project-quotes
    provides: "08-04's harness tests/the_list_at_two_hundred_thousand_rows.rs, the row shape docs/development/measurements.md accepts, and the Listing rows this plan says time a different query from the one the window runs"
provides:
  - "wx_app: load_folder_messages asks get_message_list_sorted for None; load_every_inbox asks unified_inbox for None; load_messages_with_label asks messages_with_label for None; WxUIState::message_list_limit, FOLDER_LIST_PAGE_SIZE and ALL_INBOXES_LIMIT gone; the Get Older Messages arm re-reads nothing before the sync and MoreOfTheFolderArrived re-reads and grows nothing"
  - "message_cache: tags_by_message_in_folder, tags_by_message_in_account and tags_by_message_in_every_inbox, one query joined through messages and folders with no parameter per row; get_tags_for_messages gone; unified_inbox and messages_with_label take Option<usize>; limit_clause shared"
  - "wx_app: attach_labels(items, read) taking the read and logging a failure rather than dropping it"
  - "tests/the_list_at_two_hundred_thousand_rows.rs: the list's own read path timed at 12,872 and 200,000, a refusal as a row, the command with --test-threads=1, rows named after 10-02"
  - "tests/the_list_holds_everything_the_folder_holds.rs: 12,872 rows read back whole through the window's query, the labels by folder at 12,872 and 40,000, a reading of wx_app.rs with companions"
  - "tests/a_whole_folder_moves_both_bounds.rs: the arm a chunk reaches held to re-reading and naming no limit, with a companion"
  - "docs/development/measurements.md: sixteen rows for the list's own read path, before and after, and the paragraph that says which to hold against which"
  - "guards/guards.toml: two new records with suite naming the new target, the both-bounds record rewritten and renamed, the harness record corrected; 881 records, census 802 + 79"
  - "docs/changelog.md: the #24 entry under Unreleased, Fixed; ledger 515"
affects: [10-05 (Get Older Messages means carry on downloading, this folder first, and the list has no limit for the runner to grow; a chunk arriving is shown by reread_folder_if_open), 10-07 (reads MAIL-02's clauses and the [S] line in ledger 515), the tester, who settles whether 12,872 reads as one list]

actuals:
  tokens: 25700
  tasks: 3
  commits: 9

tech-stack:
  added: []
  patterns:
    - "A labels read keyed by the listing the rows came from, never by the ids read, so the row count is never a parameter count"
    - "A measurement harness that records a refusal as a row whose value is the word refused and whose conditions carry the reason, with the statement cut off"
    - "A before-and-after pair checked against a control row whose code no commit touched, and the before commit re-run in a worktree beside the after rows when the control moves"

key-files:
  created:
    - tests/the_list_holds_everything_the_folder_holds.rs
  modified:
    - src/presentation/wx_app.rs
    - src/data/message_cache/tags.rs
    - src/data/message_cache/messages.rs
    - src/data/message_cache/mod.rs
    - src/application/asking_for_a_whole_folder.rs
    - tests/the_list_at_two_hundred_thousand_rows.rs
    - tests/a_whole_folder_moves_both_bounds.rs
    - tests/the_numbers_the_targets_ask_for.rs
    - guards/guards.toml
    - docs/development/measurements.md
    - docs/changelog.md
    - .planning/WINDOWS.md

key-decisions:
  - "The three label reads are named tags_by_message_in_folder, _in_account and _in_every_inbox rather than the plan's tags_for_folder and tags_for_account, because get_tags_for_account already answers the account's tag list and the two names would have sat a word apart meaning different things; a third sibling for every inbox because All Inboxes spans every account"
  - "get_tags_for_messages went with the change: it had no caller left outside its own two tests, which were rewritten in place onto the read by folder, keeping tags.rs at 10"
  - "The after rows are named after 10-02 in the harness, on 09-09's pattern, because the page refuses a row whose name and date repeat"
  - "The 14:47Z before rows stay on the page as taken, and the same-state re-take of the before commit at 16:00Z in a worktree is in the paragraph with its command, because the page holds a row once per name and date"
  - "The both-bounds record was rewritten and renamed rather than re-measured, as the plan's checker said, because its before block no longer existed and its red named a renamed test"

patterns-established:
  - "A step that refuses in a timing harness is a row whose value is refused, not a crash: the finding is the refusal"
  - "When a benchmark target gains a second ignored test, the command gains --test-threads=1 in the same commit, because the harness runs ignored tests at once in one process"

requirements-completed: []

coverage:
  - id: D1
    description: "The message list holds every message the folder holds on this computer: no page of 500, no limit that resets on a folder change, a message arriving adds a row and removes none; All Inboxes and the label view hold everything too"
    requirement: MAIL-02
    verification:
      - kind: integration
        ref: "tests/the_list_holds_everything_the_folder_holds.rs#test_the_window_asks_for_the_whole_folder"
        status: pass
      - kind: integration
        ref: "tests/the_list_holds_everything_the_folder_holds.rs#test_a_folder_of_the_testers_size_is_read_back_whole_through_the_query_the_window_uses"
        status: pass
      - kind: integration
        ref: "tests/a_whole_folder_moves_both_bounds.rs#test_a_chunk_arriving_rereads_the_folder_and_grows_no_limit"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#test_reread_folder_if_open_reads_the_whole_folder"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'the message list asks the cache for the whole folder and not a page of it', measured 2026-09-17: the one test named went red and nothing else did"
        status: pass
    human_judgment: false
  - id: D2
    description: "The list's own read path, the sorted query with no limit, the threading and the labels, measured at 12,872 and at 200,000 before the page came off and again after, with the rows on the measurements page with their commands"
    requirement: MAIL-02
    verification:
      - kind: integration
        ref: "tests/the_list_at_two_hundred_thousand_rows.rs#test_every_read_path_row_has_the_pages_shape_and_says_which_step_it_timed"
        status: pass
      - kind: command
        ref: "cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture --test-threads=1, at dbcddb93 and at 760a4d87, sixteen rows on docs/development/measurements.md dated 2026-09-17"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_every_row_on_the_measurements_page_carries_its_command_its_date_and_its_commit"
        status: pass
    human_judgment: false
  - id: D3
    description: "The labels read sends no parameter per row, so a folder above SQLite's variable limit reads its labels rather than failing, proved at 40,000 rows and measured at 200,000"
    requirement: MAIL-02
    verification:
      - kind: integration
        ref: "tests/the_list_holds_everything_the_folder_holds.rs#test_a_folder_above_the_variable_limit_reads_its_labels_by_folder_without_an_error"
        status: pass
      - kind: integration
        ref: "tests/the_list_holds_everything_the_folder_holds.rs#test_the_labels_of_a_folder_are_read_by_folder"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'the labels of a folder are read by folder, every one of them', measured 2026-09-17: all 2 tests named went red and nothing else did"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the tester's folder of 12,872 opens and reads as one list with his screen reader on his machine, and whether the list still answers keys at once after a folder change"
    requirement: MAIL-02
    verification: []
    human_judgment: true
    rationale: "MAIL-02's [S] line, ledger 515; the issue-close comment says it is his"

duration: 2h 2m
completed: 2026-09-17
status: complete
---

# Phase 10 Plan 02: The list holds everything the folder holds, measured before and after Summary

**The message list shows every message the folder holds on this computer, All Inboxes shows the
whole of every inbox, a label view shows every message carrying it, and a message arriving adds
a row and removes none: the page of 500 that reset on every folder change and grew only when
Get Older Messages or a whole-folder chunk asked is gone from the window (#24). The labels on
the rows come through one query by folder, by account or across every inbox, with no bound
parameter per row, where the old read sent one per row and the bundled SQLite refused it above
32,766 of them: run at 200,000 before anything changed, it refused with `variable number must
be between ?1 and ?32766` and the window showed the folder with no labels. The list's own read
path, the three steps `load_folder_messages` runs on the interface thread, is on the
measurements page at the tester's 12,872 rows and at 200,000, taken before the page came off
and again after: the sorted read and the threading cost what they cost, and the labels went
from a refusal to 103.45 ms at 200,000 and from 6.64 ms to 5.05 ms at 12,872, on a machine that
slowed by about a third between the two sets, which the page says how to read. Closed #24 from
the merge commit `48536d31` with what the harness proved and what only the tester's screen
reader settles.** Nothing pushed.

## Performance

- **Duration:** 2 h 2 min from the branch at 14:25:05Z to the merge at 16:27:26Z; of that,
  about 6 minutes on the release build of the before commit in a worktree and 8 minutes on its
  two runs, 7 minutes on the whole gate on the branch and 7 on `main`'s hook at the merge, and
  about 1 minute on the seven guard measurements by their `timed:` lines
- **Started:** 2026-09-17T14:25:05Z (the branch; first commit 14:29Z)
- **Merged:** 2026-09-17T16:27:26Z at `48536d31`
- **Tasks:** 3
- **Files modified:** 13 (1 created)

## The two measurements, with their commands

Both by `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored
--nocapture --test-threads=1`, output to a file, exit status read directly, `tasklist` clear of
`cargo.exe` and `rustc.exe` before each run, one test thread. The rows are on
`docs/development/measurements.md` verbatim, checked with `diff` against the run's output.

**Before, at `dbcddb93`, 14:47Z**, the tree still passing `Some(500)` in the window and the
harness passing `None` itself:

| Step | 12,872 rows | 200,000 rows |
|---|---|---|
| the sorted read, no limit, cold | 16.50 ms | 324.37 ms |
| the sorted read, no limit, warm | 15.26 ms | 368.90 ms |
| the threading | 5.63 ms | 208.79 ms |
| the labels, `get_tags_for_messages` over every id | 4.47 ms | refused |

The refusal, verbatim from the row: `Error: Failed to prepare statement: variable number must
be between ?1 and ?32766, followed by the statement's text, cut here`. The plan predicted "too
many SQL variables"; that is the text SQLite sends for an unnumbered `?`, and this query used
numbered `?N` placeholders, so the text is the one above. The first refusal row was 1.4 MB
long, because rusqlite writes the whole statement after the reason, which is why the harness
cuts it at the word that introduces it and says so.

**After, at `760a4d87`, 15:57Z**, the window passing `None` and the labels by folder:

| Step | 12,872 rows | 200,000 rows |
|---|---|---|
| the sorted read, no limit, cold | 23.89 ms | 464.94 ms |
| the sorted read, no limit, warm | 24.14 ms | 499.87 ms |
| the threading | 10.31 ms | 248.73 ms |
| the labels, `tags_by_message_in_folder` | 5.05 ms | 103.45 ms |

**The machine slowed between the two, and the page holds a control for it.** Every step that no
commit touched reads slower in the second set, and so does the `Listing` row the same runs
print: 256.71 ms cold at 14:47Z, 360.23 ms at 15:57Z, and three after runs between 15:35Z and
15:42Z agreed with each other. So the before commit was checked out into a worktree
(`../wixen-mail-before`, removed after), built in 6 m 19 s, and run twice, the second at 16:00Z,
three minutes after the after rows: at 200,000, 482.08 ms cold, 499.59 ms warm, the threading
244.84 ms, the labels refused; at 12,872, 24.19, 22.30, 7.42 and 6.64 ms. Held against those,
the sorted read and the threading cost what they cost before within the spread of three takes,
and the labels went from a refusal to 103.45 ms and from 6.64 ms to 5.05 ms. The worktree
figures are in the page's paragraph with their command and commit, not as rows, because the
page refuses a row whose name and date repeat and the 14:47Z rows keep the plain name. What
slowed the machine is not known: processor load 2 to 5 percent, the clock at its nominal 2000
MHz, nothing building, on mains power. The sum of the three after steps is what moving the read
off the interface thread would buy: 817 ms at 200,000 and 39 ms at 12,872, said on the page.

## What landed

**Task 1, the harness.** `tests/the_list_at_two_hundred_thousand_rows.rs` gained
`the_lists_own_read_path(count, into)`: a cache of `count` generated rows with a label on one
row in ten, reopened so the first read is cold; `get_message_list_sorted` with the default sort
of an inbox (`ORDER BY COALESCE(m.internaldate, m.date) DESC`, read from
`ColumnLayout::defaults_for(FolderKind::Inbox)`) and no limit, cold and then three takes warm;
`thread_messages` over every row read, with the inputs built inside the timing as the window's
private `apply_threading` builds them, and the conversations counted; then the labels read the
window makes, timed if it answers and recorded if it refuses. `Measured` carries an `Outcome`,
`Timed(takes)` or `Refused(reason)`, and a refused row's value is the word `refused` with the
reason in its conditions. A second ignored test takes the same steps at `THE_TESTERS_FOLDER`,
12,872. The command the rows carry gained `--test-threads=1`, because the first run of the two
ignored tests ran them at once in one process and the tester's-size series was timed while the
other test wrote 200,000 rows beside it; the header says so. Eight rows went on the page with
a paragraph saying why the read path is a different quantity from "Listing".

**Task 2, the window.** `load_folder_messages` lost its `limit` and passes `None`;
`WxUIState::message_list_limit`, `FOLDER_LIST_PAGE_SIZE` and `ALL_INBOXES_LIMIT` are gone, with
the reset on a folder change and the two arms that grew the page. `grep -c
'FOLDER_LIST_PAGE_SIZE\|ALL_INBOXES_LIMIT\|message_list_limit' src/presentation/wx_app.rs`
answers 0 with nothing excused: no comment quotes the old names. The fourteen other call sites
lost an argument, one at a time, four identical test-module blocks through one `replace_all` of
the exact seven-line call. The Get Older Messages arm no longer re-reads the folder before the
sync, because there is nothing behind a page to show; its comment says the key means "carry on
downloading, this folder first" and that 10-05 makes the next chunk the runner's job as well.
`MoreOfTheFolderArrived` re-reads the folder if it is open and moves nothing. `unified_inbox`
and `messages_with_label` take `Option<usize>` and the window passes `None`; `limit_clause` is
one function both listings share. The SCALE-03 comment on `spawn_whole_folder_fetch` and the
module doc of `application::asking_for_a_whole_folder` say the second bound is gone and when.

```
grep -n 'get_message_list_sorted' src/presentation/wx_app.rs
13756:    match cache.get_message_list_sorted(folder_id, &account_id, order.as_deref(), None) {
```

**Task 2, the labels.** `MessageCache::tags_by_message_in_folder(folder_id)`,
`tags_by_message_in_account(account_id)` and `tags_by_message_in_every_inbox()` share
`tags_by_message_where`, one query joined `message_tags` to `tags`, `messages` and `folders`
with a fixed `WHERE` and at most one bound parameter, grouped by message in Rust as before.
`attach_labels(items, read)` takes the read and logs a failure with `tracing::warn!` rather than
returning silently, so the list still arrives without its labels. Four callers: the folder read
by folder, All Inboxes across every inbox, the label view by account, and the saved-search
listing, a caller the plan did not name, by the folder the search names or by the account.
`get_tags_for_messages` had no caller left and went; its two unit tests were rewritten in place
onto the read by folder, `test_the_labels_of_a_folder_keep_each_messages_own_apart` and
`test_a_folder_nobody_has_labelled_answers_an_empty_map`, so `tags.rs` stays at 10.

**Task 2, the targets.** `tests/the_list_holds_everything_the_folder_holds.rs`, five tests: a
folder of 12,872 rows written through `upsert_messages` and read back whole through the
window's query with no limit; a label on one row in ten read through
`tags_by_message_in_folder`, 1,288 labelled rows, the first row's label named and the second
row absent; the same read over 40,000 rows, above the variable limit, answering 4,000 with no
error, 11.46 s in a debug build by the harness's own line; a reading of the shipping half of
`wx_app.rs` with comments cut, holding `load_folder_messages` to `None` and to
`.tags_by_message_in_folder(`, the two All Inboxes reads to `None`, and the file to naming none
of the three old identifiers; and a companion over a window small enough to plant in, which
requires the reading to name `Some(500)` and `message_list_limit` when they are put back and to
pass the clean text. In `tests/a_whole_folder_moves_both_bounds.rs`,
`test_the_whole_folder_request_moves_the_bound_on_what_the_list_shows` became
`test_a_chunk_arriving_rereads_the_folder_and_grows_no_limit`, holding the arm through
`what_is_wrong_with_the_arm` to calling `reread_folder_if_open(` and naming none of
`message_list_limit`, `FOLDER_LIST_PAGE_SIZE` and `+=`; `test_the_reading_would_see_a_limit_grown_again`
plants the old arm whole and requires three complaints, and an arm that forgot to re-read one
more. The module doc says the second bound is gone, dated. The four tests about the fetch bound,
the experimental sentence, the loop and the item cut are as they were for 10-05.

**Task 3, the documents.** The after rows beside the before rows, the paragraph above, and the
changelog entry's after sentence; ledger 515 both halves.

## Task commits

| Commit | What |
|---|---|
| `f8b7fc65` | test(10-02): the format test for the read-path rows, one named bare, against a stub answering no rows |
| `dbcddb93` | feat(10-02): the harness times the list's own read path at 12,872 and 200,000, the harness record corrected and measured |
| `cfe23210` | docs(10-02): the eight before rows and the paragraph |
| `56ef9699` | test(10-02): the new target, the both-bounds test rewritten, a stub `tags_for_folder`; four named bare with the two `house_style` checks |
| `f62c13f2` | feat(10-02): the page off, the labels by listing, three records measured, the changelog entry |
| `760a4d87` | test(10-02): the read-path rows named after 10-02 |
| `9a7b27e5` | docs(10-02): the eight after rows, the drift paragraph, the changelog's after sentence |
| `e104c466` | docs(10-02): ledger 515 and what moving the read off the thread would buy |
| `48536d31` | Merge 10-02 into `main` |

Branch `the-list-holds-everything-the-folder-holds` from `main` at `59c5b6a4`. Not pushed; 118
commits unpushed before the commit that lands this summary, by `git rev-list origin/main..HEAD
--count` on `main` at `48536d31`.

## Honest RED and GREEN

**Task 1's red** named one bare and the count check, because the harness gained a test and the
one record naming it held 6. It was red on the assertion, "0 rows printed and the page wants
one per step", against a stub `the_lists_own_read_path` answering no rows: a compile error is
not a red the gate can read, because `scripts/red-commit.sh` reads cargo's `FAILED` lines and a
target that does not build prints none. Two constants planned for the red moved to the green
because clippy refuses a dead constant. The green ran the ignored tests and found the two
running at once; the fix, `--test-threads=1`, is in the same commit. **One test in the green was
written beside its code and not before it:** `test_a_refusal_row_carries_the_reason_and_not_the_statement`,
for the cut of rusqlite's statement, whose need the first run found with a 1.4 MB row. Said in
the commit and here.

**Task 2's red** named four bare and the two `house_style` checks: the labels test and the
40,000-row test, red against a stub `tags_for_folder` answering no labels (0 against 1,288 and
0 against 4,000); the window reading, red on six counts quoted by the run (the three names
still present, `Some(limit)`, no read by folder, `ALL_INBOXES_LIMIT` in `unified_inbox`); and
the both-bounds test, red because the arm still grew a limit. Green on arrival, as designed:
`test_a_folder_of_the_testers_size_is_read_back_whole_through_the_query_the_window_uses`,
because `get_message_list_sorted` has taken `None` since it was written and only the window
passed a number; and the two companions, which plant into snippets. **The companion found a
bug in the reading before the reading was trusted:** `messages_with_label(` is also the tail of
`fn load_messages_with_label(`, so the reading took the definition's parameters for the call's
arguments; the calls are anchored on their dot. The plan said the cache tests are red "because
`tags_for_folder` does not exist"; it has to exist as a stub for the red to build, which is
10-01's pattern and the commit says so.

The green in task 2 was exactly green on its first run through the hook: 4 m 31 s, twelve
coupled targets, the count check silent. `cargo test --lib presentation::wx_app::` answers 199
passed before and after, quoted from the run.

## What the gate selected

| File | On the branch |
|---|---|
| `tests/the_list_at_two_hundred_thousand_rows.rs` | its own target on every commit that changed it |
| `src/data/message_cache/tags.rs`, `messages.rs`, `mod.rs` | `--lib data::message_cache::tags::` (10), `::messages::` (179), `data::message_cache` |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app::` (199) and twelve coupled targets, the new one and `a_whole_folder_moves_both_bounds` among them |
| `src/application/asking_for_a_whole_folder.rs` | `--lib application::asking_for_a_whole_folder::` (7) |
| `tests/the_list_holds_everything_the_folder_holds.rs`, `a_whole_folder_moves_both_bounds.rs`, `the_numbers_the_targets_ask_for.rs` | their own targets as changed, and coupled on task 2's green |
| `guards/guards.toml`, `docs/*.md`, `.planning/WINDOWS.md` | no scoped target; the whole-tree guards on every commit, the document-reading targets on the three `docs_only` commits |

Every commit ran formatting, clippy, the four script suites and the whole-tree guards.
`scripts/check.sh all` on the branch at `e104c466`, output to a file and the exit status read
directly, never piped: exit 0, 7,932 passed and none failed over 70 result lines, 424 s from
16:12:43Z to 16:19:47Z, the release build included. Eight more than 10-01.1's 7,924: two in the
harness, five in the new target, one in the both-bounds target. Inside the 275 s to 654 s band
on `docs/development/measurements.md`. `main`'s hook ran `all` again on the merge: 7,932 and
none failed, from 16:20:28Z to 16:27:26Z. The keyring race (ledger 374) did not appear on
either. `tests/the_settings_tab_row_says_each_tab_once.rs`, which 10-01.1 met failing in one
four-minute window, passed on every run here.

## Guard records

879 records by the TOML reader before, 881 after; census 802 + 77 before, 802 + 79 after, the
line at `guards/guards.toml:84` moved in task 2's green. Two new with `suite` naming the new
target, one rewritten and renamed, one corrected, all measured through `scripts/guards.sh
--remeasure` with `WIXEN_TEST_THREADS` untouched, the counts written by the runner.

| Record | File and suite | Break | Red | Run |
|---|---|---|---|---|
| the rows the scale harness prints keep the page's six columns (corrected) | `the_list_at_two_hundred_thousand_rows.rs`, itself | as before, five cells | 2, was 1 | rebuild 4 s, run 1 s |
| a chunk of a whole-folder request that lands is shown, and no limit grows (rewritten from "a whole-folder request moves both of the bounds and not one of them") | `wx_app.rs`, `a_whole_folder_moves_both_bounds` | the re-read taken out of the `MoreOfTheFolderArrived` arm | 1 | rebuild 12 s, run 1 s |
| the message list asks the cache for the whole folder and not a page of it | `wx_app.rs`, `the_list_holds_everything_the_folder_holds` | `Some(500)` put back into the call | 1 | rebuild 12 s, run 10 s |
| the labels of a folder are read by folder, every one of them | `tags.rs`, `the_list_holds_everything_the_folder_holds` | `LIMIT 100` on the shared query | 2 | rebuild 12 s, run 9 s |

**The count check fired once, on task 1's green, and its remedy came out short.** The harness
record held 6 and found 9, and the five-cell break reddens the new format test too, which walks
the same `the_row`; the runner said "1 test went red that this record does not name". The test
was added to the record by hand with the reason dated, and the run after that reddened exactly
the two. The both-bounds record was rewritten, not re-measured, as the plan's checker said:
its `before` was the block that grew the limit, which no longer exists, and its `red` named a
renamed test; the new break is unique in the file, checked with `grep -c`, and its count for
the target says 8 now because the target gained a companion (the plan said to rewrite the
existing companion at `:172`; that one proves the item cut for the fetch test that stays for
10-05, so a new companion was added beside it and the remedy paid, 13 s). Counts written:
`wx_app.rs` 199 on both records naming it here (58 name it now, was 57), `tags.rs` 10 (1 names
it now, was 0), the new target 5, the harness 9, the both-bounds target 8. The count check is
green on `main` at `48536d31`.

## Premises the tree contradicted

Every command in the plan's seven premises was re-run against `main` at `59c5b6a4` before
anything was built. All seven held: the two constants at `:7050` and `:7061`, the field at
`:324` with its ten sites, sixteen `load_folder_messages(`, `get_message_list_sorted` taking
`Option<usize>` at `:2159`, the two All Inboxes reads taking `usize`, the harness naming
`get_messages_for_folder` at `:214`, `:235` and `:515`, `get_tags_for_messages` at `tags.rs:229`
with its placeholders, `libsqlite3-sys 0.38.1`, the both-bounds tests at `:103` and `:172`, and
the gate mapping. Four things moved from the plan:

1. **The refusal's text.** The plan predicted "too many SQL variables"; the row says `variable
   number must be between ?1 and ?32766`, because the query numbered its placeholders. The
   prediction that it refuses held; the words did not.
2. **The count of labelled rows at 12,872 is 1,288, not 1,287.** One row in ten counted from
   the first row is `ceil(12872 / 10)`; the plan's figure was the floor. The test asserts 1,288
   and says which.
3. **"No longer says Getting older messages... twice."** `grep -n 'Getting older messages'
   src/presentation/wx_app.rs` finds one `send_status`, at `:4606` before the change; nothing
   said it twice. What the arm did do twice was read: `load_folder_messages` before the sync and
   `reread_folder_if_open` when the sync's chunk arrived. The read before the sync is gone.
4. **A fourth caller of `attach_labels`**, the saved-search listing at `:7371`, which the plan's
   three did not include. It asks by the folder the search names or by the account.

And one thing the plan could not know: the machine slowed by about a third between the before
and after runs, on steps no commit touched, which is why the before commit was run again in a
worktree beside the after rows.

## Deviations from plan

**1. [Decision] The names.** `tags_by_message_in_folder`, `tags_by_message_in_account` and
`tags_by_message_in_every_inbox` for the plan's `tags_for_folder` and `tags_for_account`, above.
The reading holds `.tags_by_message_in_folder(`.

**2. [Rule 2] A third sibling and a fourth caller.** All Inboxes spans every account, so
`tags_by_message_in_account` cannot serve it; `tags_by_message_in_every_inbox` lists by the
`WHERE` `unified_inbox` lists by. The saved-search listing asks by folder or by account.

**3. [Decision] `get_tags_for_messages` removed** with its two unit tests rewritten in place, above.
`tags.rs` is named by no record before this plan, so the rewrite cost nothing at the gate.

**4. [Rule 1 - Bug] The reading's anchor.** Found by the companion, above.

**5. [Rule 1 - Bug] The harness's ignored tests ran at once.** `--test-threads=1` in the command
the rows carry, above. Observation 644 in the skill log.

**6. [Rule 3] rusqlite's statement in the error**, cut by the harness, with the one test written
beside its code, above.

**7. [Decision] The harness's labels step moved to the read by folder in task 2's green**, a
file in task 1's list and not task 2's, because the after rows had to time what the window
runs. And the rows are named "after 10-02" in a commit of their own before the after run, so
the commit column names a tree that prints those names.

**8. [Decision] The changelog entry landed with the change, in task 2's green**, with the before
figures, and task 3 added the after sentence; `CLAUDE.md` wants the entry in the commit that
makes the change and the plan put it in task 3, and both are satisfied this way.

**9. [Process] The before commit built and run in a worktree** for the same-state pair, and the
worktree removed after; two worktrees that were there before (`wixen-mail-mutants`,
`wixen-mail-sweep`) are untouched.

**10. [Decision] The page's paragraph carries the same-state re-take rather than rows**, because
the page refuses a row whose name and date repeat; the paragraph carries the command, the
commit and the date, which `every_figure_on_a_page_carries_its_date_and_its_source` reads.

**11. [Rule 1 - Docs] `asking_for_a_whole_folder.rs`'s module doc** named the second bound as a
fact and now says it is gone; `the_numbers_the_targets_ask_for.rs` passed `A_THOUSAND` to
`unified_inbox` and passes `None`, the shape the window asks for.

Everything else executed as written. **No scripted edit touched a tracked file: the exception
set for this plan is zero, and it stayed there.** Every tracked file was changed by Read then
Edit or Write, including the four identical test-module call sites, which went through one Edit
with `replace_all` over the exact seven-line call, a shape no comment holds; the only `sed`,
`awk`, `grep` and `cut` in the session read files and logs, and the one Python one-liner read
`guards.toml` and counted. Commit messages were written to the scratchpad and passed with `-F`;
`cargo fmt` ran before each commit. Carriage returns measured with `tr -cd '\r' | wc -c` on
every changed file before each commit: zero on each. No em-dash in any file this plan wrote,
measured with `grep -c` for the byte sequence: zero on each; none of the six words. `git commit`
and `git merge`, not `gsd-tools query commit`; never `--no-verify`; `check.sh` never piped. No AI
attribution in any commit. `Cargo.toml` and `Cargo.lock` untouched; no crate, no feature. The
version stays `1.0.0-alpha.1`. The tester's profile was not read; the number 12,872 is quoted
from the README's reading of 2026-09-16; no binary was started.

## Threat register

T-10-05: accepted and measured, the first open of 200,000 on the interface thread under a second
by the after rows, said in the changelog's Known limitations and in ledger 515, with what
moving the read off the thread would buy on the page. T-10-06: mitigated, the three reads with
no parameter per row, the test at 40,000 and the row at 200,000, the record measured. T-10-07:
mitigated, the reading holds the shipping half to naming none of the three identifiers and the
changelog dates the change; no comment quotes the old names. T-10-SC: nothing added. No new
surface outside the register: the queries take fixed `WHERE` clauses and at most one bound
parameter, and nothing a user typed reaches them.

## Ledger

`.planning/WINDOWS.md` 515 written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after, 16 passed. 514 before, 515 after; 486
open before, 487 after.

| id | kind | what |
|---|---|---|
| 515 | unrun-verify | what only the tester settles for #24: whether 12,872 reads as one list with his screen reader and still answers keys at once after a folder change; that nobody has opened a folder of 200,000 in the running program; the first open of one under a second on the interface thread |

## The issue

`gh issue close 24` from the repository root after the merge, quoting `48536d31`, closed at
16:27:46Z with the comment the plan drafted: the list holds everything the folder holds here and
so does All Inboxes, a message arriving adds a row and removes none, measured before and after
at his size and at 200,000 with the rows dated on the page, the labels by folder because the old
read failed above 32,766, and whether it reads as one list with his screen reader is his.
Closing an issue is not a publish; no other issue was filed or edited.

## Known stubs

None. `load_folder_messages` is reached by the folder change, the refresh, the outbox handlers,
the sync's arrivals and the tests; the three label reads by their four callers; the harness by
its command; the new target by the gate through its two records' `suite`. No control, sentence
or setting was added.

## For 10-05

- Get Older Messages means "carry on downloading, this folder first": the arm sends the status
  line and `spawn_mail_sync(app, Some(path), MoreOfWhatIsAlreadyThere)` and reads nothing
  first; there is no limit for the runner to grow, and a chunk that lands is shown by
  `reread_folder_if_open` from `MoreOfTheFolderArrived` and `FolderMessagesArrived`.
- `tests/a_whole_folder_moves_both_bounds.rs` still holds the fetch bound
  (`INITIAL_FETCH_LIMIT` in `spawn_whole_folder_fetch`), the experimental sentence, the loop and
  the item cut; retiring the command means rewriting those four and the record named "a chunk
  of a whole-folder request that lands is shown, and no limit grows", whose break is the re-read
  in the `MoreOfTheFolderArrived` arm.
- The list at the tester's size costs the interface thread about 40 ms on open by the after
  rows; at 200,000 about 800 ms, and the window answers no keys for that long. A runner that
  sends `FolderMessagesArrived` per chunk re-reads the open folder per chunk, at that cost per
  chunk on a folder that size.

## Self-Check: PASSED

`tests/the_list_holds_everything_the_folder_holds.rs` exists; `grep -c
'FOLDER_LIST_PAGE_SIZE\|ALL_INBOXES_LIMIT\|message_list_limit' src/presentation/wx_app.rs`
answers 0; `grep -n 'get_message_list_sorted' src/presentation/wx_app.rs` shows one call with
`None`; `guards/guards.toml` holds 881 records by the TOML reader and the census line says 79;
`docs/development/measurements.md` holds eight rows beginning "| The list's own read path, " and
eight beginning "| The list's own read path after 10-02, "; `docs/changelog.md` holds the line
beginning "**The message list holds every message the folder holds on this computer"; `.planning/WINDOWS.md`
holds 515 in both halves. Commits `f8b7fc65`, `dbcddb93`, `cfe23210`, `56ef9699`, `f62c13f2`,
`760a4d87`, `9a7b27e5`, `e104c466` and `48536d31` are in `git log --oneline` on `main`. Issue
#24 reads CLOSED by `gh issue view 24 --json state`.

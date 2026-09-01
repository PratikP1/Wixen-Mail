# Deferred items found during phase 2

Things found while executing this phase that are real, are not caused by the
work in hand, and were deliberately not fixed here. Each one carries the
measurement that found it, so acting on it does not mean measuring it again.

## A guard record that was already short of the truth: "a WebDAV read cannot carry a changing verb"

**Found during:** plan 02-01, task 3, running `scripts/guards.sh` over the
records this plan bears on.

**What is wrong:** the record at `guards/guards.toml` named
`a WebDAV read cannot carry a changing verb` names one test. Its break reddens
two. Measured 2026-08-31 against the whole library:

```
-- a WebDAV read cannot carry a changing verb
   1 test went red that this record does not name:
       service::outward::a_question_is_not_a_change::test_every_way_of_asking_is_a_verb_that_changes_nothing
```

**Why it is not fixed here:** it is not this branch's doing, and it was checked
rather than assumed. `git diff 8836efc -- src/service/outward.rs` shows plan
02-01 touched neither `mod a_question_is_not_a_change`, nor
`test_every_way_of_asking_is_a_verb_that_changes_nothing`, nor `AskWith::EVERY`
which that test walks. The record was already understating its coverage before
this branch existed. Folding an unrelated correction into a commit about the
read dimension would make the commit say two things.

**What acting on it costs:** one line. Add

```
    "service::outward::a_question_is_not_a_change::test_every_way_of_asking_is_a_verb_that_changes_nothing",
```

to that record's `red` list, in the sorted position, and re-run
`bash scripts/guards.sh "WebDAV read"` to confirm. That single record takes
about three and a half minutes.

## The unfiltered guard sweep is now a job measured in hours, not an hour or two

**Found during:** plan 02-01, task 3.

**What is wrong:** `CLAUDE.md` says a full `scripts/guards.sh` run is "an hour
or two". Measured on 2026-08-31: `guards/guards.toml` holds 536 records, each
one a rebuild plus a whole `cargo test --lib`, and the library suite alone
takes 196 to 208 seconds on this machine. Timed against the run itself, the
first thirteen records had not finished after twenty-four minutes, which puts
the whole sweep between sixteen and thirty-two hours.

This is not a complaint about the sweep, which earned its keep in this very
plan: the subset that was run found five stale records, four of them made stale
by this plan's own new tests, and none of them would have been found by running
the tests that were changed.

**Why it is not fixed here:** it is a documentation figure and a workflow
question, not code, and the answer is a decision rather than an edit. Some
possibilities, none of them chosen:

- Correct the figure in `CLAUDE.md` so nobody plans a session around it.
- Give the sweep a resumable mode, so an interrupted run does not start again
  from record one. It currently has no progress file, and killing it skips the
  `finally` that puts the broken file back, which happened here: the run was
  stopped and left `src/application/contacts_sync.rs` broken in the working
  tree, restored by hand with `git checkout --`.
- Scope a routine run to the records whose `file` a change touched, and keep
  the whole sweep for a scheduled overnight job. The filter today matches on a
  record's *name*, so selecting by file is not something the script can do yet.

**What was done instead in 02-01:** fourteen records were selected by hand and
run, chosen as the three new ones, the two other records on `allowed.rs`, and
every record about the write census, the imap gate or the POP gate. That took
about fifty minutes.

## From 02-04

- **`tests/manager_dialog_labels.rs` does not run on the commits that could
  break it.** `scripts/check.sh` maps a changed `src/a/b.rs` to
  `cargo test --lib a::b::` and always adds `house_style` and `wired`. A guard
  living in `tests/` that covers a `src/` module is reached only when the test
  file itself changes. `tests/checkbox_labels.rs` has the same gap for
  `wx_item_form.rs`, and it is excluded from the per-commit code path
  deliberately, so this was recorded rather than reversed. The guard record in
  `guards/guards.toml` does run it, through `scripts/guards.sh`.

- **A rule naming a field this build has never heard of still loses its field
  on the way through the dialog.** Opening it selects nothing, and pressing OK
  stores the empty string. Before this plan that was true for five of the
  eleven real fields; now it is only reachable by a rule written by a later
  version. The fix is a refusal or a passthrough rather than a silent
  rewrite, and it is a decision about what a dialog owes a value it cannot
  show.

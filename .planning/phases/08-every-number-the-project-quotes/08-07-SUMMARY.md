---
phase: 08-every-number-the-project-quotes
plan: 07
status: complete
subsystem: testing
tags: [guards, sweep, resume, measurements, tooling]

requires:
  - phase: 08-every-number-the-project-quotes
    plan: 01
    provides: "the `timed:` line the runner prints per record, and the rate and product rows on `docs/development/measurements.md` the checkpoint quotes"
  - phase: 08-every-number-the-project-quotes
    plan: 06
    provides: "the sweep's cost stated once as a rate times a count, so the checkpoint quotes a row rather than a figure"
provides:
  - "`scripts/guards.py` with `--log`, `--resume`, `--stop-after`, `--wait-until-quiet` and `--shard K/N`; `verdicts_in`, `foreign_builds_in`, `is_quiet`, `why_a_resume_is_refused`, `the_resume_command`, `the_closing_line`, `the_shard_asked_for`, `the_records_in_shard` and `the_header_for` as pure parts with worked examples the gate runs"
  - "`.github/workflows/guards.yml`, the sweep dispatched by hand onto GitHub's Windows runners in shards, each shard's log kept; `tests/the_guard_sweep_runs_on_runners.rs` holding the workflow to the script, in `check.sh`'s whole-tree list, with its record"
  - "A worktree at `../wixen-mail-sweep`, built once, to be moved to the dispatched commit when the run completes; the local start kept as the fallback"
  - "The checkpoint below, usable without the plan, with the exact download-and-merge lines"
affects: [08-07 task 3, 08-08, 08-09]

actuals:
  tokens: 35858
  tasks: 3
  commits: 13

tech-stack:
  added: []
  patterns:
    - "A long run's log is its own resume state: the runner prints a marker line before each unit and a verdict after, and the resume is a reading of exactly those lines, held by the examples of the function that prints them"
    - "A tool that breaks the tree and restores in a `finally` gets a refusal at the next start, read from `git status` over the files it breaks, because a process-tree kill does not reach a `finally`"

key-files:
  created:
    - .github/workflows/guards.yml
    - tests/the_guard_sweep_runs_on_runners.rs
  modified:
    - scripts/guards.py
    - scripts/guards.sh
    - scripts/check.sh
    - guards/guards.toml
    - docs/changelog.md
    - .planning/STATE.md
    - .planning/ROADMAP.md
    - .planning/WINDOWS.md

key-decisions:
  - "`--resume` is refused without `--log`, because a run that records its verdicts nowhere cannot itself be resumed; the plan only said a bare `--resume` takes the `--log` path"
  - "`--stop-after` cuts the selection before the pre-read, so a one-record chunk reads one suite rather than the 23 integration targets the file names; seen in the runaway run's output"
  - "`could not be measured` is a verdict and a resume skips it, because measuring a moved break again finds the same thing and the correction is by hand in task 3"
  - "The log is read before it is opened for appending, because the open creates it and the first live probe of a resume from a missing log started the whole sweep"
  - "`own_pids` is passed empty: the runner polls only between records, when `subprocess.run` has returned and no child of its own is alive, and the parameter stays so `is_quiet` states what quiet means"
  - "The worktree was created and built by the executor, because a checkpoint instruction nobody has run is the check nobody reads; the sweep itself was not started"
  - "The runner reading lives in a target of its own, `tests/the_guard_sweep_runs_on_runners.rs`, in `check.sh`'s whole-tree list, rather than in `house_style.rs`, which twenty-five records name as their suite and every added test flags for re-measurement"
  - "`WIXEN_TEST_THREADS` stays at 8 on the 4-core runners, unmeasured, so the runner's timing lines read against the page's rate row at one setting; ledger 465"
  - "The workflow caches the cargo registry and not `target/`, because twenty jobs saving eight gigabytes each into a ten-gigabyte cache would evict each other"

patterns-established:
  - "A refusal in an impure shell is run once on the input it refuses before the work is called done; the pure examples cannot see where in the sequence it sits"

requirements-completed: []

coverage:
  - id: D1
    description: "The runner appends its output to a log, resumes from that log measuring only what has no verdict, stops after a known number of records, and waits for a quiet machine"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/house_style.rs#test_the_guard_runner_still_obeys_its_own_examples, 95 examples in 54 items"
        status: pass
      - kind: other
        ref: "bash scripts/guards.sh --log target/one-record.log --stop-after 1 --wait-until-quiet, exit 0, one verdict and its timing line, then --resume --stop-after 1 skipping it and measuring the next"
        status: pass
    human_judgment: false
  - id: D2
    description: "A resume refuses over a modified guarded file and prints the checkout; a contended record is re-measured on resume; a detached start finishes with its window gone"
    requirement: PERF-06
    verification:
      - kind: other
        ref: "the three logs quoted under Proof by running below"
        status: pass
    human_judgment: false
  - id: D3
    description: "The sweep dispatched onto GitHub's runners in shards, each shard's log kept and the logs read back as one"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/the_guard_sweep_runs_on_runners.rs#test_the_guard_sweep_dispatch_hands_the_script_flags_it_accepts, with its companion and its record"
        status: pass
      - kind: other
        ref: "python scripts/guards.py --shard 41/41, --shard 3, --shard 0/1000, --shard 3/41 --stop-after 0, --shard 3/41 --recount-everything, each refused or reported as the examples say"
        status: pass
    human_judgment: false
  - id: D4
    description: "The one guard sweep of the milestone, every record on one commit, and every record it found short corrected by hand and measured again"
    requirement: PERF-06
    verification:
      - kind: other
        ref: "run 34965790937 at df3437a1; bash scripts/guards.sh --log sweep.log --resume --stop-after 0 in the worktree printing 'Every record selected has a verdict: 803 of 803'; scripts/guards.sh --remeasure over the 28 corrected records, 27 agreed in one run and the renamed one on its own"
        status: pass
      - kind: integration
        ref: "tests/house_style.rs#test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it, #test_every_guard_record_says_how_many_tests_the_files_it_names_held"
        status: pass
    human_judgment: false

duration: 1h20min for task 1, 40min for the answer, 2h50min for task 3
completed: 2026-09-15
---

# Phase 8 Plan 07: A sweep that can be stopped and picked up Summary

**The one guard sweep of the milestone is done: every one of the 803 records `guards/guards.toml` held at `df3437a1` has a verdict, taken on GitHub's Windows runners in 41 shards in 4 h 7 min of wall clock, and the 31 records it found not to be what they say are corrected by hand and measured again. 772 agreed. Of the 31, 24 named too few because a test written after the record reaches the same rule, 21 of those in a file the record had never named; 2 named a test that had stopped reaching the break; 1 had its break on the wrong line; 1 named a rule no code now holds and is renamed to what its break guards; 1 reddened nothing and is retired; 2 are right on a machine and blind on a runner and are left as they are. On the way there the runner learned to be stopped and picked up from its own log, to wait for a quiet machine, and to take one shard of the file, and the sweep moved from twenty hours on Pratik's machine to four on the runners after he widened the checkpoint: "Go for running the guard sweep via CI as well."**

Task 1, the checkpoint widened and answered, the sweep dispatched by Pratik, and task 3. Criterion 5 closes on the count of records the log holds a verdict for.

## Task 3: the log read after it was complete, and every short record corrected

**The run.** `gh run view 34965790937 --json headSha` answers `df3437a1077bb3c49675d742efe326ff28285f14`, `main`'s head when Pratik dispatched "Would each guard still go red" on 2026-09-15 with `shards=41 first=0 last=40`. 42 jobs: the generator and 41 shards, 26 green and 15 red, a red shard being one that exited 1 because a record was not what it says. The first shard started 11:56:37Z and the last finished 16:04:09Z, 4 h 7 min 32 s of wall clock in two waves of twenty; the 41 jobs' own timestamps sum to 64.7 hours of runner time. Every shard's log was uploaded and every one ends with its `Every record selected has a verdict` line, checked one by one, so no shard was killed at the six-hour cap and no second dispatch was needed.

**The merge.** The sweep worktree was moved to the run's commit, `git -C ../wixen-mail-sweep checkout --detach df3437a1`; `gh run download 34965790937 --dir target/runner-sweep` inside it laid out 41 directories; the 41 logs concatenated in shard order into `sweep.log`, 3,066 lines, 3,066 carriage returns, one per line, as the runner's Python writes them; 803 `-- ` blocks, no `contended:` line, no `could not be measured`. The read-back, the plan's precondition for this task:

```
Resuming from sweep.log: 803 of 803 already measured, 0 to go

31 guards are not what the record says they are:
    [the 31 names]

31 of those 31 verdicts came from the log.
[...]
Every record selected has a verdict: 803 of 803, 803 from the log and 0 from this run.
```

`git status --porcelain` in the worktree named only `?? sweep.log`; the download sits under `target/`, which is ignored. The merged log is the plan's artifact at `.planning/phases/08-every-number-the-project-quotes/sweep.log`, re-made from the 41 shard logs alone after the first copy was found to carry the read-back's own 44 lines, which `--log sweep.log` had appended to the worktree's file: 3,066 lines and 3,066 carriage returns by `tr -cd '\r' | wc -c` in the working copy. Not kept that way in git: `.gitattributes` sets `text=auto` and `eol=lf` for the tree, so the commit normalised it and `git show HEAD:<path> | tr -cd '\r' | wc -c` answers 0, which git said at the commit as "CRLF will be replaced by LF the next time Git touches it". The premise that the log would hold carriage returns was true of the file the runner wrote and of the file on disk, and false of the blob; said here rather than converted by hand either way. The phase directory's Markdown checks walk `.planning` for `.md` and do not read it.

**Every one of the 31 was measured again on this machine before any record was edited**, through `scripts/guards.sh --remeasure` with the 31 names, logged: 29 came out exactly as the runner found them, and 2 agreed here. The first attempt at that run passed the names with their carriage returns from the file they were read out of, and the runner refused all 31 as "No record is named exactly"; stripped with `tr -d '\r'`, the run took about 50 minutes. Because a runner is not this machine, a record that stayed green there was not corrected on the runner's word alone.

**The runner's rate**, off the 803 `timed:` lines that follow a `-- ` line in the log: median 260 s a record, rebuild 144 s and run 111 s; mean 234 s, lower because records naming an integration target rebuild and run in seconds. This machine's rate row says 92 s. The first shard to finish, 8, took 72 minutes for 20 records with 17 of them its first build, which is the four minutes a record the coordinator read off it.

### The findings, by cause

**A test written after the record reaches the same rule, in a file the record had never named: 21 records.** This is the limit `CLAUDE.md` names as the one the count check cannot see, and it is most of what the sweep found. Each record now names the tests, with the date they were written and the note that the file was one it had never named.

| Record | Written | Tests added, where and when |
|---|---|---|
| a count and the thing it counts agree in number | 2026-08-07 | 2, in `notes_sync.rs` (2026-09-10) and `note_folder_tree.rs` (2026-09-11) |
| one change waiting is not read out in the plural | 2026-08-07 | 1, `tasks_sync.rs` (2026-09-09) |
| an edit written here and then lost is still counted and said | 2026-08-07 | 1, `carddav_sync.rs` (2026-09-10) |
| a deletion made here leaves a note for the address books that knew her | 2026-08-08 | 1, `carddav_sync.rs` (2026-09-10) |
| the read asks who was deleted, not only what is still owed | 2026-08-09 | 1, `carddav_sync.rs` (2026-09-10) |
| an address book that took a deletion is remembered, not dropped | 2026-08-09 | 1, `carddav_sync.rs` (2026-09-10) |
| only a deletion a provider took is let go of by the clock | 2026-08-09 | 6, `carddav_sync.rs` and `notes_sync.rs` (2026-09-10 and 11) |
| a moment written with a T is one the reader knows | 2026-08-09 | 2, `wx_reminder_alert.rs` (2026-09-14) |
| a flag the mail server refused is not reported as set | 2026-08-11 | 4, `flag_changes_waiting.rs` (2026-09-05) and `mail_across_accounts.rs` (2026-09-08) |
| a connection that closed mid answer is not an answer that finished | 2026-08-11 | 3, `mail_session.rs` (2026-09-04) |
| a message the server hands over is not reported as one that is not there | 2026-08-11 | 19, `mail_across_accounts.rs` (2026-09-07 and 08); a record naming two said nothing about nineteen for a week |
| a task or event body written as html is read as the structure it carries | 2026-08-13 | 3, `onenote_notes.rs` (2026-09-11) |
| an exact language match has to be available, not merely spelled the same | 2026-08-22 | 1, `data/config.rs` (2026-09-03) |
| the exact language match has to spell the tag the same, not differently | 2026-08-22 | the same test |
| a family language match has to be available, not merely share the family | 2026-08-22 | the same test |
| a family language match has to share the family, not differ from it | 2026-08-22 | the same test; a fifth spellcheck record was found short by this very test on 2026-09-13, the finding written on it alone, and the four beside it with the same break site were not measured again |
| the form a signed message arrived in is really kept | 2026-08-28 | 1, `how_it_arrived.rs` (2026-09-06) |
| a folder with folders inside it is deleted deepest first | 2026-08-30 | 1, `emptying.rs`, the same day |
| a change to a page is built from the read immediately before it | 2026-09-11 | 1, `onenote_notes.rs`, the same day |
| a build identifier is not part of the version | 2026-09-12 | 1, `update_check.rs`, the same day |
| an event cannot be done | 2026-09-14 | 2, `wx_reminder_alert.rs`, the same day, by 06-09's task 2 after task 1 wrote the record |

**A test written after the record, in a file the record names, stamped over by the recount: 3 records.** The tests arrived between the record and 2026-09-02, when `--recount-everything` wrote every record's counts without measuring, so the count check had nothing to compare against.

| Record | Written | Test added |
|---|---|---|
| a change in a calendar no account holds is said | 2026-08-08 | `test_two_calendars_that_share_a_reason_are_still_said_separately`, `calendar.rs`, 2026-08-19 |
| a name the timezone database does not know writes no rules | 2026-08-09 | 10 tests, 2026-08-11 to 13, one in `caldav_sync.rs` which it names and nine in `calendar.rs` and `managers.rs` which it did not |
| a zone that follows no yearly rule lists its days rather than guessing | 2026-08-09 | `test_listed_blocks_keeps_each_pair_of_offsets_in_its_own_block`, `vtimezone.rs`, 2026-08-19 |

**A named test that no longer reaches the break: 2 records.** The name is taken off with the reason on the record.

- **folding two address books counts one person once**: `test_one_edit_to_one_contact_both_books_hold_is_said_once_and_not_twice` stayed green, 8 of 9 red. `ce3e89ba` of 2026-09-05 rewrote its scenario so the edit is held for somebody to choose and the second address book is offered nothing, so the fold this break doubles is not on its path. Eight remain.
- **a settings file written before reading was a setting still loads**: `test_a_command_that_names_a_different_folder_opens_that_one_first` stayed green, 1 of 2. It is the test `scripts/guards.py` records as failing only in company; it was red in company when the record was written on 2026-08-31 and is not about a settings file. The one test that is remains.

**A break on the wrong line: 1 record.** **a note's body comes back from a document the bytes it went out as** broke the title's reading while its two tests are about the body, and no fixture's title ends in a space, so the break was one nothing could see; its own comment says it was measured red on 2026-09-10, and how is not reconstructed. The break is now the body's line, and both tests go red. The title's reading is guarded by nothing, said on the record.

**A rule no code now holds: 1 record renamed.** **a changed day of a CalDAV series is marked seen before the removal pass can run** took out the `seen_uids` insert at the top of `one_caldav_day_kept_out_of_its_series` and its one test stayed green. The break was moved to the other marking, where the loop over the server's answer marks whatever identity a row was stored under, on the reading that the rule had moved there; measured, that break reddens exactly one test, `test_an_event_stored_under_half_an_identifier_keeps_what_was_typed_on_it`, and still not the moved-day test. So neither marking is what keeps a moved day now, and what does was not reconstructed. The record is renamed to the fact its break guards, "a row stored under the identity the server named is not taken for one it dropped", measured on its own and agreed; the unguarded rule is ledger 471 and the redundant insert ledger 469.

**A break that reddens nothing: 1 record retired.** **the other setting is named only while the change is somebody's work** took the `the_copy_here_was_written_here` gate off a `note` call, and its one test stayed green on the runners and here, with nothing else red. The same `ce3e89ba` moved that test's scenario off the gate. A record naming no test is one the runner refuses, so it is retired with a comment where it stood, and the gate, which the tree still holds and nothing tests, is ledger 468; the test that would notice belongs in `contacts_sync.rs`, which 77 records name.

**Right here, blind on a runner: 2 records left as they are.** **an hour with no zone means an hour here, to Outlook as well as to Google**: both named tests red here, both green on the runner, whose clock is UTC, so a break that sends the local hour as UTC changes nothing there. **the walk into Windows own chain structures really happens**: red here, green on a runner with no certificate chain to walk. Ledger 470 says a runner sweep will report both this way again.

**Re-measured after correction.** The 28 corrected records through one `--remeasure` run: 27 agreed and had their counts written, the renamed CalDAV record disagreed with its first rewrite, was rewritten to what the run found, and agreed on its own run. The four record checks pass: the census header, the count check (which had named 23 records whose new tests lived in files never counted, until the counts were written), the exists check and the one-place check.

**The census header** now opens with the sweep of 2026-09-15: 802 swept that day and 3 arrived since, 805 records; 802 rather than 803 because one of the 803 was retired the same day, and the 3 arrived with the theme_reach fix on 2026-09-15, each measured when written. The 2026-08-12 section is kept below it as the record of that sweep, its two count lines reworded so the reading finds the new pair first.

**Commits, all through the hook, on branch `every-record-the-sweep-found-short-corrected` from `main` at `3e633252`:** records `66d8b73d`, documents `5a888378`; `scripts/check.sh all` on the branch exit 0 on its first run, 303 s, 7,758 passed and none failed over 60 result lines; merged at `a52db2fc` on the gate's second run, the first refused by ledger 374's `keyring` race in `test_a_note_filed_into_a_folder_made_here_has_nothing_to_be_sent`, the test that entry names, and the second green in 303 s with 7,758 passed. `git merge` left `MERGE_HEAD` staged after the refusal and `git commit -F` completed it. Nothing pushed.

**The gate per file**, by `scripts/which-checks.sh every-record-the-sweep-found-short-corrected <file>`: `guards/guards.toml` `affected`, read by the seven `house_style` tests as tree guards; `docs/development/measurements.md` and `docs/changelog.md` `docs_only`, the page's reading passing with 25.

**Ledger** 467 before, 471 after: 468 unmet-truth, the untested gate in `contacts_sync.rs`; 469 todo, the redundant insert in `caldav_sync.rs`; 470 deviation, the two records blind on a runner; 471 unmet-truth, the changed-day rule no marking holds. Both halves of each, no backslash, no carriage return. 463 was closed by the answer.

**Rows on the page**, all dated 2026-09-15 at `df3437a1`: the runners' wall clock, 4 h 7 min 32 s against the 20 hours predicted for one machine, with 64.7 hours of runner time behind it; 803 measured, 772 agreed, 31 not, 0 contended, 0 unmeasurable; the 31 by direction, 24 named too few and 5 named a test that stayed green, 0 whose break no longer applied; the runner's rate, 260 s a record median against 92 s here. `every_number_carries_its_command_and_its_date` accepts them, 25 passed.

**What the coordinator's message said that the tree did not.** "15 shards failure" and "27 green": the run's jobs say 15 red and 26 green among the 41 shards, with the generator the 42nd job. "`3e633252` added 3 guard records" and `main` at `b611ed82`-era metadata: `main` was at `3e633252` when this started, holding 806 records, the 803 of `df3437a1` plus 3. Both are counts re-taken rather than quoted.

## The checkpoint, answered

Pratik answered on 2026-09-15 by widening it: **"Go for running the guard sweep via CI as well."** The repository is public, so GitHub's Windows runners are free and twenty jobs run at once; the sweep moves there in shards instead of twenty hours on his machine. What was done for the answer, all on branch `the-sweep-runs-in-shards-on-runners` from `main` at `b611ed82`, merged alone at `bd8c2832`, nothing dispatched:

**`--shard K/N` on `scripts/guards.py`.** Shard K of N counting from 0, one contiguous block of the file's records cut at `K * len // N`, taken over the whole file's order before any other narrowing, so the same file and N give the same shard on every machine and two shards' logs concatenated in shard order read as the file does; 802 records in 41 shards is 19 or 20 each and every record once. Refused outside `0 <= K < N` before the record is read, so a runner handed a bad shard finds out in its first second. Written into the run's first line: `== 20 guards, shard 3/41 of 802 records, one build and one run each ==`. Refused beside `--recount-everything` with the other narrowings. An empty shard, which more shards than records leaves, is reported as nothing to measure rather than refused, because the dispatch that asked for it has already started the other jobs. Each refusal run once:

```
--shard 41/41: k counts from 0 and must sit inside 0..40, so shard 41 of 41 is not one.
--shard 3: written as k/n, which shard out of how many, for example 3/41.
Shard 0/1000 of 802 records holds no record, so there is nothing to measure. That is an answer, not a clean sweep.
--recount-everything writes a fingerprint on every record in the file, so it cannot also be narrowed, and this run asked for --shard 3/41.
```

and `--shard 3/41 --log target/shard.log --stop-after 0` printed `0 measured this run; 20 of 20 remain`. `verdicts_in` reads several shards' logs joined into one file, headers, pre-read timing lines and closing lines included, and a worked example shows it; that example was green on arrival, because the reading is anchored on the loop's three-space indent and nothing else in a log carries it, and the summary says so rather than calling it a red. 113 worked examples in 57 items where there were 95.

**Probing the flag with a hard kill left a real break behind.** A probe of `--shard 900/1000`, meant to be empty, held one record (positions 721 to 722), started measuring, and was killed with `taskkill /F`; `git status` then showed `src/presentation/managers.rs` modified with the record's break in it. `python scripts/guards.py --log target/shard.log --resume --stop-after 0` refused, naming the file and printing `git checkout -- src/presentation/managers.rs`, which cleaned it. That is the first time the refusal has fired on a leftover it was written for rather than one planted for it, and it is observation 0005 in the skill log seen live.

**`.github/workflows/guards.yml`, "Would each guard still go red".** `workflow_dispatch` and nothing else under `on:`. A `range` job turns `first` and `last` into the matrix and refuses a range the 256-job cap would truncate; a `shard` job per k on `windows-latest`, `timeout-minutes: 360`, `fail-fast: false`, checks out `github.sha` with `fetch-depth: 0` (twenty-five records name `house_style` as their suite, and that target holds the share-of-history test that a one-commit checkout fails at the pre-read), installs `dtolnay/rust-toolchain@1.98.1`, caches the cargo registry and not `target/`, prints `python --version` and imports `tomllib` so the log says which interpreter ran, runs `bash scripts/guards.sh --shard k/n --log sweep-k-of-n.log`, and uploads the log with `if: always()`. `WIXEN_NO_AUDIO` is set as `ci.yml` sets it. No `--wait-until-quiet`: a runner is nobody's machine. The Python it finds: under `shell: bash` on the image CI last went green on, `windows-2025-vs2026` `20260907.229`, `python` is the image's own 3.12.10, and the same doctests run green under CI's Test Suite job there, so `tomllib` is present.

**`tests/the_guard_sweep_runs_on_runners.rs`**, a target of its own. The reading holds the workflow to: `workflow_dispatch` and no other trigger; the three inputs declared and every `inputs.<name>` a step reads declared; a step running `scripts/guards.sh` with `--shard` and `--log`, passing only flags read off the script's own `add_argument` lines and not `--wait-until-quiet`; the pinned compiler; `WIXEN_NO_AUDIO`; and in the job that runs the script, `ref: ${{ github.sha }}`, `fetch-depth: 0`, an upload with `if: always()`, a timeout and `fail-fast: false`. The companion accepts a sound workflow and plants each of eleven mistakes, one at a time, requiring each to be named. Red first: the reading failed on the absent workflow; the companion was green on the same commit. Two bugs in the reading were found by the companion before the red commit, a `match_indices` index misread and `- if: always()` not matching as a list item; both fixed before anything was committed, which is what the companion is for.

**`scripts/check.sh`**: the target joins `guards_that_read_the_whole_tree`, seventh, with the reason: a workflow commit answers `all` on its own, and the other half of the coupling, a change to the script's flags, maps to no target.

**One record**, "the guard sweep's shard step hands the runner a flag it accepts", `file = .github/workflows/guards.yml`, `suite = the_guard_sweep_runs_on_runners`, breaking `--shard` to `--shards` in the run line. Taken by hand first: exactly the named test red, the companion green. Then `scripts/guards.sh --remeasure` agreed and wrote `tests_last_seen` as 2. 803 records; census 192 + 611.

**Commits, all through the hook:** red `ef2f5346` (14 of 17 new examples failing, trailer `test_the_guard_runner_still_obeys_its_own_examples`), green `a9220a25`, red `930cb991` (trailer `test_the_guard_sweep_dispatch_hands_the_script_flags_it_accepts`), green `9032b9be` (the workflow; `.github/workflows/` answers `all`, 7,752 passed and none failed over 59 result lines), record `6cf50cdb`, `scripts/check.sh all` on the branch's final tree exit 0 on its first run in 334 s, 7,752 passed, merge `bd8c2832` with the gate green on the merge in 327 s. Nothing pushed, nothing dispatched.

**Sizing, a guess until a shard has run.** A record on a 4-core runner is a rebuild plus a library run, guessed at three times this machine's 92 s, about five minutes; 802 records at 20 a shard is 41 shards of about 1.7 hours plus a build each, two waves of twenty jobs, roughly four hours of wall clock. Twenty a shard rather than forty because a runner job is capped at six hours, and forty at that guess is 3.3 hours plus the build. The first shard's timing lines are the measurement. `WIXEN_TEST_THREADS` is left at 8, unmeasured on 4 cores, so the runner's `timed:` lines read against the page's rate row at one setting; ledger 465 says so and says to re-take the curve on a runner if the shards run slower than the guess.

**What the gate selected, by `scripts/which-checks.sh the-sweep-runs-in-shards-on-runners <file>`:** `scripts/guards.py` `affected`; `tests/the_guard_sweep_runs_on_runners.rs` `affected` (its own target plus the tree guards); `.github/workflows/guards.yml` `all`; `scripts/check.sh` `affected`; `guards/guards.toml` `affected`, read by the seven `house_style` tests plus the new target's record; `docs/changelog.md` `docs_only`. `--suites-for .github/workflows/guards.yml` prints nothing, ledger 442's reason. `house_style.rs` holds 74 tests before and after (08-08 took it from 70), `wx_app.rs` 199; the count check printed no remedy.

## Performance

- **Duration:** about 1 hour 20 minutes from the first read at about 03:15Z to the merge at 04:22Z, then the worktree's first build and this summary
- **Started:** 2026-09-15T03:15:00Z, the evening of 2026-09-14 on the machine
- **Completed:** task 1 merged at 2026-09-15T04:22:39Z; the plan is partial
- **Tasks:** 1 of 3
- **Files modified:** 3 on the branch, plus this summary, `STATE.md`, `ROADMAP.md` and `WINDOWS.md` on `main`

## What the runner can now do

`scripts/guards.sh` takes four new flags, documented in its usage block with a sentence each:

| Flag | What it does |
|---|---|
| `--log PATH` | Appends every line the runner prints to PATH as well as the screen, flushed per write, stderr included so a traceback lands in the log under the record it ended on. Appends and never truncates, because `Start-Process` has no append and a redirect would overwrite the log a resume reads. |
| `--resume [PATH]` | Reads PATH, or the `--log` path when PATH is left out, with `verdicts_in`; skips every record with a verdict; prints `Resuming from PATH: N of M already measured, K to go`; measures the rest. A name with no verdict is measured again. Refused without `--log`. Before measuring anything it runs `git status --porcelain` over every guarded file and refuses, naming each modified file and the `git checkout` that puts it back. The pre-read is taken again on every invocation, because the tree may have moved between them. |
| `--stop-after N` | Cuts the selection to N records before the pre-read, so a chunk reads only the suites its records name; ends with a line saying how many remain and the resume command; exits 0. `--stop-after 0` measures nothing and reports what remains. |
| `--wait-until-quiet` | Before the pre-read and before each record, polls `tasklist` for `cargo.exe` and `rustc.exe` and waits, printing once a minute what it is waiting for, until none is alive; after each record's run returns, polls once more and, if one is alive, prints `   contended: another cargo ran during this record` under the verdict with the process named, and counts the record as unmeasured so a resume takes it again. |

The last line of every run is `the_closing_line`: `Every record selected has a verdict: M of M, N from the log and K from this run.` when nothing remains, or `K measured this run; R of M remain. Resume with:` and the command. That line is how somebody told not to read the verdicts knows whether the sweep is done.

`verdicts_in` reads the exact lines the loop and `say_what_it_found` print: `-- <name>` opens a block; the first line in the block indented with the loop's three spaces, the timing line excepted, is the verdict; `agreed` for the two clean shapes, `short` for a named test that stayed green or an unnamed one that went red, `could not be measured` for the five `Wrong` shapes and the loop's catch-all line; a `contended:` line anywhere in the block voids it; the last block for a name is the one that counts; a line it cannot place is refused with the line quoted and the record named. `could not be measured` is a verdict, so a resume skips it: a moved break measured again finds the same thing, and task 3 corrects it by hand.

The module docstring now says the script reads git in two places rather than one, and says which and since when.

## Red first

The six pure functions were written as docstrings with worked examples and no body. `python -m doctest scripts/guards.py` reported 33 of the 38 new examples failing, the five that passed being the helper definitions and an `is None` case a bodiless function satisfies. `cargo test --test house_style test_the_guard_runner_still_obeys_its_own_examples` was red on it: `test result: FAILED. 0 passed; 1 failed`. The red commit names that test in its trailer as `test_the_guard_runner_still_obeys_its_own_examples`, the bare name cargo prints for an integration target, which is what `red-commit.sh` reads and what 08-01's red commit `bb61e88e` used; the plan and the prompt wrote it with a `house_style::` prefix, which the gate would have reported as a test that never ran. The gate confirmed the red: "a red that is exactly the one this commit named."

`python -m doctest -v scripts/guards.py`: **57 tests in 39 items** before this task, **95 tests in 54 items** after, 38 more where the criterion asked for six.

## Proof by running

Each behaviour was run on this machine, with `tasklist` checked for `cargo.exe` and `rustc.exe` before each start; it was quiet every time except where a stand-in was started on purpose. Every log below was written by `--log`; every carriage-return count is `tr -cd '\r' < file | wc -c`.

**The plan's verify, one record.** `bash scripts/guards.sh --log target/one-record.log --stop-after 1 --wait-until-quiet`, exit 0, 10 lines and 10 carriage returns, one per line, as premise 7 said a text-mode write on Windows would leave:

```
== 1 guard, one build and one run ==

Reading what already fails with --lib, before anything is broken.
   timed: rebuild 29 s, run 49 s, 78 s in all
-- the Google merge asks what the whole contact is owed
   timed: rebuild 38 s, run 48 s, 86 s in all
   all 4 tests named went red, and nothing else did

1 measured this run; 797 of 798 remain. Resume with:
    scripts/guards.sh --log target/one-record.log --resume --wait-until-quiet
```

**The refusal.** With `src/application/allowed.rs` given a trailing blank line, `bash scripts/guards.sh --log target/one-record.log --resume --stop-after 1` exited 1 with, appended to the same log:

```
A guarded file is modified in the working tree, so nothing was measured:
    src/application/allowed.rs

A run killed hard enough to skip its restore leaves its break behind, and a
sweep resumed over it would measure every record against that edit. If the
change is not yours, put the file back and start again:
    git checkout -- src/application/allowed.rs
```

`git checkout -- src/application/allowed.rs` cleaned it; `git status --short src/` was then empty.

**The resume.** The same command on the clean tree skipped the measured record and measured the next; the log then held both verdicts, 30 lines and 30 carriage returns:

```
Resuming from target/one-record.log: 1 of 798 already measured, 797 to go
== 1 guard, one build and one run ==

Reading what already fails with --lib, before anything is broken.
   timed: rebuild 38 s, run 47 s, 85 s in all
-- the Microsoft merge asks what the whole contact is owed
   timed: rebuild 38 s, run 47 s, 85 s in all
   all 2 tests named went red, and nothing else did

1 measured this run; 796 of 798 remain. Resume with:
    scripts/guards.sh --log target/one-record.log --resume
```

**The detached start.** From PowerShell, `Start-Process -FilePath bash -ArgumentList 'scripts/guards.sh','--log','target/detach-test.log','--stop-after','1','--wait-until-quiet' -WindowStyle Hidden; exit`, so the PowerShell process was gone at once. Started 03:53:49Z; the closing line was in the log by 03:56:52Z; no `python.exe`, `cargo.exe` or `rustc.exe` left. The log, 10 lines and 10 carriage returns:

```
== 1 guard, one build and one run ==

Reading what already fails with --lib, before anything is broken.
   timed: rebuild 37 s, run 48 s, 85 s in all
-- the Google merge asks what the whole contact is owed
   timed: rebuild 37 s, run 47 s, 84 s in all
   all 4 tests named went red, and nothing else did

1 measured this run; 797 of 798 remain. Resume with:
    scripts/guards.sh --log target/detach-test.log --resume --wait-until-quiet
```

**The wait, the contended mark, and the re-measure.** A copy of `sleep.exe` named `target/rustc.exe` stood in for a foreign build, so no real build contended. In the first attempt it was started on a fixed delay and landed before the record instead of during it, so the runner waited for it twice (`waiting for a quiet machine: rustc.exe 8708` before the pre-read, `rustc.exe 32304` before the record, printed once a minute) and measured a clean record; that proved the wait and not the mark, and is observation 587 in the skill log. The second attempt started the stand-in five seconds after the `-- ` line appeared in the log, for 150 seconds:

```
-- the Google merge asks what the whole contact is owed
   timed: rebuild 37 s, run 47 s, 84 s in all
   all 4 tests named went red, and nothing else did
   contended: another cargo ran during this record
       rustc.exe 2012

1 record had another cargo alive as its run returned, so it is unmeasured and a resume takes it again:
    the Google merge asks what the whole contact is owed

0 measured this run; 798 of 798 remain. Resume with:
    scripts/guards.sh --log target/contend2.log --resume --wait-until-quiet
Resuming from target/contend2.log: 0 of 798 already measured, 798 to go
== 1 guard, one build and one run ==

waiting for a quiet machine: rustc.exe 2012
waiting for a quiet machine: rustc.exe 2012
Reading what already fails with --lib, before anything is broken.
   timed: rebuild 37 s, run 46 s, 83 s in all
-- the Google merge asks what the whole contact is owed
   timed: rebuild 36 s, run 47 s, 83 s in all
   all 4 tests named went red, and nothing else did

1 measured this run; 797 of 798 remain. Resume with:
    scripts/guards.sh --log target/contend2.log --resume --wait-until-quiet
```

The resume read the contended block as no verdict, waited for the stand-in, and measured the same record again. The stand-in was removed afterwards.

**The other refusals**, each run once: `--resume` without `--log` prints why and exits 1; `--log target/nope.log --resume` with no such file prints `Nothing to resume from: target/nope.log is not there` and creates nothing; a log holding `-- odd` followed by `   something else` is refused as `under "-- odd", a line this cannot read as a verdict:` with the line; a made-up complete log resumes with `798 of 798 already measured, 0 to go` and prints the closing line without a build; a log with one contended and one short block reports the short one as `That verdict came from the log.` and 1 remaining.

**The rate today.** Seven timing lines over the session: rebuild 36 to 38 s, run 46 to 49 s, 83 to 86 s a record, below the 92 s the page's rate row gives for 2026-09-14 at `bb61e88e`. The page is not changed by this plan; task 3 writes the sweep's own figures beside the prediction.

## What the first live run found

Opening the log for appending before reading it for a resume created the file, so `--log target/nope.log --resume` did not refuse: it read an empty log, found nothing measured, and started the whole sweep, pre-reading all 23 integration suites before the library. It was killed during the pre-read with no break applied; `git status` showed only `scripts/guards.py`. Two things changed: the log is read before it is opened, and `--stop-after` cuts the selection before the pre-read so a chunk reads only the suites its records name. The pure examples could not have seen either, because both are orderings in `main`; each refusal was then run once on the input it refuses. Observation 586 in the skill log.

## What the gate selected for each file

By `scripts/which-checks.sh a-sweep-that-can-be-stopped-and-picked-up <file>`: `scripts/guards.py` alone answers `affected`, `scripts/guards.sh` alone `affected`, `docs/changelog.md` alone `docs_only`, all three together `affected`; on `main` the three answer `all`. Neither script maps to a target of its own; what reaches `scripts/guards.py` is `house_style`'s doctest runner, in `guards_that_read_the_whole_tree`, and `scripts/guards.sh` is read by nothing. The hook on both commits ran formatting, clippy, the four shell suites and the tree-reading guards. `scripts/check.sh --suites-for scripts/guards.py scripts/guards.sh` prints nothing, for the reason ledger 442 records.

## Task commits

Branch `a-sweep-that-can-be-stopped-and-picked-up` from `main` at `4bdaa47f`.

1. **Red:** `812241ea` test(08-07), six bodiless functions with 38 examples, trailer `Fails-until-green: test_the_guard_runner_still_obeys_its_own_examples`.
2. **Green:** `bbd1f28d` feat(08-07), the bodies, the flags in `main`, `guards.sh`'s usage block, the changelog entry.
3. **Merge:** `1837f93b`, `Merge 08-07 task 1`, alone into `main`, the whole gate on the merge: 7,746 passed and none failed over 58 result lines, 307 s.

`scripts/check.sh all` on the branch at `bbd1f28d`: exit 0 on its first run, 345 s, 7,746 passed and none failed over 58 result lines; ledger 374's keyring race was not met. Every commit went through `git commit` or `git merge` with the hook on; nothing was piped; the gate's output went to a file and its exit status was read directly; no `.git/index.lock` was left. Nothing is pushed.

**Plan metadata:** the commit carrying this summary, `STATE.md`, `ROADMAP.md` and `WINDOWS.md`.

## The checkpoint: dispatch the sweep onto the runners, and tell the executor when the run has finished

This is written to be acted on without the plan. It was rewritten on 2026-09-15 after Pratik widened the checkpoint with "Go for running the guard sweep via CI as well."; the local start it used to give is kept below as the fallback.

**What it is.** The one guard sweep of the milestone: every record in `guards/guards.toml`, 803 today, one build and one whole-suite run per record, with nothing else building beside it. On GitHub's Windows runners it is 41 jobs of about 20 records each, dispatched by hand from the Actions tab, and nothing has been dispatched yet.

**First, push `main`.** Nothing is pushed: the workflow exists only here, and GitHub lists a workflow for dispatch only once it is on the default branch there (`gh run list --workflow guards.yml` answers 404 as this is written). The push runs CI's seven jobs as every push does; that is the usual push and not a dispatch.

**Dispatch.** On GitHub, Actions, the workflow "Would each guard still go red", Run workflow, on `main`. The three inputs as you will see them, with their defaults:

| Input | Description shown | Default |
|---|---|---|
| `shards` | how many shards the record is divided into, n; about 20 records each at the 2026-09-15 count of 802 | `41` |
| `first` | the first shard this dispatch runs, from 0 | `0` |
| `last` | the last shard this dispatch runs, inclusive, at most first + 255 | `40` |

The defaults run the whole record in one dispatch: 41 jobs, twenty at a time. The run is at `main`'s head at the moment you press the button (`github.sha`), which is what "the same commit" means below. Nothing else needs to be quiet on your machine; the runners are nobody's machine.

**How long, a guess until a shard has run.** A record on a 4-core runner is a rebuild plus a library run, guessed at three times this machine's 92 s, about five minutes; 20 records a shard is about 1.7 hours plus a build each, two waves of twenty jobs, roughly four hours of wall clock. The 6-hour job cap is why it is 20 a shard and not 40. The first shard's `timed:` lines are the measurement; the page's rate row is this machine's and says nothing about a runner.

**How to know it is done, without reading the verdicts.** The run's page shows every shard job finished, green or red; a red shard is a shard that found a record short, and its log is still uploaded. A job killed at the six-hour cap is the one case to look at: its log stops at some `-- name` with no verdict, and the read-back below names what remains. Do not read the verdicts: task 3 reads them after the logs are merged, because a partial log has already produced a wrong commit message here once.

**When it is done, give the executor the run id** (the number in the run's URL). The executor then, on this machine:

```
git -C ../wixen-mail-sweep checkout --detach <the run's github.sha>
cd ../wixen-mail-sweep
gh run download <run id> --dir target/runner-sweep
for k in $(seq 0 40); do cat target/runner-sweep/sweep-$k-of-41/sweep-$k-of-41.log; done > sweep.log
bash scripts/guards.sh --log sweep.log --resume --stop-after 0
```

The first line moves the sweep worktree to the commit the run judged, which is `main`'s head when it was dispatched, so that task 3's corrections are read against the tree the sweep measured; the worktree sits at `abf3e24c` as this is written and is moved when the run completes, not before. The last line must print `Every record selected has a verdict: 803 of 803, 803 from the log and 0 from this run.` (or the count the record held at that commit); `K measured this run; R of N remain. Resume with:` names what a killed shard left, and a second dispatch of that shard number alone (`first` and `last` both set to it) fills it, its log concatenated onto `sweep.log` before the read-back is run again. That line is task 3's precondition. The executor also measures the merged log's carriage returns with `tr -cd '\r' < sweep.log | wc -c`, since the runner's Python writes them as this machine's does, and checks `git status --porcelain` in the worktree names nothing but `?? sweep.log` and `target/`'s untracked download is under an ignored directory.

## The fallback: the sweep on this machine, as the checkpoint first gave it

Kept in case the runners cannot be used. Everything below was run once here except the sweep itself.

**How long.** The row "The whole guard sweep, every record once" on `docs/development/measurements.md` reads **784 x 92 s = 72,128 s, about 20 hours**, dated 2026-09-14 at `bb61e88e`, from the rate row above it, 92 s a record. The file holds 798 records today, so the same rate gives about 20.4 hours; the seven records timed in this session ran 83 to 86 s each, which would be about 19 hours. Both are a rate times a count and the log will say what it really cost, one `timed:` line per record. Triage of what it finds is on top.

**Where.** A worktree at `main` as it stood after this task merged, `1837f93b`, which is the first commit that holds the flags below. It already exists and is built:

```
git worktree add ../wixen-mail-sweep 1837f93b
```

was run from the main checkout, and inside it `cargo test --lib -- --list | tail -1` answered `7264 tests, 0 benchmarks` after 5 m 48 s, 8.2 GB under `target/`. If the worktree is gone when you read this, those two commands recreate it. `git log --oneline -1` inside it must print `1837f93b`. Nothing else has landed on `main` since that merge as this is written; if something has by the time you start, the sweep still runs at `1837f93b`, because task 3's corrections are made against the tree the sweep judged and anything newer is covered only by the per-commit checks.

**Corrected 2026-09-15 by 08-08, before the sweep started: the worktree is at `abf3e24c`, not `1837f93b`.** 08-08 merged into `main` three times that day, `99682439`, `1401e4d3` and `abf3e24c`, and added four records to `guards/guards.toml` between them ("shards from two commits are not read as one run", "shards over two file filters are not read as one run", "a dispatched shard hands the mutation script a flag it accepts", "a CI job that runs the tests checks out the whole history") and corrected a fifth, so a sweep at `1837f93b` would have measured 798 records and never those. The worktree was moved first to `1401e4d3` and then, after the third merge, with `git -C ../wixen-mail-sweep checkout abf3e24c`; `git log --oneline -1` inside it prints `abf3e24c`, `git status --porcelain` is empty, and `cargo test --lib -- --list | tail -1` there still answers `7264 tests, 0 benchmarks`, because nothing under `src/` changed between the commits; `tests/house_style.rs` gained four tests, which is why the remedy re-measured 22 records naming it. The record count is 802 and the census 192 + 610. Wherever this checkpoint says `1837f93b`, read `abf3e24c`, including the check under "When it is done": `git log --oneline -1` must be `abf3e24c`, and the closing line reads `802 of 802`. The mutation worktree of 08-08, `../wixen-mail-mutants`, sits at the same commit; run the sweep first, then the scoped mutation run Pratik chose, and neither beside the other. Nothing else in this summary is changed.

**While it runs, nothing commits on `main`.** The worktree keeps `main` from changing the files under the sweep, but a commit on `main` runs the hook, and the hook runs the suite on the same machine and the same `target/` contention; that is the case the tree records as reporting three guards green that go red on their own. The runner now notices a foreign cargo alive at the end of a record and marks that record for re-measuring, but a build that starts and finishes inside a record's run is seen by neither poll, so the cheaper answer is not to start one. Nothing that runs cargo, including an editor's rust-analyzer, should be open against either checkout.

**Start.** From PowerShell, with no `cargo.exe` or `rustc.exe` running (`tasklist /FI "IMAGENAME eq cargo.exe"` and the same for `rustc.exe` should both say no tasks):

```
Start-Process -FilePath bash -WorkingDirectory 'C:\Users\prati\Documents\projects\wixen-mail-sweep' -ArgumentList 'scripts/guards.sh','--log','sweep.log','--wait-until-quiet' -WindowStyle Hidden
```

Close the PowerShell window; the run does not need it. `sweep.log` grows in the worktree root by one `-- name` block per record. The first two to three minutes are the pre-read of what already fails, one whole-suite run per distinct suite the records name, 23 integration targets and the library, and they print only a `timed:` line each; that silence is by design.

**Stop.** Every stop of a hidden run is a hard kill, so expect what the next paragraph describes. Find it and kill its tree:

```
Get-CimInstance Win32_Process -Filter "Name='python.exe'" | Where-Object CommandLine -like '*guards.py*' | Select-Object ProcessId, CommandLine
taskkill /PID <that ProcessId> /T /F
```

`/T` takes the cargo and rustc it spawned with it; without `/T` they run on as orphans and the next start waits for them to finish, which is also fine.

**What a hard kill leaves, and what catches it.** The runner puts the broken file back in a `finally`, which runs on Ctrl+C and not on `taskkill /F`, so the record that was running is likely to leave its break in the tree. The next start with `--resume` reads `git status` over every guarded file first and refuses, naming the file and printing the command, which you run in the worktree:

```
git checkout -- <the file it names>
```

then start again. The block for that record in the log has no verdict, so the resume measures it again. A copy of the file may also be left under `%TEMP%\wixen-guards-*`; it is harmless and can be deleted. Nothing else is touched: the runner changes nothing through git.

**Resume.** The same line with `--resume` added:

```
Start-Process -FilePath bash -WorkingDirectory 'C:\Users\prati\Documents\projects\wixen-mail-sweep' -ArgumentList 'scripts/guards.sh','--log','sweep.log','--resume','--wait-until-quiet' -WindowStyle Hidden
```

It appends to the same log, prints `Resuming from sweep.log: N of 798 already measured, K to go`, takes the pre-read again, and measures again any record the log marks contended or left without a verdict. Repeat until a start prints that nothing remains.

**How to know it is done, without reading the verdicts.** The last line of the log after the final start is

```
Every record selected has a verdict: 798 of 798, N from the log and K from this run.
```

Any other last line, `K measured this run; R of 798 remain. Resume with:` and the command, means resume again. Do not read the verdicts above it: task 3 reads them after the log is complete, because a partial log has already produced a wrong commit message here once.

**When it is done, say so and give the worktree's path.** The executor then runs, in the worktree, `bash scripts/guards.sh --log sweep.log --resume --stop-after 0`, which must print the line above; checks that `git status --porcelain` names nothing but `?? sweep.log`; checks that `git log --oneline -1` is `1837f93b`; measures the log's carriage returns with `tr -cd '\r' < sweep.log | wc -c`; and does task 3. The plan's verification sentence said the worktree's status would be empty; the log is untracked at the root, so the exact expectation is written here instead.

## What the tree contradicted in the plan

1. **The trailer name.** The plan and the prompt name the red as `house_style::test_the_guard_runner_still_obeys_its_own_examples`; `red-commit.sh` reads cargo's `test NAME ... FAILED` lines, which for an integration target carry the bare name, and 08-01's red commit `bb61e88e` used the bare name. The bare name was used and the gate accepted it.
2. **The census header is at `guards/guards.toml:82-83`, not `79-80`**; 08-06's comment edit above it moved it. It still reads 192 and 606, which sum to the 798 records the parser counts. Not touched.
3. **The product row's `what` is "The whole guard sweep, every record once"**, not "guard sweep, product"; the checkpoint quotes it by its own words.
4. **`--resume PATH` without `--log`** is a run whose verdicts go nowhere and whose next resume starts from the same place, so it is refused with the flag to add, where the plan only said a bare `--resume` takes the `--log` path. Ledger 456.
5. **The checkpoint's verification** says `git status --porcelain` in the worktree is empty; `sweep.log` at the root is untracked and not ignored, so it will show as `?? sweep.log`. Written into the checkpoint as the exact expectation rather than left to be found. The log stays at the root as the plan placed it rather than under `target/`, which a build tool owns.
6. **`Start-Process` from PowerShell's cwd** is what the plan's line relies on; `-WorkingDirectory` is added so the line is exact wherever it is pasted.

## Deviations from Plan

**1. `--resume` refused without `--log`.** Under contradiction 4. Ledger 456.

**2. A build that starts and finishes inside one record's run is marked by neither poll.** The plan's behaviour names this gap as accepted and the changelog's Known limitations says it; ledgered so it is visible at ship time rather than only in a plan. Ledger 457.

**3. The worktree was created and built by the executor.** The plan gives both commands to Pratik. They were run once here so the checkpoint's instructions are ones that have been executed: the worktree is at `1837f93b`, 8.2 GB, 5 m 48 s for the first build, and `--stop-after 0` in it printed `0 measured this run; 798 of 798 remain`. The log that run created was removed so the sweep's log starts with the sweep. The sweep itself was not started.

**4. The stop-after cut moved before the pre-read.** Under "What the first live run found". A design refinement inside the plan's behaviour, not a change to it.

**5. The answer's reading is a new integration target, not two tests in `house_style.rs`.** The coordinator named `house_style.rs` as the pattern's home; twenty-five records name that target as their suite and every added test flags them, and the phase README says new readings go in new targets at zero records. The target is in `check.sh`'s whole-tree list and has its record. Ledger 465 is the thread-setting guess, not this.

**6. The concatenation example was green on arrival.** The coordinator asked that the parser be made to tolerate several headers and summary lines with a worked example; it already did, since the reading is anchored on the loop's indent, so the example documents rather than drove, and the red of that commit was the three shard functions.

Otherwise the plan's task 1 and the answer were executed as written. No package installed, `Cargo.toml` untouched, no version bump, no test added to or removed from any existing `.rs` file (the two in the new target are the new target's), no existing record's `before`, `after` or `red` changed, no committed tracked file edited by anything but the editing tool (exception set: zero, kept at zero; one temporary break applied to `guards.yml` by a two-line script to take the record red by hand and restored by `git checkout`, the way the runner applies one, said here rather than hidden), carriage returns measured with `tr -cd '\r' | wc -c` on every tracked file touched, none; no em dash, `test_no_dashes_that_should_be_punctuation` in the gate on every commit. `WIXEN_TEST_THREADS` at its default here and in the workflow.

## Guard records

None added, removed or re-measured. 798 by the parser before and after; census 192 + 606 at `guards/guards.toml:82-83`. `tests/house_style.rs` holds 70 test functions and `src/presentation/wx_app.rs` 199 before and after by `grep -cE '^\s*#\[(test|tokio::test)\]\s*$'`, so `test_every_guard_record_says_how_many_tests_the_files_it_names_held` printed no remedy on any commit. `scripts/guards.py` is named by no record. The two records measured during the proofs, `the Google merge asks what the whole contact is owed` and `the Microsoft merge asks what the whole contact is owed`, agreed every time, four and two named tests red and nothing else; no count was written, because none of those runs was `--remeasure`.

**For the answer:** one record added, "the guard sweep's shard step hands the runner a flag it accepts", taken by hand first and then through `--remeasure`, which agreed and wrote `tests_last_seen` as `tests/the_guard_sweep_runs_on_runners.rs`, 2. 802 records before, 803 after; census 192 + 611 at `guards/guards.toml:83-84`. `house_style.rs` 74 and `wx_app.rs` 199 before and after; the count check printed no remedy on any commit.

## Ledger

`.planning/WINDOWS.md` 455 before, 457 after, both halves of each entry written by `gsd-tools windows append`, checked by `grep -c "| 456 |\|| 457 |"` and `grep -c '"id": 456\|"id": 457'`, two and two; no backslash in the added lines, `grep -cF '\'` over the diff's added lines 0; no carriage return.

- 456, deviation, `scripts/guards.py`: `--resume` refused without `--log`.
- 457, deviation, `scripts/guards.py`: a build inside one record's run is seen by neither poll.

**For the answer:** 464 before, 465 after. 463, 08-08's entry that the sweep could be sharded onto runners and was not, closed with `gsd-tools windows fixed 463`, both halves `fixed`. 465 added, deviation, `.github/workflows/guards.yml`: `WIXEN_TEST_THREADS` left at 8 on a 4-core runner, unmeasured, with why and what re-takes it. Both halves checked, one and one; no backslash, no carriage return.

## Issues Encountered

The runaway run, above, killed during its pre-read with nothing broken. `git merge -F -` does not read a message from stdin; the merge was made with a message file. Nothing else stopped anything; the gate passed on every commit through the hook and `scripts/check.sh all` passed on its first run.

## Known Stubs

None. Every flag is reachable from `scripts/guards.sh`, exercised in this session, and documented in its usage block; the workflow is reachable from the Actions tab, held by a reading and a record, and has run once to completion. Two rules the sweep found unguarded are ledgered, not stubbed: the tests that would notice belong in files many records name.

## Threat Flags

| Flag | File | Description |
|---|---|---|
| threat_flag: ci-spend | `.github/workflows/guards.yml` | A dispatch is 41 runner jobs of hours each; `workflow_dispatch` only, held by the reading, so nothing but a person starts one. `permissions: contents: read`; nothing is written back. |

`tasklist` and `git status` are read; nothing is written through either. No network endpoint, auth path or schema. T-08-SC: no package added.

## Self-Check: PASSED

`scripts/guards.py` holds `def verdicts_in`, `def foreign_builds_in`, `def is_quiet`, `def why_a_resume_is_refused`, `def the_resume_command`, `def the_closing_line`, `def the_shard_asked_for`, `def the_records_in_shard`, `def the_header_for` and `class Logged`, checked by `grep -c`; `scripts/guards.sh` holds `--wait-until-quiet` and `--shard`; `.github/workflows/guards.yml` and `tests/the_guard_sweep_runs_on_runners.rs` exist; `scripts/check.sh`'s list holds `the_guard_sweep_runs_on_runners`; `docs/changelog.md` holds "The guard sweep can be stopped and picked up from its own log" and Pratik's words, which wrap across a line there so `grep -c "Go for running the guard sweep via" docs/changelog.md` is the grep that finds them, 1; the commits `812241ea`, `bbd1f28d`, `1837f93b`, `ef2f5346`, `a9220a25`, `930cb991`, `9032b9be`, `6cf50cdb`, `bd8c2832`, `66d8b73d`, `5a888378` and `a52db2fc` are in `git log --all`; `../wixen-mail-sweep` exists at `df3437a1` with `sweep.log` and the download under `target/`; `.planning/phases/08-every-number-the-project-quotes/sweep.log` holds 3,066 lines, 3,066 carriage returns and 803 `-- ` blocks; `guards/guards.toml` parses to 805 records and its census reads 802 and 3; `main` is ahead of `origin/main` and nothing was pushed by this executor. The earlier sentence here, that `gh run list --workflow guards.yml` answered 404, was true when written and is not now: Pratik pushed and dispatched, and the run is 34965790937.

## Status

`complete`. Task 1 merged alone into `main` at `1837f93b`; the checkpoint's answer, the runner sweep, merged alone at `bd8c2832`; the sweep dispatched by Pratik, run 34965790937 at `df3437a1`; task 3 merged at `a52db2fc`. The plan's three verify commands pass: `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`, `test_every_guard_record_says_how_many_tests_the_files_it_names_held`, and `every_number_carries_its_command_and_its_date` with 25.

Criterion 5 closes on the count of records the log holds a verdict for: 803 of the 803 the file held at the sweep's commit, every one it reported short corrected by hand and measured again, and the two it reported short in error named as the runner's blindness rather than corrected.

The checkpoint section above is kept as it was written, as the record of how the dispatch was to be made; the paragraph "When it is done" is what task 3 did, with `sweep.log` at the worktree root named as `?? sweep.log` in the status as it said.

---
*Phase: 08-every-number-the-project-quotes*
*Completed: 2026-09-15*

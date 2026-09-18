---
phase: 11-reading-and-the-list
plan: 02
subsystem: CI, NVDA tests, account manager, status line, guards, requirements
tags: [nvda-workflow, guardrail-4, sign-in-failure, one-notification, settings-tabs, found-08, found-09, ledger]

requires:
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-05: the five editors as scan targets in accessibility.yml's list; 09-06: nvda-tests/tests/settings-tabs-read-once.test.js and the tab row answering its own arrows"
  - phase: 11-reading-and-the-list
    provides: "11-01 merged at 316ea755, so this merge lands on a main whose one CI failure is fixed"
provides:
  - ".github/workflows/nvda.yml: the job's failure is the run's since 2026-09-18; the summary and the upload keep if: always(); the header says why with the run named"
  - "nvda-tests/tests/settings-tabs-read-once.test.js: the mark after the settle and no wait for an opening announcement; the comment says what the case proves and no longer proves"
  - "nvda-tests/README.md: all five files under tests/ in the table, four run and one skipped, dated; a section on what the log holds and does not"
  - "presentation::status_line::shown_and_signalled(line, a11y, event, said): the sentence on the line and the event raised with the sentence as its detail, once"
  - "wx_account_manager::reauthorize_selected: the NoCreds, NotSaved and Failed arms one call each with AccountNeedsAttention, no separate signal; the two readings rewritten in place, 14 tests"
  - "guards/guards.toml: 915 records, census 802 + 113; the sign-in record rewritten onto the new call at 3 red, a record on the account manager at 2 red, a record on status_line at 1 red, the status_line record re-measured at 3 tests"
  - "docs/changelog.md: Changed, the NVDA workflow fails when a case fails; Fixed, a sign-in failure is one announcement"
  - ".planning/REQUIREMENTS.md: FOUND-08 ticked on run 35336142914's walk, FOUND-09 on the tester's ear, FOUND-18 on its [D] lines; ledger 489, 492 and 494 closed, 531 and 532 opened"
affects: [11-03 onward, which merge onto a main whose next push runs four NVDA cases as a gate; 11-12, which reads FOUND-18's [S] line when the push has run]

actuals:
  tokens: 17911
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "An outcome that is both an answer and an event goes out as one event carrying the sentence, not as a sentence and then the event; the feedback module composes the line as the event's word then the detail, so one call and one topic"
    - "A harness's transcript is evidence of what the harness captures, not of what was spoken: read every passing transcript's first entry before writing a wait for something no transcript has ever held"

key-files:
  created: []
  modified:
    - .github/workflows/nvda.yml
    - nvda-tests/README.md
    - nvda-tests/tests/settings-tabs-read-once.test.js
    - src/presentation/status_line.rs
    - src/presentation/wx_account_manager.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/REQUIREMENTS.md
    - .planning/WINDOWS.md

key-decisions:
  - "The existing sign-in record is rewritten onto the new call rather than deleted: its before, the said_and_shown call in the Failed arm, no longer named a place in the tree, and its break (the sentence shown and nothing said) is still the silence it was written against; it now reddens three, measured"
  - "Ledger 494 closes with this plan though the plan did not name it: the README rewrite the plan asked for is that entry's remedy, and an entry left open over a fix that landed is a ledger nobody reads"
  - "The two-failures reading locates the sign-in arm by the function and the arm's head, not by 150 characters around the sentence: the four-line call sits above that window, and the first Failed arm in the file is the add flow's"
  - "No stub in the red: both new readings read source text, so the helper's absence is an assertion failure and not a compile error, and the red commit holds tests only"
  - "The whole-file grep for shown_and_signalled( answers 5, not the plan's 3, because the two readings quote the needle; the code part answers 3 and the acceptance criterion is met in its meaning"

patterns-established:
  - "A criterion that greps a file for a string's absence collides with the instruction to write, in the same file, why the string went (observation 670); scope the grep to where the string acts"
  - "A source reading that finds its site by a fixed window around a sentence encodes the code's line count; locate by the function and the arm (observation 687)"

requirements-completed: [FOUND-18, FOUND-08, FOUND-09]

coverage:
  - id: D1
    description: "A failing NVDA case fails the run; the summary and the upload still run; the header says why"
    requirement: FOUND-18
    verification:
      - kind: command
        ref: "grep -c 'continue-on-error' .github/workflows/nvda.yml -> 0; grep -c 'if: always()' .github/workflows/nvda.yml -> 2; grep -c '2026-09-18' .github/workflows/nvda.yml -> 1"
        status: pass
      - kind: integration
        ref: "tests/house_style.rs, the workflow readings among its 74, green at af30aa0f on a hook run of all"
        status: pass
    human_judgment: false
  - id: D2
    description: "The settings case takes its mark after the settle and presses Right; the README lists every case and says what the log holds"
    requirement: FOUND-18
    verification:
      - kind: command
        ref: "node --check nvda-tests/tests/settings-tabs-read-once.test.js -> ok; grep -c 'waitToHearAll(nvda, \\[TABS\\[0\\]' -> 0; grep -c 'timesHeard(heardGoingRight, TABS\\[0\\]' -> 1; each of the five test file names found in the README"
        status: pass
    human_judgment: true
    rationale: "The case is read and not run here, by nvda-tests/README.md's rule; the next push of main runs it, ledger 531"
  - id: D3
    description: "The three sign-in failure arms are one notification each through shown_and_signalled with AccountNeedsAttention"
    requirement: FOUND-18
    verification:
      - kind: unit
        ref: "src/presentation/status_line.rs#test_an_outcome_that_is_also_an_event_is_one_notification_carrying_the_sentence"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_account_manager.rs#test_an_account_still_unauthorised_after_trying_again_reaches_the_earcon_channel"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_account_manager.rs#test_the_two_failures_on_this_screen_are_said_above_the_ordinary_run"
        status: pass
      - kind: other
        ref: "guards/guards.toml, three records measured and one re-measured 2026-09-18 on the library, each 'and nothing else did'"
        status: pass
    human_judgment: false
  - id: D4
    description: "FOUND-08 ticked on the walk, FOUND-09 on the tester's ear, the ledger told"
    requirement: FOUND-18
    verification:
      - kind: command
        ref: "grep -c '^- \\[x\\] \\*\\*FOUND-08\\*\\*' .planning/REQUIREMENTS.md -> 1, the same for FOUND-09; cargo test --test the_planning_files_agree_with_themselves -> 16 passed"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the sign-in line is heard whole on the runner, and which tab the corrected case's first Right reaches on a fresh profile"
    requirement: FOUND-18
    verification: []
    human_judgment: true
    rationale: "The next push of main is Pratik's; these cases never run on a machine somebody is using; ledger 531 and the changelog's Known limitations"

duration: 47min
completed: 2026-09-18
status: complete
---

# Phase 11 Plan 02: The NVDA run's verdict is its job's, and a sign-in failure is one notification Summary

**A case that fails in the NVDA workflow fails the run now. The job had carried
`continue-on-error: true` since 2026-08-16, and run 35336142908 on `main` at `744d05ef`
reported success over a job in which two of five cases failed, one of them at every run since
2026-09-15. Inside it, the two cases were read and each was fixed where it was at fault: the
settings tab-row case had waited for an opening announcement no transcript of any case has ever
held, and now counts from its first key; the sign-in case had heard the event's line and not the
sentence, because the Account Manager sent them as two notifications a millisecond apart, and
the three failure arms are one call each now, the event carrying the sentence. FOUND-08 is
ticked on the Accessibility run's walk of the five editors and FOUND-09 on the tester's ear.
Nothing pushed; whether the sign-in line is heard whole and which tab the first Right reaches
are the next push's to show, and that push is Pratik's.**

## Performance

- **Duration:** 47 min from the reading's end at 13:15:56Z to the merge at 14:02:34Z, of
  which 7 min 19 s was guard measurement in one run (rebuild 35 to 41 s and run 53 to 58 s per
  record by the runner's `timed:` lines), about 6 min the four hook runs on the branch (356 s,
  106 s, 119 s, 119 s), and about 11 min 40 s the two whole gates (362 s on the branch, 340 s
  on `main` at the merge); the summary and the planning files after
- **Started:** 2026-09-18T13:15:56Z (the branch; the reading began about 12:50Z)
- **Merged:** 2026-09-18T14:02:34Z at `1c0e9b0b`
- **Tasks:** 3
- **Files modified:** 9, the ledger and the requirements among them; none created

## What landed

**Task 1, the workflow, the case and the README.** `.github/workflows/nvda.yml` loses the
job's `continue-on-error: true`; `if: always()` stays on the summary step and the upload, so a
red run still carries its transcript and the summary still tells "no transcript was written"
from "at least one test failed". The header's last paragraph says, dated, that a failing case
fails the run since 2026-09-18, names the run that reported success over a failed job, and
says why `accessibility.yml` keeps its own setting. The literal strings `continue-on-error` and
`if: always()` are not in the header, because the plan's acceptance criteria grep the file for
them (0 and 2), which is observation 670.

`nvda-tests/tests/settings-tabs-read-once.test.js`: the first `waitToHearAll(nvda,
["General"])` is gone; the case sleeps the settle, takes the log's length as its mark, and
presses Right. The comment above says why with the run named: the case's first run timed out
there with an empty log before any key, and the transcript of every case that passed begins
with what its first key made NVDA say. It says what the corrected case proves (Compose to
Advanced once each and in order going Right, General not heard going Right, Feedback once
going Left) and what it no longer proves (General's own reading at open, which only the
tester's ear settled, #33 closed 2026-09-18), and what a failure reading "never heard: Compose"
would mean (focus at open on a fresh profile, not the row). Two other comments in the file that
had claimed the opening reading was in the log were corrected to say what the run captured.
`node --check` accepts it. Read, not run: `nvda-tests/README.md`, "This never runs on your own
machine", and this machine is the tester's with NVDA up.

`nvda-tests/README.md`: the table lists all five files under `tests/` with what each holds
and whether it runs; the intro dates the two cases the package began with and counts the five;
the CI section says four run and one is skipped, that a failing case fails the run since
2026-09-18 and why, and a new section, "What the log holds, and what it does not", says that
the spoken-phrase log begins with what the first key made NVDA say and holds no dialog-opening
announcement, so a case waits for a key's reading and never for an opening. This closes ledger
494, which the plan did not name.

**Task 2, one notification.** `status_line::shown_and_signalled(line, a11y, event, said)`
sets the label and calls `a11y.signal(event, said)`, once; its doc names the run and says why
one event carrying the sentence is one notification, and what line NVDA hears. The three arms
of `reauthorize_selected` that leave the account unauthorised, `NoCreds`, `NotSaved` and
`Failed`, call it with `FeedbackEvent::AccountNeedsAttention` and their sentence, and call
neither `said_and_shown` nor `a11y.signal` (`grep -v '^\s*//' | grep -c
'a11y.signal(FeedbackEvent::AccountNeedsAttention'` answers 0; the code part holds 3 calls of
the new helper, and the whole file 5 because the two readings quote the needle). The
`Authorized` arm and the password arm are unchanged. `said_and_shown` and its guard
`what_this_screen_never_says` are unchanged: the screen's count of bare `status.set_label(` is
0 and of `said_and_shown(` is 23, exactly the floor that check asks. Under `signal`, the
event's `text_with` composes "Sign-in needs attention, Signing in failed: ..." and the event's
own priority, Urgent, carries it, which is above the High the sentence used to go out at.

**Task 3, the requirements.** FOUND-08's second `[D]` line quotes the five walks from run
35336142914's log and the requirement is ticked with a dated read line; its row says Complete
on 09-05 and this plan, the walk still crashing here (ledger 390) and the tester's listening
pass his (ledger 490). FOUND-09 is ticked on the tester's ear, his closing comment on #33
quoted with its timestamp (2026-09-18T11:34:22Z), the third `[D]` line carrying the dated
account of the case's one run, the correction, what it proves and no longer proves, and
ledger 531; its row says Complete on 09-06 and this plan with the runner's case pending.
Neither `[S]` line is reworded. FOUND-18 is ticked in the commit that lands this summary, on
its four `[D]` lines with the tests, readings and runs above, its `[S]` line untouched.

## The tester's report

None arrived on the sign-in case by hand before this plan ran; the coordinator's note in the
prompt was that #33 closed on the tester's ear, which is quoted in FOUND-09.

## Task commits

| Commit | What |
|---|---|
| `af30aa0f` | fix(11-02): the workflow, the case, the README, the ledger (489, 492, 494 closed; 531, 532 opened) and the Changed entry; the hook ran `all` because `.github/` changed, 8,038 and none failed, 356 s |
| `eb2011ad` | test(11-02): the red half of task 2, three readings named by module path and the count check bare; the gate held it to exactly those four, 106 s |
| `a697f61f` | feat(11-02): the helper, the three arms, the four records and the Fixed entry; 119 s |
| `8c72e9b2` | docs(11-02): FOUND-08 and FOUND-09 ticked; the document-reading targets, 119 s |
| `1c0e9b0b` | Merge 11-02 into `main` |

Branch `the-nvda-runs-verdict-is-its-jobs` from `main` at `37a44643`. Not pushed; 12 commits
ahead of `origin/main` before the commit that lands this summary, by `git rev-list --count
origin/main..main`.

## Honest RED and GREEN

The red at `eb2011ad` named three readings by module path, from cargo's own lines, and the
count check bare:
`presentation::status_line::tests::test_an_outcome_that_is_also_an_event_is_one_notification_carrying_the_sentence`
(the helper did not exist; the reading returns "nothing here both shows a sentence and raises
the event", an assertion and not a compile error, because it reads text);
`presentation::wx_account_manager::tests::test_an_account_still_unauthorised_after_trying_again_reaches_the_earcon_channel`
(rewritten in place to count `shown_and_signalled(` with the event against the unauthorised
arms and to refuse a separate `a11y.signal(` in the function; 0 calls against 3 arms);
`presentation::wx_account_manager::tests::test_the_two_failures_on_this_screen_are_said_above_the_ordinary_run`
(the sign-in half rewritten to hold the arm to the new call and the event, and the event's own
level to above High by a real call on `FeedbackEvent::AccountNeedsAttention.priority()`; the
browser half as it was); and
`test_every_guard_record_says_how_many_tests_the_files_it_names_held`, because
`status_line.rs` went from 2 tests to 3 and one record names it. The gate in `red` mode ran
exactly those four red and nothing else. No stub was written: the plan's "a stub that sets the
label and does not signal keeps it red on the assertion rather than on a compile error" was
written for a test that calls the helper, and both new readings read text.

The green at `a697f61f` ran the two scoped targets, 3 and 14, and the tree guards, no marker,
no remedy printed, because the remedy the red commit printed was run in the foreground before
the green was committed, and read. One reading's window changed in the green and not in the
red: `test_the_two_failures...` had read 150 characters around "Signing in failed:", which the
four-line call sits above once rustfmt lays the extra argument on its own line; the first
repair, reading forward from the first `OAuthFlowResult::Failed(msg) =>` in the file, found the
add flow's arm, which says "Account added, but authorization failed" through `said_and_shown`
and is not this plan's. It reads the arm inside `reauthorize_selected` now, through the module's
own `the_reauthorize_function`, and it was red under either window at `eb2011ad`. Observation
687.

`wx_account_manager.rs` holds 14 tests before and after, both readings rewritten in place, so
its six records, seven now, were not flagged. `wx_app.rs` is untouched: `git diff 37a44643..1c0e9b0b
-- src/presentation/wx_app.rs` is empty and its 73 records carry 199.

## What the gate selected

| File | On the branch |
|---|---|
| `.github/workflows/nvda.yml` | `all`, on task 1's commit: 74 result lines, 8,038 passed, the release build included |
| `nvda-tests/README.md`, `nvda-tests/tests/settings-tabs-read-once.test.js` | no scoped target, by the README's rule; read and syntax-checked |
| `src/presentation/status_line.rs` | `--lib presentation::status_line` on the red and the green |
| `src/presentation/wx_account_manager.rs` | `--lib presentation::wx_account_manager` on the red and the green |
| `guards/guards.toml`, `docs/changelog.md`, `.planning/*.md` | the whole-tree guards on every commit; the document-reading targets on task 3's documents-only commit |

`scripts/check.sh all` on the branch at `8c72e9b2`, its output to a file with the exit status
written by the same shell, never piped: exit 0, 8,039 passed and none failed over 74 result
lines, 362 s from 13:50:41Z to 13:56:43Z, the release build included; its last block says five
of CI's seven jobs. One more than 11-01's 8,038: the status line's third test. Inside the 275 s
to 654 s band on `docs/development/measurements.md`. `main`'s hook ran `all` again on the
merge: 8,039 and none failed, 340 s from 13:56:54Z to 14:02:34Z. The keyring race (ledger 374)
did not appear on any run.

## Guard records

913 records by the TOML reader before, 915 after; census 802 + 111 before, 802 + 113 after,
the line at `guards/guards.toml:84` moved in the green commit. One rewritten, two new, one
re-measured, all four in one `scripts/guards.sh --remeasure` run in the foreground with nothing
else building, `WIXEN_TEST_THREADS` untouched, the counts written by the runner; 7 min 19 s
from 13:36:05Z to 13:43:24Z, "All 4 guards redden exactly the tests their records name".

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| signing in failing on the account screen is said, not only shown (rewritten) | `wx_account_manager.rs` | the Failed arm's `shown_and_signalled` call replaced by a bare `status.set_label`, the silence the record was written against | 3, "all 3 tests named went red, and nothing else did": the two it always named and the reading of `reauthorize_selected` | rebuild 35 s, run 55 s |
| a sign-in failure on the account screen is one notification, not a sentence and a cue a moment apart (new) | `wx_account_manager.rs` | the Failed arm's call replaced by `said_and_shown` at High and a separate `a11y.signal(AccountNeedsAttention, &name)`, the tree before this plan | 2, "all 2 tests named went red, and nothing else did": the reading of `reauthorize_selected` and the two-failures reading | rebuild 41 s, run 58 s |
| an outcome that is also an event is shown and signalled from one call (new) | `status_line.rs` | `a11y.signal(event, said)` replaced by `(a11y, event)` discarded | 1, "the one test named went red, and nothing else did" | rebuild 40 s, run 57 s |
| every answer a window gives on a line of text is said out loud (re-measured) | `status_line.rs`, naming four files | unchanged, `announce` dropped from `said_and_shown` | 4, unchanged, "all 4 tests named went red, and nothing else did"; `status_line.rs` written at 3 | rebuild 40 s, run 58 s |

The first draft of each red list was a prediction and the runner's answer agreed with all
four. Counts written: `status_line.rs` 3 on two records; `wx_account_manager.rs` 14 on seven.
The count check printed its remedy once, at the red (named in that commit's trailer), and the
remedy was run before the green; it printed nothing at the green or after.

## What the tree contradicted

Every command in the plan's six premises was re-run against `main` at `37a44643` before
anything was built. All held: `continue-on-error: true` at `nvda.yml:39`, the two `if:
always()`, the arms at `wx_account_manager.rs:562-590`, `said_and_shown` at
`status_line.rs:19-27`, `signal` at `accessibility.rs:243-266`, the event's text at
`feedback.rs:203` and its priority at `:246`, the three signal sites and nowhere else, the
reading at `:2766` (the plan said `:2787`, which is inside it), 14 tests and 6 records on
`wx_account_manager.rs`, 2 tests and 1 record on `status_line.rs`, the four transcripts as the
plan quotes them, entries 489 and 492 open. Three things the plan did not have:

1. **A second reading in `wx_account_manager.rs` read the Failed arm and would have gone
   red.** `test_the_two_failures_on_this_screen_are_said_above_the_ordinary_run` required
   `said_and_shown(` and `Priority::High` near "Signing in failed:". The plan named only the
   reading at `:2787` for rewriting. Rewritten in place too (Rule 3), the sign-in half holding
   the new call, the event, and the event's own level above High; the count stays 14.
2. **The existing sign-in record's `before` was the very call task 2 deletes.** The plan
   asked for a new record on the account manager and did not say what became of "signing in
   failing on the account screen is said, not only shown", whose `before` is the
   `said_and_shown` call in the Failed arm; a record whose break no longer matches the file
   fails the run and says so. Rewritten onto the new call, its break kept, measured at 3.
3. **Ledger 494 is the README rewrite.** 09-06 recorded that the README's table lists two of
   five files and its prose says two; task 1's README rewrite is that entry's remedy, and it is
   closed with the two the plan named.

## Deviations from plan

**1. [Rule 3 - Blocking] The two-failures reading rewritten in place**, above. Found during
task 2's red. Without it the green would have failed on a reading the plan did not name.

**2. [Rule 3 - Blocking] The sign-in record rewritten onto the new call**, above. Found
during task 2's records. Without it `scripts/guards.sh` would refuse the record at the next
sweep as a break that names no place in the tree.

**3. [Decision] Ledger 494 closed**, above, with 489 and 492.

**4. [Decision] The two-failures reading's window**, changed in the green after the red's
window proved too narrow for the green code; red under either.

**5. [Decision] No stub in the red.** Both readings read text, so the helper's absence is an
assertion failure; the red commit holds tests only.

**6. [Decision] Two more comments in the settings case corrected.** The plan said "nothing
else changes"; the `beforeAll` comment and the header's last paragraph had claimed the opening
reading is in the log, which the run showed it is not, and a comment the run proved false is
the shape this project's own file warns about. Both now say what the run captured; no code
beyond the wait changed.

Everything else executed as written. **No scripted edit touched a tracked file: the exception
set for this plan is zero, and it stayed there.** Every tracked file was changed by Read then
Edit or Write; `cargo fmt` ran before each Rust commit, which is the project's formatter and
not a rewrite; the only `sed`, `awk`, `grep`, `tr` and `python` in the session read files and
logs; `git checkout` was not needed. Commit messages were written to the scratchpad and passed
with `-F`. Carriage returns measured with `tr -cd '\r' | wc -c` on every changed file before
each commit, the `.yml` and `.js` files included: zero on each. No em-dash in any file this
plan wrote, measured with `grep -c` for the byte sequence: zero on each. `git commit` and `git
merge`, never `gsd-tools query commit`; never `--no-verify`; `check.sh` never piped, its exit
status written to its own log by the shell that ran it. No AI attribution in any commit.
`nvda-tests/package-lock.json`, `Cargo.toml` and `Cargo.lock` untouched; no package or crate
added (T-11-SC). NVDA on this machine was not stopped, reconfigured or driven; the tester's
profile was not read; no binary was started; no `npm test` ran. Nothing pushed.

## Threat register

T-11-11 mitigated: `continue-on-error` gone from the NVDA job, `if: always()` on the summary
and the upload, the header dated. T-11-12 mitigated: one call per arm, held by the two readings
and three measured records. T-11-13 accepted, unchanged: the reason is `run_oauth_flow`'s own
sentence. T-11-14 accepted: the person's own Feedback row for the event, as phase 6 designed;
the line of text stays whatever the row says. T-11-SC: nothing added. No new surface outside
the register.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after, 16 passed. 530 before, 532 after; 500
open before, 499 after; 30 fixed before, 33 after.

| id | kind | what |
|---|---|---|
| 489 | closed | the walk over the five editors, run 35336142914, 0 without a name on each; the walk still crashes here (390) |
| 492 | closed | on the tester's word of 2026-09-18 (#33); the runner's case is 531 |
| 494 | closed | the README's table lists all five files; the count is a dated sentence and no reading holds it |
| 531 | unrun-verify | the corrected settings case's next run: which tab the first Right reaches on the runner's fresh profile |
| 532 | deviation | `accessibility.yml` keeps `continue-on-error`; its findings are counts and whether a count fails a run is the finding owner's decision |

## The issue

None to close or comment: #33 is closed on the tester's word already, and nothing was run
through `gh` after the merge beyond the read of #33's comments before task 3.

## Known stubs

None. `shown_and_signalled` has three non-test callers, the three arms of `reauthorize_selected`,
which `wire_account_manager_actions` wires to the Sign In Again button's `on_click`; the
runner's sign-in case presses that button on a synthesised account and is the non-test path
that reaches the `Failed` arm at the next push.

## Not done here, on purpose

Whether the one line "Sign-in needs attention, Signing in failed: ..." is heard whole is the
runner's next run or the tester's ear, and the changelog entry says nobody has heard it. Which
tab the corrected case's first Right reaches on a fresh profile is ledger 531. Whether the
Accessibility scan's counts should fail its run is ledger 532 and belongs to whoever owns the
findings. FOUND-18's `[S]` line and roadmap criterion 12's runner clause wait on the push. The
row is `2/15`. `wx_app.rs` is as it was, and no page owes this plan a sentence beyond the
changelog and the nvda-tests README.

## Self-Check: PASSED

`grep -c 'continue-on-error' .github/workflows/nvda.yml` is 0 and `grep -c 'if: always()'` is
2; `sed -n '1,/^#\[cfg(test)\]/p' src/presentation/status_line.rs | grep -c 'pub(crate) fn shown_and_signalled('`
is 1 (the whole file answers 2, because the reading's own sample quotes the signature);
`sed -n '1,/^#\[cfg(test)\]/p' src/presentation/wx_account_manager.rs | grep -v '^\s*//' | grep -c 'shown_and_signalled('`
is 3; `guards/guards.toml` holds 915 records by the TOML reader and the census line says 113;
`docs/changelog.md` holds the lines beginning "**The NVDA workflow fails when one of its cases
fails.**" and "**A sign-in failure in the Account Manager was two announcements a moment
apart"; `.planning/WINDOWS.md` holds 531 and 532 in both halves and 489, 492 and 494 fixed in
both; `.planning/REQUIREMENTS.md` has FOUND-08 and FOUND-09 ticked. Commits `af30aa0f`,
`eb2011ad`, `a697f61f`, `8c72e9b2` and `1c0e9b0b` are in `git log --oneline` on `main`.

# The planning files agree with themselves

Built 2026-09-07, after phase 4.2 completed. The check lives in
`tests/the_planning_files_agree_with_themselves.rs` and runs on every commit
that could break it.

## Why it exists

`.planning/STATE.md` holds the same facts twice, in frontmatter that tooling
parses and in a prose heading that people read. The two came apart three times
in one week, and the third time happened in a file that by then carried three
paragraphs explaining exactly that failure. `.planning/WINDOWS.md` holds every
ledger entry twice, as a markdown table row and as a JSON object, and an edit to
the table alone reverts on the next tool write. `.planning/ROADMAP.md` drifts
from disk instead of from itself.

Prose is not a check. That is the whole argument, and it is the argument
`CLAUDE.md` makes about every other rule here.

## What is asserted

Seven things, in six readings.

| # | The claim | Read from |
|---|-----------|-----------|
| 1 | `current_phase` names the phase the Current Position heading names | STATE.md, both halves |
| 2 | `current_plan` equals `Current Plan:` in the body | STATE.md, both halves |
| 3 | `Total Plans in Phase` equals the `*-PLAN.md` files in that phase's directory | STATE.md and disk |
| 4 | `progress.completed_plans` equals every `*-SUMMARY.md` across all phases | STATE.md and disk |
| 5 | Every ledger id in the table is in the JSON block, and the reverse | WINDOWS.md, both halves |
| 6 | For every id in both, the same status and the same description | WINDOWS.md, both halves |
| 7 | Every progress-table row's `n/m` matches the summaries and plans on disk | ROADMAP.md and disk |

Checks 5 and 6 are one reading, because both are "the two halves came apart".

Three things about how they are written.

**Phases are compared as numbers, not as bytes.** The same phase is spelled
`04.2` in a directory name and in the state frontmatter, and `4.2` in a roadmap
row. A check demanding one spelling would fire on a file that is right.

**A fact written twice is itself a finding.** Each reading requires exactly one
copy of the key it reads. None is a finding, and so is more than one. That is
not tidiness: STATE.md at `ccd0fdf` carried two `Phase:` lines in the same
section, which is how the heading described phase 02 while the frontmatter said
04.2, and nobody had characterised it that way before this check reported it.

**Every reading complains when it cannot find what it is comparing.** A missing
`current_plan`, a Current Position section with no `Phase:` line, a ledger with
no JSON block, a progress table that is not there: each is reported rather than
skipped. This matters more than the companions do. A check that reads nothing
and reports nothing is the exact failure `CLAUDE.md` describes in the paragraph
about a guard whose trigger is "a document mentions X", and structurally
refusing to be silent is a stronger defence than proving the reading works once.

## What each companion proves

One companion per reading, six in all. Each one runs the same function over the
real file's own lines with one violation spliced in, and requires the answer to
name exactly that violation: that file, and that value. A fixture built by hand
would have proved only that the extractor works on text the author wrote.

Before splicing, each companion asserts the real file produces no findings. That
is the vacuity check: if the file already disagreed with itself, the spliced
violation would not be the only thing the reading has to find, and the exact
comparison would be meaningless. This fired for real during the hand-red pass,
which is how it is known to work.

| Companion | What it puts in, and where |
|-----------|---------------------------|
| `test_the_phase_reading_can_see_a_disagreement` | `current_phase: 99.9` into STATE.md's own frontmatter; requires the finding to name 99.9 against whatever the real heading says |
| `test_the_plan_reading_can_see_a_disagreement` | `current_plan: 4242`; requires the finding to name 4242 against the real body value |
| `test_the_plan_count_reading_can_see_a_disagreement` | `Total Plans in Phase: 4242`; first asserts 4242 is not the real count, so the reading could not be right to stay quiet |
| `test_the_summary_count_reading_can_see_a_disagreement` | `completed_plans: 4242`; same guard against the number being right by accident |
| `test_the_ledger_reading_can_see_all_three_ways_the_halves_come_apart` | Three splices into the ledger's last real row: a renumbered id, a changed status, a changed description. Requires two findings for the first and one each for the others |
| `test_the_roadmap_reading_can_see_a_row_that_disagrees_with_the_disk` | A wrong `n/m` into the first real counted row |

Two more tests sit beside them.
`test_the_phase_directories_are_really_being_counted` asserts the tree walk
found directories, plans and summaries, so a walk that returned nothing cannot
leave three checks comparing numbers against zero and reporting agreement.
`test_a_phase_is_the_same_phase_however_it_is_spelled` proves the normalisation
on the three real spellings and on the three things that are not a phase.

## What was taken red by hand

Every check, one at a time, and the companions in both directions.

**Against the real historical divergences**, by checking the file out of git
over the working copy and running the check. This is stronger evidence than any
splice, because the file carried the defect as it actually happened.

| Commit | What the check said |
|--------|--------------------|
| `c6e08a9`, the 04.2-06 divergence | `the frontmatter says current_plan 6 and the body says Current Plan: 7`, which is verbatim the failure recorded by hand at the time |
| `ccd0fdf`, the phase-02 heading | `a Phase line in the Current Position section appears 2 times, on lines 33, 44`, plus `a current_plan in the frontmatter is missing` and `a Total Plans in Phase line in the Current Position section is missing`, which is exactly why `state.advance-plan` could not parse the file |
| `d0e5a3d2^`, 2026-09-01 | `a current_plan in the frontmatter is missing` and the same for `Total Plans in Phase`. The self-consistency checks cannot catch this one, because both halves agreed and both were wrong; what they do catch is that the fields the tooling needs were not there |

**Against the current files**, by editing each fact, running, and restoring.

| Break | What the check said |
|-------|--------------------|
| `current_phase: 04.2` to `03` | `the frontmatter says phase 03 and the Current Position heading says 04.2` |
| `current_plan: 9` to `8` | `the frontmatter says current_plan 8 and the body says Current Plan: 9` |
| `Total Plans in Phase: 9` to `10` | `Total Plans in Phase says 10 and phase 4.2 holds 9 *-PLAN.md files on disk` |
| `completed_plans: 59` to `58` | `progress.completed_plans says 58 and the phase directories hold 59 *-SUMMARY.md files` |
| Ledger entry 1 closed in the table only | `ledger 1 status: the table says fixed and the JSON block says open`, and the description difference beside it |
| Ledger entry 1's table row deleted | `ledger 1 is in the JSON block and has no table row` |
| The 4.2 roadmap row back to `8/9` | `row 4.2 What was built and never reached says 8/9 and phase 4.2 holds 9 summaries and 9 plans on disk` |

**And the wiring**, which was a genuine red in its own commit rather than a hand
break: `test_this_target_runs_on_the_commits_that_could_break_it` failed by
reporting the five targets a documents-only commit really ran, and went green
when the sixth was added to `scripts/check.sh`.

The companions were taken red the other way round, by the stub. In the RED
commit the six readings returned empty lists, all six companions failed and all
six checks passed, which is the asymmetry recorded in the commit message and
worth restating: **a check that asserts an absence cannot be red in its own RED
commit while the tree is correct.** The only committable red is the companion.

## What was found wrong while writing this

**`progress.total_plans` says 87 and there are 85 plan files, and what it counts
could not be established.** It is left unasserted for that reason. The roadmap's
progress denominators sum to 85. `deriveProgressFromRoadmap` in the vendored
tooling sums exactly those denominators, so that reading gives 85 too. The 87
arrived at `b998dfaf` on 2026-09-07, the commit whose message records
`state.advance-plan` being run by accident, over a roadmap whose denominators
already summed to 85. A second writer in `state-transition.cjs` sets the field
to whatever an argument passes it. So there are at least two writers with
different meanings and an observed value matching neither reading. Ledger 176.

**The em-dash rule does not reach `.planning` at all.** `ours()` in
`tests/house_style.rs` collects `src`, `docs`, `tests`, `scripts`, `guards`,
`installer` and `.github`, plus five named files. `.planning` is in none of
them. Measured: 699 em and en dashes across `.planning/**/*.md` today, with
`house_style` green. `scripts/which-checks.sh` carries a comment saying the
em-dash guard caught a real break "in a planning file", which now reads as
though `.planning` were covered. Either that meant a document under `docs/`, or
the coverage changed. Not fixed, because widening the guard to `.planning` would
redden 699 places at once and that is a decision, not a cleanup.

**The divergence is not recorded where I was pointed.**
`04.2-07-SUMMARY.md` does not mention it. The record is in `STATE.md`'s own
`last_activity` field: "04.2-06 landed at c6e08a9 and updated this file's
Current Position heading without updating the frontmatter". Worth knowing,
because it means the record of a state divergence lives inside the file that
diverged.

**A hand break left a neighbouring check green by coincidence.** Setting
`current_phase` to `03` reddened check 1 as intended and left check 3 green,
because phase 3 also holds nine plans. The check was reading the wrong directory
and getting the right answer. This is the trap the brief warned about, at 9
rather than at 1, and in real data rather than in a fixture. It does not make
check 3 wrong; it does mean a green under that break said nothing.

**The 2.1 roadmap row carries a second `n/m` in its Status cell**: `9/9` in
Plans Complete and `(12/13)` in the status text. The reading takes the column by
header name, so it is unaffected, but a positional reader would take the wrong
one. Recorded here because the vendored tooling's own comment says it used to be
positional.

## What is not held, and is written down instead

Four ledger entries, 175 to 178.

- **175.** The roadmap check goes one direction only. A phase directory with
  plans and no row at all is not seen.
- **176.** `progress.total_plans`, above.
- **177.** The ledger check compares two of the ten columns each entry carries
  twice. The other eight can still be edited in the table alone.
- **178.** The state check holds four facts. `current_phase_name`, `status`,
  `stopped_at` and `state_head` are also written twice and are unheld.
  `stopped_at` is the one that already went wrong.

## Where it lives and why

A new integration target, not `tests/house_style.rs` and not `tests/wired.rs`.
Those two are named by 18 and by 15 or 16 guard records in their
`tests_last_seen` counts, so adding tests to either puts a count-keyed
re-measurement run on the critical path. This file is named by no record, so
nothing is owed. `house_style` stayed at 64 tests and `wired` at 69, both
measured after the last test landed rather than after the first green.

It is in both of `scripts/check.sh`'s target lists, because
`scripts/which-checks.sh` answers two different things about the two ways a
planning file gets committed. Alone it is `docs_only`, so the target joins the
five that read documents. Beside code it is `affected`, so it joins
`house_style` and `wired` at the end of every scoped run. A plan summary lands
the second way and a hand correction to `STATE.md` lands the first. Without both,
this would be a guard that runs on every commit except the ones that break it,
which is the failure `CLAUDE.md` names twice.

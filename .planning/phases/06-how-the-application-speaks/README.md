# Phase 6: How the application speaks

Eight plans, one per wave. Assembled 2026-09-12 against `main` at `b8857bc8`,
version `0.116.0`, `guards/guards.toml` holding 729 records, the library holding
7,075 tests, `.planning/WINDOWS.md` at entry 324 with 302 open. Nothing in the
repository was changed while these were written, except that one `cargo test`
was run to check a command these plans use, which wrote only to `target/`.

**Read against `06-RESEARCH.md`, written the same day against `febe8e4`.** That
document is the primary source and most of it survived being used. Where a claim
in it turned out wrong when a plan went to lean on it, the plan carries the
correction in its `<premise_corrections>` block and this README lists the three
that change what gets built. `06-RESEARCH.md` measured 720 guard records and
version `0.112.0`; four version bumps and nine records arrived between its read
and this one, all from phase 7.

**Goal.** The user controls what is spoken, brailled, sounded and shown, reads
dates in their own language, and the project knows which parts of WCAG its scans
can and cannot judge.

**Requirements:** FEEDBACK-01, FEEDBACK-02, FEEDBACK-03.

**Roadmap success criteria this phase owns:** all four, plus the two items
inherited from phase 1.

## The plans

| Plan | Wave | Criterion | Depends on | Human | What it does |
|---|---|---|---|---|---|
| 06-01 | 1 | 1 | none | no | **Done.** The model can say what somebody chose, separately from what they get, and a seventeenth event cannot exist without a control |
| 06-02 | 2 | 1 | 06-01 | decision, answered | **Done.** Sixteen events, three answers each, on the Feedback tab, and the two global boxes that cannot mean what they say are one that can. Criterion 1 has five clauses, four close structurally, none is heard |
| 06-03 | 3 | 2 | none | decision | Month and day names come from Windows, through `GetDateFormatEx`, at all four shipping sites |
| 06-04 | 4 | inherited A | none | decision | A permission for one account gets a screen, and the list that recorded it as unreachable empties without disarming its own guard |
| 06-05 | 5 | inherited B | none | decision | A reminder and somebody who is typing |
| 06-06 | 6 | 3 | none | decision | The scan is reproducible, and a change to it earns the checks that could catch it |
| 06-07 | 7 | 3 | 06-06 | decision | "Roughly half of WCAG" becomes a list of fifty-five, and the reading that checks the list is shown a planted violation |
| 06-08 | 8 | 4 | 06-06, 06-07 | action | The findings the scan really produces, judged one at a time, and the list only a person can walk |

Requirement coverage: FEEDBACK-01 by 06-01, 06-02 and 06-04. FEEDBACK-02 by
06-03. FEEDBACK-03 by 06-06, 06-07 and 06-08.

## Why eight plans, and why one per wave

**Eight because the phase is eight nearly independent pieces**, and because each
fits in two or three tasks without a task touching more than five files. The
research counted four pieces; it is eight once the two inherited items are
separated from the criteria they attach to, the per-event model is separated
from the panel that uses it, and criterion 3's workflow half is separated from
its document half.

**One per wave because every plan here writes `guards/guards.toml`.** That is
the same reason phase 5.1 ran six plans in six waves, and the arithmetic is the
same. Seven of the eight also write `docs/changelog.md`. A wave is a set of
plans sharing no file, and no two plans here can satisfy that.

Why every plan writes the guard records, stated per plan rather than asserted,
counted 2026-09-12 at `b8857bc8` by parsing `tests_last_seen` blocks rather than
by grepping a file name:

| Plan | The file it adds a test to | Records fingerprinting it | Tests there today |
|---|---|---|---|
| 06-01 | `src/presentation/accessibility/feedback.rs` | 1 | 41 |
| 06-02 | `tests/checkbox_labels.rs`, plus a new file under `tests/` | 1, plus one new record | 1 |
| 06-02, as it turned out | `feedback.rs` gained 2 tests, `tests/every_event_has_a_control.rs` is new at 1, `tests/checkbox_labels.rs` stayed at 1 | 2 flagged and remeasured, 5 written | 49, 1, 1 |
| 06-03 | `src/presentation/date_display.rs` | 3 | 37 |
| 06-04 | `src/data/config.rs`, `tests/house_style.rs` | 5 and 19 | 63 and 69 |
| 06-05 | `src/presentation/one_question_at_a_time.rs` | 1 | 19 |
| 06-06 | `src/presentation/scan_target.rs`, `scripts/which-checks.test.sh` | 0 and 0 | 6 and n/a |
| 06-07 | wherever the coverage reading is homed | to be decided in the plan | varies |
| 06-08 | `.planning/WINDOWS.md`, documents | 0 | n/a |

Take the command from `CLAUDE.md` rather than a grep. A grep for `config.rs`
across `guards/guards.toml` answers far more than five, because a file name
appears in a record's `file`, its `before`, its `after`, its `red` list and its
prose as well as in `tests_last_seen`.

```bash
awk -v target=src/data/config.rs \
  '/^\[\[guard\]\]/ {if(n>0) c++; n=0}
   /^tests_last_seen/ {b=1; next} b && /^\]/ {b=0; next}
   b && /file *=/ && index($0, "\"" target "\"")>0 {n++}
   END {if(n>0) c++; print c+0}' guards/guards.toml
```

**Every record naming a file this phase touches was in step with the tree on
2026-09-12.** Checked by comparing each record's written count against a count
taken by the check's own rule, which counts lines that are exactly `#[test]` or
`#[tokio::test]` after trimming. `config.rs` 63/63, `wx_settings.rs` 0/0,
`feedback.rs` 41/41, `date_display.rs` 37/37, `occurrences.rs` 62/62,
`one_question_at_a_time.rs` 19/19, `house_style.rs` 69/69,
`checkbox_labels.rs` 1/1. So this phase starts from a clean baseline, and the
first commit adding a test to any of them will flag its records and print the
`scripts/guards.sh --remeasure` command. Running that when a commit prints it is
not optional.

**Do not use a naive grep to take those test counts either.** `grep -c '#\[test\]'`
answers 76 for `tests/house_style.rs` against the check's 69, because that file
quotes the attribute inside string literals.

## The collision with phase 7, which shrank to almost nothing while this was written

`06-RESEARCH.md` calls the shared use of `src/data/config.rs` "the single largest
risk to this phase". **It is now measured gone, and it went two ways.**

**Plan 07-05 merged at `d83bed3` and changed nothing inside
`mod every_setting_is_acted_on`.** Its own summary says so in a section addressed
to this planner: not one of the four tests, not `stored_setting_names`, not
`files_that_act`, and not one of the three exception lists.
`OFFERED_BY_ANOTHER_SCREEN` still holds three entries,
`NOT_ANYTHING_ANYBODY_CHOOSES` two and `STORED_AND_OFFERED_BY_NOTHING` one.
Verified here rather than taken from the summary, 2026-09-12 at `b8857bc8`:

```bash
grep -n 'const OFFERED_BY_ANOTHER_SCREEN\|const NOT_ANYTHING_ANYBODY_CHOOSES\|const STORED_AND_OFFERED_BY_NOTHING' src/data/config.rs
```

**Phase 7's four remaining plans name neither file.** Read out of their own
frontmatter, which is the half a same-wave overlap check reads:

```bash
for p in 06 07 08 09; do
  awk '/^files_modified:/{f=1;next} f&&/^[a-z_]+:/{f=0} f' \
    .planning/phases/07-installing-updating-and-what-is-stored/07-$p-PLAN.md
done
```

Neither `src/data/config.rs` nor `src/presentation/wx_settings.rs` appears. The
only mention of either anywhere in those four documents is one line of 07-09
quoting a guard-record cost figure, not a file it edits.

**What is left of the collision, and it is real.**

| File | Phase 6 plan | Phase 7 plan |
|---|---|---|
| `src/presentation/wx_app.rs` | 06-05 | 07-09 |
| `tests/house_style.rs` | 06-04, possibly 06-07 | 07-06 |
| `guards/guards.toml` | 06-01 to 06-08 | 07-07, 07-08, 07-09 |
| `docs/changelog.md` | seven of the eight | all four |
| `.planning/WINDOWS.md` | 06-08, and each plan's own entries | all four |

`.planning/WINDOWS.md` was being written by a phase 7 executor while this README
was written: the file moved from 322 entries to 324 between two reads twenty
minutes apart. Add entries through `gsd-tools windows` and never by typing into
the table, because the ledger has a JSON half and
`test_both_halves_of_the_ledger_say_the_same_thing` holds the two together.

**What to re-take rather than re-read, if phase 7 lands more before this phase
starts.** Every line number these plans give for `src/presentation/wx_app.rs` and
`tests/house_style.rs`, the record count for those two files, and
`Cargo.toml`'s version. Nothing in criterion 1's model, criterion 2's dates or
criterion 3's scan appears in phase 7's plans at all.

**The decision that remains is scheduling and it is Pratik's**, carried at a
checkpoint in 06-04. It is much narrower than the research's version of it.

## Three things in the research that were wrong when a plan went to use them

Listed here as well as in each plan, because each would have become a false
premise.

**1. The suggested shape for the `Event::ALL` guard does not close the hole.**
The research says the missing guard "can be written as an exhaustive match from
each variant to a unit, or by asserting `ALL.len()` against a count derived from
the source". The first of those cannot work. An exhaustive match forces a new
variant to be answered, but nothing can call that match for a variant that is
not in `ALL`, because `ALL` is the only enumeration of the variants that exists.
A seventeenth variant given an arm in a fourth match is exactly as invisible as
one given an arm in the three that already exist. Only two mechanisms really
close it: generate the enum and `ALL` from one list, which makes the disagreement
unrepresentable, or read the source and count, which makes it detectable and
needs a companion proving the reading can see a violation. 06-01 takes the first,
on the `menu_ids!` precedent at `src/presentation/wx_app.rs:74`, whose own doc
comment gives the same argument about a numbering nobody should be doing by hand.

**2. The recommended panel has four checkboxes and the criterion asks for
three.** The research recommends "a Choice listing the sixteen events by
`Event::text()`, four checkboxes below it". Criterion 1 says Speech and Braille
"are set **together**, as one choice". Four boxes would rebuild, per event, the
exact false control the criterion was rewritten to remove. 06-02 builds three.

**3. The research does not notice that the global pair is the same defect,
today, on the same tab.** `build_feedback_tab` iterates `Channel::ALL` and builds
one checkbox per channel, so the Feedback tab offers Speech and Braille as two
independent boxes. Verified 2026-09-12 at `src/presentation/wx_settings.rs:2160-2169`.
Two boxes, four states, and only two outcomes, because `src/presentation/accessibility.rs:247-255`
releases the notification when either is on. The roadmap already assigns this
here: "Correcting those three false sentences belongs to phase 4.2; offering the
honest control belongs here." Phase 4.2 corrected the changelog sentences and
left the two boxes standing, which is checkable:

```bash
grep -n 'for channel in Channel::ALL' src/presentation/wx_settings.rs
```

So 06-02 owns the global pair as well as the per-event controls, and the phase is
larger than the research says by one task.

## What Pratik is asked, and where

Nine decisions, one checkpoint each, each in front of the evidence at the moment
it is needed. None is answered here and none is answered in a plan.

| Decision | Plan | What it changes |
|---|---|---|
| 1. What relative wording does in a non-English locale | 06-03 | Whether "2 days ago" is kept English, replaced by the date, or given real plural rules and a new dependency |
| ~~2. The panel's shape, against the comment that argues for no panel~~ | 06-02 | **Answered 2026-09-12: option 1, a Choice with three controls and a reset button beneath it.** Not option 4's extra row setting one answer for everything, whose state is ambiguous when the sixteen disagree; it costs one row to add later if a listening pass says the sixteen trips are the real problem |
| 3. Whether a per-account Allow Changes answer is three or one | 06-04 | `Allowed` has three fields and `allowed_for` can only narrow |
| 4. Whether a reminder waits for typing to stop | 06-05 | Wait, raise without focus, or hold briefly and raise anyway |
| 5. Whether to widen the scan's target list | 06-06 | Eleven windows are scanned and at least nine more dialogs exist |
| 6. Whether to pin the Axe.Windows CLI | 06-06 | Whether the coverage list is a claim about something reproducible |
| 7. Whether `REQUIREMENTS.md` is corrected in place | 06-07 | Three of its evidence lines are wrong and this phase disproves one of its figures |
| 8. Whether the coverage list is a document or a check | 06-07 | A document goes stale the way the five findings did; a check costs records and needs a companion |
| 9. How phases 6 and 7 share the tree | 06-04 | Narrowed by measurement to `wx_app.rs` and `tests/house_style.rs` |

Each checkpoint carries a recommendation rather than a menu, and says what each
option costs. A decision put without a recommendation is a list, not a question.

## What no plan in this phase can close

Ten things. Each becomes one `unrun-verify` entry in `.planning/WINDOWS.md`,
written by the plan that meets it, one entry per unrun thing rather than one
entry for all of them. `.planning/WINDOWS.md` stood at 324 entries with 302 open
when this was written, and a phase 7 executor was adding to it at the time, so
read its last row rather than that number.

1. **Whether the per-event panel is usable.** Whether a Choice of sixteen with
   three controls beneath it reads well; whether the controls reloading under the
   cursor when the Choice changes is announced or silent. That is a live-region
   shaped problem and only a listening pass settles it.
2. **Whether the effective-versus-chosen distinction is understood.** The
   sound-only fallback means the panel explains something subtle. No test can
   tell whether the explanation lands.
3. **Whether the new controls are named on both channels in practice.** A test
   can prove a label was set. Only Narrator proves UI Automation reads it and
   only NVDA proves MSAA does.
4. **Whether the honest global control reads as a loss.** Somebody who ticked
   Braille and not Speech meets one box where there were two. Whether that reads
   as an explanation or as a feature being taken away is a person's judgement.
5. **Whether localised dates sound right.** A French month read by a French voice
   in a date whose order came from the same machine is what FEEDBACK-02 exists
   for. It needs a French Windows, a French NVDA voice and somebody who speaks
   French.
6. **Whether the genitive really arrives.** Microsoft documents that it does.
   Nothing here has ever called `GetDateFormatEx` on a Russian or Polish locale.
7. **Whether the sixteen earcons are distinguishable.** Already disclosed in the
   product, still unmeasured, and a per-event panel makes it more likely somebody
   switches sounds on for all sixteen.
8. **The findings the scan produces.** Each has to be looked at and judged: ours,
   or upstream with a named issue. The scan produces the artifact; the judgement
   is a person's, and 06-08's checkpoint is where the artifact arrives.
9. **The scoped manual list.** Criterion 4 asks for the list of interactions only
   a human pass can walk. That is a judgement about what matters, not a
   derivation.
10. **Every WCAG criterion an automated scan cannot judge, which is fifty-two of
    the fifty-five.** The coverage list 06-07 writes is the record that this is
    so. Writing it down is not testing it.

**Pratik does the manual and screen reader testing after phase 8.** So these are
planned to be recorded rather than performed. Criterion 4's second clause, "each
of the five WebView2 findings is either fixed or recorded as upstream with the
upstream named", is the one criterion here whose closure needs a CI run that a
person triggers, and 06-08 carries a `checkpoint:human-action` for exactly that
rather than pretending an executor can do it.

## Costs every plan is written around

**Guard records: 729 on 2026-09-12 at `b8857bc8`.** This was 720 on the same day
at `febe8e4`, and 632 when phase 5.1 was planned. Re-take the count rather than
quoting it, and count records rather than mentions.

**The per-file figures, taken the same way on the same day.**

| file | records | `#[test]` today |
|---|---|---|
| `src/presentation/wx_app.rs` | 48 | not counted, see below |
| `tests/house_style.rs` | 19 | 69 |
| `src/presentation/wx_account_manager.rs` | 6 | 13 |
| `src/data/config.rs` | 5 | 63 |
| `src/presentation/ui_types.rs` | 4 | 78 |
| `src/presentation/wx_settings.rs` | 3 | 0 |
| `src/presentation/date_display.rs` | 3 | 37 |
| `src/application/occurrences.rs` | 3 | 62 |
| `src/service/signed_mail.rs` | 2 | 112 |
| `src/presentation/accessibility/feedback.rs` | 1 | 41 |
| `src/presentation/one_question_at_a_time.rs` | 1 | 19 |
| `src/presentation/wx_item_form.rs` | 1 | 14 |
| `tests/checkbox_labels.rs` | 1 | 1 |
| `src/presentation/scan_target.rs` | 0 | 6 |
| `src/presentation/accessibility/sound_scheme.rs` | 0 | not counted |
| `tests/theme_reach.rs` | 0 | 3 |
| any new file under `tests/` | 0, and needs one written | 0 |

**Where a test goes is a cost decision here, and two plans turn on it.** 06-05
puts its rule in `src/presentation/one_question_at_a_time.rs` at 1 record rather
than in `src/presentation/wx_app.rs`'s test module at 48, which is why
`what_to_raise` takes its conditions as arguments. 06-02 puts its live check in a
new file under `tests/` at 0 records rather than in
`src/presentation/wx_settings.rs`, which has 3 records and no tests at all, so
the first test there newly couples three records to a count that will then move
on every later commit.

**A companion written into a file no record fingerprints is the technique 07-05
found and it is copied here.** That plan homed its hand-named settings companion
in `src/service/update_check.rs` for that reason. 06-02 and 06-07 do the same.

**Land every test for one file in one commit.** The count check fires per file,
not per test, so four tests in `config.rs` cost the same remeasure as one.

**Guard sweeps are not on any plan's critical path.** By the decision of
2026-09-03, one sweep once every phase is complete, and phase 8's criterion 5
owns it. No executor here runs `scripts/guards.sh` except the scoped
`--remeasure` a commit prints. **No plan here quotes a sweep duration**, for the
reason `CLAUDE.md` gives at length: the record count and the per-record rate both
move, the rate roughly halved on 2026-09-09, and the whole-tree total has been
quoted at 15, 16 and 20 hours at different counts.

**`cargo test` takes one `--lib`, and the form these plans use was checked rather
than copied.** `CLAUDE.md` records that 55 plans told executors to run
`cargo test --lib a:: --lib b::`, which cargo refuses, and that none of them was
ever run. The form used here is one `--lib` with several filters after `--`:

```bash
cargo test --lib -- presentation::accessibility::feedback:: presentation::date_display::
```

Run 2026-09-12 at `b8857bc8` against two real test paths, it compiled once and
ran both, reporting three tests passed and 7,075 filtered out. The `&&`-joined
form `CLAUDE.md` gives is also correct and costs a second build.

**The library holds 7,075 tests** as of that run. `CLAUDE.md` quotes 6,720 at
`10effea` on 2026-09-09 and 5,837 on 2026-08-30. Take it again rather than
quoting any of the three.

**No plan here touches `Cargo.toml` except for a version bump beside the change
that needs it.** Nothing in this phase adds a dependency, so
`scripts/which-checks.sh` answers `affected` on every source commit and
`docs_only` on the document commits. One exception is live: if decision 1 goes to
real plural rules, that is a new crate, a `dependency-audit` conversation and a
`Package Legitimacy Audit` that `06-RESEARCH.md` explicitly does not carry. The
checkpoint in 06-03 says so.

**A commit touching `.github/workflows/*.yml` answers `affected` and selects no
tests at all.** Verified 2026-09-12 by reading `scripts/which-checks.sh:325-338`
and `scripts/check.sh:387-432`: the first answers `affected` for anything that is
not `*.md` or `*.txt`, and the second maps only `src/*.rs` and `tests/*.rs` to
targets. So the two tests in `src/presentation/scan_target.rs` that read
`.github/workflows/accessibility.yml` do not run on a commit that changes it.
06-06 closes that, and it is the gate this phase would otherwise walk into twice.

**A red commit is allowed only on a branch**, must name every failing test in
`Fails-until-green:` trailers at column 0, and is held to three things at once:
every named test ran, every named test failed, nothing else failed. Where adding
a test makes `test_every_guard_record_says_how_many_tests_the_files_it_names_held`
red at the same time, name that check as one of the failures, in the same commit,
with the reason in the message.

**Never pipe `scripts/check.sh` into anything whose exit status is then read.**
Run `scripts/check.sh all` once before a merge, and do not assume the hook
already ran it.

**No em dashes anywhere under `.planning` or `docs`, and no carriage returns
anywhere this project writes.** `tests/house_style.rs` walks `.planning` for
markdown since 2026-09-07, and its carriage-return check reads the same set.
There were zero em dashes under `.planning` before these files were written and
there are none in them. Checked by byte search rather than by eye: zero `\r`,
zero em dashes and zero en dashes across all nine files.

**These eight files cannot be committed until the roadmap's progress row for
phase 6 says `0/8`.** That is a correction to what an earlier draft of this
README said, and it was caught by running the check rather than by reading it.

```bash
cargo test --no-fail-fast --test house_style --test the_planning_files_agree_with_themselves
```

Run 2026-09-12 at `b8857bc8` with the eight plans on disk: 14 passed, 2 failed,
both of them `the_planning_files_agree_with_themselves`, and both about one row.
`test_the_roadmap_counts_the_files_that_are_on_disk` reads the progress table in
`.planning/ROADMAP.md` against the phase directories, and
`test_the_roadmap_reading_can_see_a_row_that_disagrees_with_the_disk` is its
companion, which refuses to run while the tree already disagrees and says so in
its own message. That target is one of the four whole-tree guards `scripts/check.sh`
runs on every commit in every mode, so this is a blocking failure and not a
warning.

What an earlier draft of this README checked was `.planning/STATE.md`, and it was
right about that: `Total Plans in Phase` is read for the phase the frontmatter
names, which is 7, and `completed_plans` counts `*-SUMMARY.md`, which these files
do not add. It then concluded that committing was safe, having read two of the
checks in that file and not the third. The roadmap row is the third.

**The row was already moving while these plans were being written, which is worth
seeing.** It reads today:

> | 6. How the application speaks | 0/3 | Planning in flight | Not `TBD` any more,
> because plans are on disk and the check that reads this row treats `TBD` as
> honest only while a phase has none. The 3 is the plan files present on
> 2026-09-12 while phase 6's planning was still running, and it had already moved
> twice while 07-06 tried to commit past it, so it belongs to whoever is doing
> that work rather than to this number

So a phase 7 executor met this the hard way, twice, and wrote a number that was
correct for the minute it was written. `0/8` is the answer once these eight files
are final. Nothing in this phase edits `.planning/ROADMAP.md`, by instruction,
so it belongs to whoever commits these.

## Estimates, and why the numbers are small

`raw_tokens` is 30,000 per work task, which is the projection shape phase 7's
plans used. `tokens` is that multiplied by **0.15**, which is the mean of
`actuals.tokens / estimate.raw_tokens` over phase 7's five landed plans: 0.128,
0.102, 0.134, 0.135, 0.228. Read 2026-09-12 from each plan's `estimate` block and
each summary's `actuals` block.

**Phase 7's own estimates were between four and ten times too high**, and the
factor those plans used was about 1.2 rather than a measured one. Nothing was
wrong with the projections as projections; the factor was never calibrated
against an actual. Five samples spanning 2.2 times is `low` confidence, not
`med`, and the number is derived from the sample count rather than self-rated.

Checkpoints are not counted as tasks in the projection, because a checkpoint is a
block that waits rather than a block that works.

Re-derive the factor once this phase has actuals of its own. Five samples from
one phase is a thin basis and the plans say so.

## What is owed to documents, and belongs to whoever lands these

1. **The roadmap's progress row for phase 6 has to say `0/8`, and the gate is red
   until it does.** This is the blocking one. See the paragraph above for the
   measurement and for the correction to what this README first said about it.
   Not written here on purpose: `.planning/ROADMAP.md` and `.planning/STATE.md`
   are being edited by phase 7's executors in parallel and were moving during
   this work.
2. **The roadmap's phase 6 entry**, which today says `**Plans**: TBD`. It wants
   the count and the eight-line plan list, in the shape phase 7's entry uses.
   Nothing checks that line, so it is owed rather than blocking.
3. **`.planning/STATE.md`'s `progress.total_plans`** says 92 and there are 89
   `*-PLAN.md` files on disk today, which these eight will make 97. Nothing
   checks that field against the tree: `Total Plans in Phase` is read for the
   phase the frontmatter names, which is 7, and `completed_plans` counts
   `*-SUMMARY.md`, which these files do not add. Checked rather than assumed, by
   reading `tests/the_planning_files_agree_with_themselves.rs:343-413` and then
   by running the target. The field is still wrong and whoever owns the merge
   should correct it by counting rather than incrementing.
4. **Nothing in this phase corrects a requirement ahead of the code.** Where a
   requirement wants rewording after a plan makes it true, the plan that made it
   true does the rewording. Decision 7 in 06-07 is about the requirement evidence
   lines that are wrong about the code as it stands today, which is a different
   question.

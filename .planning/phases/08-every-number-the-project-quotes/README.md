# Phase 8: Every number the project quotes

Nine plans, one per wave, written 2026-09-14 against `main` at `b14d6379`,
version `0.124.0`, `guards/guards.toml` holding 783 records by a TOML reader,
`.planning/WINDOWS.md` at entry 441, all seven CI jobs green on the push of
2026-09-14. Phase 6 is complete and merged; phase 7 has nine summaries with
three of them `partial` on human checkpoints. This is the last phase of the
milestone and the first with no phase after it.

**Read against `08-RESEARCH.md`, taken 2026-09-12 and 2026-09-13 at
`a0b909e8`.** That document is the primary source, and its own instruction is
the rule these plans follow: every figure in it was a guess until the command
beside it was run again. Every count it gives was re-taken on 2026-09-14 and
the ones that moved are in the table below. Its four-kind taxonomy, its list
of machinery to extend, and its headline finding that the prescribed counting
command miscounts all held.

**Goal.** Replace the estimates with measurements, so no figure in the
documents is both aspirational and undated.

**Requirements:** PERF-01 to PERF-07.

**Roadmap success criteria this phase owns:** all six.

## The plans

| Plan | Wave | Criterion | Depends on | Human | What it does |
|---|---|---|---|---|---|
| 08-01 | 1 | 3, 4, 5 | none | no | Every count and rate the phase depends on, taken today: the mutant count, the guard record count, the sweep rate from one record timed, the suite figures for the mutation timeouts. One page, `docs/development/measurements.md`, holds them, and a reading refuses a row without its command, date and commit |
| 08-02 | 2 | 3 | 08-01 | no | The checks: a count on a product page carries a date and a source; the three test-count pages quote one row; the red/green ratio is computed and printed instead of written; twelve prose figures are held to the constants they restate |
| 08-03 | 3 | 1 | 08-01 | no | The instrument for memory and cold start: a usable line the application writes once, a profile of exactly 1,000 messages of a defined shape, a harness that starts the release binary and reads the working set of the process and its WebView2 tree; the three numbers taken |
| 08-04 | 4 | 2 | 08-01 | no | The list over 200,000 rows: sort, filter and scroll each timed by a harness that stays in the tree; the virtual text callback's body pulled into a function over slices, with a reading that holds it to naming no database |
| 08-05 | 5 | 3 | 08-01, 08-02 | no | Coverage re-measured with the same command as the 2026-07-26 figure, and the low areas attributed to the untested transport with their own figures rather than raised |
| 08-06 | 6 | 3 | 08-01 to 08-05 | no | Corrected by hand what no reading reaches: the four sweep-cost figures retired, the awk replaced by the parser, comments in source and scripts, the three planning documents with dated figures |
| 08-07 | 7 | 5 | 08-01 to 08-06 | action | The one guard sweep of the milestone: a resumable runner that waits for a quiet machine and refuses a tree a kill left broken; a person starts it in a worktree; the log is read after it is complete and every short record is corrected and re-measured |
| 08-08 | 8 | 4 | 08-01, 08-07 | decision, then action | The whole-tree mutation run: shards on one commit that a person starts and the launcher resumes; the rate measured on one shard under both suite shapes; the products put to Pratik with the options; the report read after the last process exits, survivors killed or reasoned |
| 08-09 | 9 | 6 | all | no | Each target met or revised with the reason beside it; the seven requirements and six criteria closed clause by clause; the ledger told what this machine cannot measure; the person told what waits after the last phase |

Requirement coverage: PERF-01, PERF-02 and PERF-04 by 08-03; PERF-03 by
08-04; PERF-05 by 08-05; PERF-06 by 08-01, 08-02 and 08-06; PERF-07 by 08-01
and 08-08. 08-07 carries PERF-06 in its frontmatter because the sweep is
criterion 5 and no requirement names it; 08-09 carries all seven because it
closes them.

## Why nine plans, and why one per wave

Nine because the phase is nine pieces that do not share a shape: a page and
its reading, four checks, two instruments, one re-measurement, one correction
pass, two long jobs, and a closing read. Each fits in two or three tasks. The
two long jobs are each three parts on purpose, a launcher, a checkpoint where a
person starts it, and a reading after the log is complete, because neither can
be sat through by an agent and both have already produced wrong conclusions
here when read early.

One per wave because eight of the nine write `docs/development/measurements.md` and six
write `guards/guards.toml`, and a wave is a set of plans sharing no file. The
order is also real: the page comes first so every later plan has somewhere to
put a row and a reading that refuses a bare one; the checks come second so the
numbers the instruments write are held from the day they land; the corrections
by hand come after the checks have named what they can name; the sweep comes
after the last plan that adds a test, because a sweep over a tree that then
changes is stale at once; the mutation run comes after the sweep because
neither may run beside the other; the closing read comes last.

The mutation run may outlast the phase. 08-09 closes on whatever has completed
and says what is still running, with the date it started. Nothing waits for it.

## The five assumptions these plans are written under

The research put six decisions to Pratik. He was asked the five still open in
the same turn this README was written, with recommendations. Each plan that
depends on one says so at the place it depends, and the dependent part is
separable, so a different answer changes one plan and not the phase. He tends
to answer with a combination.

**Assumption 1, the guard sweep.** A resume flag is added to `scripts/guards.sh`
first, test-first; the sweep runs in a worktree at a fixed commit, unattended,
started and stopped by Pratik, with the log as the record; the reading of what
it finds is its own task after the log is complete. Touches 08-01 (the rate
timed and the product written), 08-07 (all three tasks), and 08-08's
launcher, which takes the same shape: a run a person starts in a worktree,
resumed at shard granularity, waiting for a quiet machine. A different
answer changes 08-07's task 1 and its checkpoint and the launcher half of
08-08's task 1; the rate rows and the reading tasks stay.

**Assumption 2, what counts as documentation for criterion 3.** `docs/`,
`CLAUDE.md` and `README.md` are read by a check; `.planning/REQUIREMENTS.md`,
`.planning/PROJECT.md` and `.planning/STATE.md` are corrected by hand with
each figure dated; everything under `.planning/phases/` and `.planning/intel/`
and every audit is a record and is not corrected. The precedent is
`.planning/writing-checks-widened.md`. Touches 08-02 (the walk list) and 08-06
(task 3). A wider answer widens one list in 08-02 and one task in 08-06.

**Assumption 3, the filter number.** 200,000 rows written into a `tempfile`
cache and `search_messages` timed, no window, the number recorded with the
machine and the build. Touches 08-04 task 3 only. Dropping the filter number
removes five rows from that task and adds a sentence saying why.

**Assumption 4, the no-query check.** The type-level version is the check: the
callback's body is a function whose inputs are slices and copies, so it cannot
reach the connection the program holds. It cannot be a guard record, because
its break is a compile error and the runner measures breaks by which tests go
red, and 08-04 says so where the record would have been. The planner found
that a companion source reading adds something the type does not: a function
whose inputs hold no connection can still open one from a path, and the
reading holds the module and the closure to naming no database. That half is
recordable and is recorded. There are two virtual text callbacks in
`wx_app.rs`, the message list's at `:1222` and the five PIM lists' one
closure at `:1572`; PERF-03 is about the message list, the reading anchors
on it and counts both, and the PIM closure is held to naming no database
only, because that costs one line. Touches 08-04 task 1 only.

**Assumption 5, "red/green started at commit 182 of 344".** A check computes
the share of the history before the commit that sentence called number 182,
prints it with the date, and the four tree sites name the check, while the
two planning records that carry the sentence stay as written: six sites,
four replaced and two kept. One of the four wraps across two comment lines
in `scripts/mutants.sh:9-10`, so the check joins lines before it matches
and a line grep is not used to prove it gone. The commit is
`18a02454` of 2026-07-26, found by counting at the commit that first wrote the
sentence (`3f7ebd09`, 2026-07-29, when the repository held 345 commits), and it
is the commit that added `CLAUDE.md` with the red/green rule, which is a
better reason to call it the start than its position. 8.9% of the history
predates it today; 53% did when the sentence was written. Touches 08-02
task 2 only.

**The research's five open questions, and where each stands.** Question 1,
why two published one-file counts were wrong at two commits, is open and
nothing here depends on it; the figure becomes a check in 08-02 and the
history of the wrong ones stays in the research. Question 2, whether the
2026-08-05 mutation run was whole-tree, is open and matters only for what
shape of failure to expect; 08-08 measures the rate before scheduling so
it does not need the answer. Question 3, how many of the records are
stale, is open and is what 08-07's sweep answers; nothing before the sweep
depends on it. Question 4, what "1,000 cached messages" and "idle with a
cache present" mean, is answered by the definitions below, pinned in
08-03. Question 5, whether the `cargo audit` failure was an eighth
advisory, is answered: it was `rsa` RUSTSEC-2023-0071, accepted with its
reasoning in `.cargo/audit.toml` on 2026-09-13, and 08-06 dates the four
accepted exit conditions.

**Decision 6, whether phase 8 owns the CI problem, is settled outside the
phase.** The toolchain was pinned at `1.98.1` in `rust-toolchain.toml` on
2026-09-13 with a test comparing it against every workflow; `cargo audit` runs
in `scripts/check.sh` through `scripts/audit.sh` since 2026-09-13; the failing
advisory the research could not read was `rsa` RUSTSEC-2023-0071, accepted
with its reasoning in `.cargo/audit.toml`; `main` was pushed 2026-09-14 and all
seven CI jobs are green, the Accessibility and NVDA workflows too. No plan here
carries it. 08-06 dates the four accepted advisories' exit conditions, which is
the one thing the research's reading of that problem left for this phase.

## What the tree now contradicts in the research

Every figure was re-taken 2026-09-14 at `b14d6379`. The command is the one the
research used unless it says otherwise. "The reader" in the table is this
one, run over `guards/guards.toml`, with the sum changed per row:

```
python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(len(g), sum(1 for r in g if len(r.get('tests_last_seen',[]))==1), len({e['file'] for r in g for e in r.get('tests_last_seen',[])}))"
-> 783 568 200
```

| Figure | Research, 2026-09-12 | Today | Command |
|---|---|---|---|
| Guard records | 734 | 783 | the reader, `len(g)` |
| Records naming one file | 537 | 568 | the reader, the second sum |
| Records spelled as inline tables | 4 | 4 | `grep -c 'tests_last_seen = \[{'` |
| Distinct files named by records | 190 | 200 | the reader, the set's size |
| Mutants the config allows | unknown | 12,335 over 247 files | `cargo mutants --list \| wc -l`, 3 seconds |
| Commits | 1,911 | 2,023 | `git rev-list --count HEAD` |
| Commits since the coverage reading | 1,679 | 1,824 | `git rev-list --count --since=2026-07-26 HEAD` |
| Lines under `src/` | 352,680 over 280 files | 360,792 over 286 | `find src -name '*.rs' \| xargs wc -l` |
| Test attribute lines in `src/` and `tests/` | 7,544 | 7,679 | `grep -rhE '^\s*#\[(test\|tokio::test)\]\s*$'` |
| Test functions in `tests/house_style.rs` | 69 | 70 | the same grep on one file |
| Records naming `tests/house_style.rs` | not taken | 21 | the reader, counting records with that file in `tests_last_seen` |
| `.planning/WINDOWS.md` entries | 342 | 441 | its frontmatter |
| Accepted advisories in `.cargo/audit.toml` | 7 | 4 | `grep '"RUSTSEC'` |
| CI | red since 2026-09-10 | green, 2026-09-14 | `gh run list` |
| Toolchain | unpinned, local 1.97.1 | pinned 1.98.1 | `rust-toolchain.toml` |
| "roughly half of WCAG" in product pages | 28 sites across 14 files | dated corrections only in `CLAUDE.md`, `docs/principles.md`, `docs/IMPLEMENTATION_STATUS.md`, `docs/wcag-coverage.md`; one historical changelog entry; three `.planning/intel` compilations dated 2026-08-29 | `grep -rln "half of WCAG"` |

Three things the research said that are now different in kind, not in number.

**The mutant count changes the phase's shape.** The research declined to
extrapolate it and said `--list` would answer in seconds. It answers 12,335.
The tree's two documents say a whole-tree run is "about two days". At the two
terms a guard record costs, 29 s to rebuild and 64 to 70 s for the library at
eight threads on 2026-09-10 and 2026-09-11, that is about 95 s a mutant with
the library suite alone, which is 325 hours, and about 230 s with every target
at the harness default, which is 33 days. Both are inferences; 08-08 measures
the rate on one shard under both shapes before anything is scheduled, and the
decision of which run to make is Pratik's with the products in front of him.
Criterion 6 allows a target to be revised with the reason, and "about two
days" is the target that had no basis.

**Decision 6 is moot**, as above.

**Phase 6's collision with this phase is history.** The research's section
"What phase 6 could invalidate" was written while phase 6 had not started.
Phase 6 corrected the "roughly half of WCAG" sentence at five sites and wrote
`docs/wcag-coverage.md` with the 155-rule and 55-criteria figures, their
commands and dates. What remains of the phrase is dated corrections and
records. 08-02's reading is scoped to five kinds of number and WCAG figures are
not among them; they have their own reading in
`src/presentation/what_the_scans_can_judge.rs`.

## The definitions the plans pin

The research's open question 4 asked what "1,000 cached messages" and "idle
with a cache present" mean. 08-03 pins them, writes them into the harness's
header and onto the page, and holds its executor to them.

- **1,000 cached messages**: 1,000 rows in `messages` for one IMAP account's
  `INBOX`, written through `upsert_messages`, each with a plain-text body of
  about 2 KB through `save_message_body`, no attachment content, no signed
  originals. That is what the list reads at startup and what the target was
  written about; attachments and signed originals have their own budgets and
  their own constants.
- **Cold start**: a fresh process of the release binary against that profile,
  from the first instruction of `main` to the usable line. The first start
  after the binary is built is reported on its own; the next five are the
  series and their median is the number.
- **Usable**: the first `MessagesLoaded` after startup whose rows reached the
  list control's count and were at least one. The application writes one log
  line at that moment, once per process.
- **Idle**: no input after the usable line, memory read at 60 s and 120 s; the
  120 s reading is the number, the 60 s reading sits beside it, and the
  difference is idle growth. The empty profile's 120 s reading is the floor.
- **Memory with 1,000 cached messages**: the application process's peak
  working set between start and 60 s, plus its WebView2 tree at 60 s, reported
  separately and summed, because a number that leaves out the renderer
  processes is a claim about half the program.

## The two long jobs

Neither can be sat through by an agent, both need a quiet machine, and both
have produced wrong conclusions here when read before they finished. Each is
three parts.

**The guard sweep (08-07).** `scripts/guards.py` gains `--log PATH` (appends
its own output, so no shell redirection is needed), `--resume` (reads the log's
own `-- name` and verdict lines, skips what has a verdict, measures the rest,
and re-measures a name with no verdict, which is what a kill mid-record
leaves), `--stop-after N`, and `--wait-until-quiet` (polls for `cargo.exe` and
`rustc.exe` before each record, and again after it, marking a record whose
run overlapped a foreign cargo as unmeasured so a resume takes it again).
Before a resume starts it runs `git status --porcelain` over every guarded
file and refuses, naming the file, if a kill left a break behind. Pratik
starts it from PowerShell with `Start-Process -WindowStyle Hidden` in a
worktree at `main` as it stands after 08-07's task 1 has merged, which is
the first commit holding those flags; the executor writes that hash into
the checkpoint, task 1 merges alone before it, and the sweep judges `main`
so that task 3's corrections land on the tree it measured. He closes the
window, kills it when he needs the machine, restarts it with `--resume`,
and does not commit on `main` while it runs, because `main`'s hook would
run the suite beside it. The runner's tests are its worked examples, run
and counted by `tests/house_style.rs`, which is in the tree-reading list and
so runs on a commit that touches the script. The cost on the day is 08-01's
rate row times the record count, both dated; the log's timestamps give the
real figure afterwards.

**The mutation run (08-08).** `scripts/mutants.sh` gains `--shard k/n`, each
shard to its own directory with the commit, the arguments and the copy
setting written beside it before it starts, `--out DIR` so two measurements
of one shard do not overwrite each other, `--in-place` passed through, and
`--shards n`, which runs shards in turn, skips complete ones, waits for a
quiet machine before each, and so resumes at shard granularity when
restarted. `scripts/mutants_report.py --shards DIR N` merges every shard's
own record and refuses a missing, partial, or differently-committed shard
through the same four refusals a single run gets. The run lives in a
worktree at `main` after 08-08's task 1 has merged, at one commit for its
whole length, however long, because shards divide the list of the tree they
run in and two commits are two lists. Every product is two terms: the
per-mutant rate times the count, plus a per-shard fixed cost, the copy and
build from nothing that each `cargo mutants` invocation pays before its
first mutant, which at `n` near 500 is up to about 40 hours over the run;
`--in-place` in the dedicated worktree turns that copy-and-build into a
build, and `--baseline skip` on later shards removes a suite run each but
also the check that the tree is green, which the frozen worktree keeps
true. The rate is measured on one shard of about twenty-five mutants
under both suite shapes, and with and without `--in-place`, before the
decision, and each row says its shard count, copy setting and baseline
setting.

## What cannot be measured on this machine

From the research's table, re-checked 2026-09-14. Each becomes a ledger entry
in 08-09 if one is not already there.

| What | Why |
|---|---|
| Anything against a real mail account | no account has ever been used; the ledger and the roadmap record it across every phase |
| PERF-03's real mailbox of 100,000 messages | the requirement itself says synthetic rows answer the list question and the provider question waits for a live account |
| A OneNote tenant, a CalDAV or CardDAV server, Google or Microsoft Graph | every request is answered by a loopback server the tests start |
| A Linux or macOS build | `other-platforms.yml` is `workflow_dispatch` only and 07-06's checkpoint has not been run |
| A published release or a signed artefact | 07-08's certificate waits on an Azure account only Pratik can create; the update download's size in `docs/privacy.md` is about an artefact no release has produced |
| Screen reader confirmation of anything | the project's second guardrail; `docs/manual-accessibility-pass.md` lists seventy-six items and its first sentence says none has happened; 417 of the ledger's 441 entries are open |
| The whole-tree mutation run inside the phase | at any rate this machine can give it is days to weeks; it may outlast the phase and 08-09 says so if it does |

## Costs every plan is written around

**Guard records: 783 on 2026-09-14, 21 of them naming `tests/house_style.rs`
and 48 naming `src/presentation/wx_app.rs`.** A test added to either file
flags every record naming it, at 08-01's rate each, which is about half an
hour for the first and over an hour for the second, detached. So no plan adds
a test to either: the new readings go in new integration targets at zero
records, the new pure functions go in new modules at zero records, and the
two existing sort tests in `wx_app.rs` stay where they are and call the moved
function through a `use`. Every plan reports both files' test counts before
and after, 70 and 199 on 2026-09-14.

**A new integration target needs a record**, or `scripts/check.sh` cannot tell
which commits could break it; `CLAUDE.md` records two guards that ran on every
commit except the ones that mattered for exactly that reason. Every new target
here gets one whose `suite` names it, and the plan quotes
`scripts/check.sh --suites-for` afterwards. A target that reads documents goes
in both of `check.sh`'s lists.

**`cargo test` takes one `--lib`.** Every `<verify><automated>` here is one
`--lib` per invocation with several filters after `--`, or several invocations
joined with `&&`. The count check for this is `CLAUDE.md`'s own paragraph; 55
plans got it wrong before 2026-09-07.

**Two records break `sort_messages`** and move with it in 08-04; each is
re-measured and should still redden exactly its one named test.

**The known gate-mapper holes**, so each plan says which test reaches what it
touches: `scripts/*.py` maps to no target and is reached by the `house_style`
tests that run the doctests; `scripts/mutants.sh` is read as text by two
`house_style` tests; `.cargo/mutants.toml` and `.cargo/audit.toml` are read by
nothing and are configuration; `docs/*.md` are read by the document-reading
targets, which a documents-only commit runs; `guards/guards.toml` is read by
seven `house_style` tests on every commit.

**The recurring findings from earlier executors**, carried into every plan's
context rather than restated per task:

- Trace an absence claim rather than accepting it; the research's own first
  pass wrote a number beside a command nobody had run.
- A guard record goes stale inside its own plan; run the `--remeasure` remedy
  whenever the count check prints it, detached, and read the result.
- Running `scripts/check.sh affected` by hand runs only the tree guards; the
  hook runs the scoped tests.
- Do not write a count you have not just taken. Every figure in these plans is
  dated 2026-09-14 and every task re-takes what it quotes.
- `gsd-tools roadmap update-plan-progress` counts a README as a plan; edit
  `ROADMAP.md` by hand and read the diff. `gsd-tools windows append` corrupts
  an entry holding a backslash; write none.
- Measure carriage returns with `tr -cd '\r' | wc -c`, never grep. No em dash
  anywhere, `.planning` included; `tests/house_style.rs` reads it.

## Estimates, and the factor behind them

`raw_tokens` is 30,000 per work task, the projection shape phases 6 and 7
used. `tokens` is that multiplied by **0.25**, the mean of
`actuals.tokens / estimate.raw_tokens` over the eighteen landed plans of
phases 6 and 7, read 2026-09-14 from each plan's `estimate` block and each
summary's `actuals` block: the nine of phase 6 are 0.063, 0.239, 0.351, 0.124,
0.285, 0.241, 0.378, 0.357 and 0.539; the nine of phase 7 are 0.128, 0.102,
0.134, 0.135, 0.228, 0.049, 0.310, 0.526 and 0.276. The spread is eleven
times, so `low`, derived from the sample and not self-rated. Phase 6's README
gave 0.17 over nine samples; the nine plans that landed after that reading
ran between 0.049 and 0.539, and four of them above 0.3, which is why the
mean moved.

Checkpoints are not counted as tasks. The two long jobs' checkpoints wait for
hours or weeks of machine time that no token estimate measures.

## What is owed to documents, and who does it

1. **The roadmap's phase 8 entry and progress row.** Done by the planner in
   the commit that lands these plans: `**Plans**: TBD` becomes the nine-line
   list, the row becomes `0/9`, and criterion 5's "roughly 15 hours" becomes
   the dated product. `test_the_roadmap_counts_the_files_that_are_on_disk`
   holds the row to the files.
2. **`.planning/STATE.md`.** Done by the planner in the same commit: phase 8
   current, `Total Plans in Phase: 9`, `progress.total_plans` counted from the
   disk.
3. **`docs/development/measurements.md`.** 08-01 creates it; every later plan adds rows
   and nothing else in the tree restates them.
4. **The requirement evidence lines and the ticks.** 08-06 corrects the
   evidence by hand; 08-09 ticks clause by clause. No earlier plan ticks a
   requirement because its own tasks finished.

---
phase: 06-how-the-application-speaks
plan: 06
status: partial
subsystem: infra
tags: [commit-gate, which-checks, accessibility-scan, workflow, axe-windows, msaa, checkpoint]

requires:
  - phase: 07-installing-updating-and-what-is-stored
    provides: "07-02's `*.iss` rule in `scripts/which-checks.sh`, the shape this rule copies and the reason it sits below the version-bump exception; 07-06's measurement that a workflow file selects no scoped target"
provides:
  - "`scripts/which-checks.sh`: any path under `.github/workflows/` answers `all` on a branch, below the manifest block, keyed on the folder rather than the extension"
  - "`scripts/which-checks.test.sh`: five cases, three the rule makes true and two that hold its shape, each a name a commit can carry"
  - "The checkpoint's two questions, set out with options, costs and a recommendation, unanswered"
affects: [06-07, 06-08, gate, ci]

actuals:
  tokens: 1400
  tasks: 1
  commits: 2

tech-stack:
  added: []
  patterns:
    - "A gate rule for a data file keys on what makes the file that kind of file: the extension for a setup script, the folder for a workflow, and the suite carries one allow case per wrong spelling of the rule"
    - "A selection hole is measured by staging the data file's change alone and running the gate the way the hook does, not by breaking the code that reads it"

key-files:
  created: []
  modified:
    - scripts/which-checks.sh
    - scripts/which-checks.test.sh

key-decisions:
  - "The rule matches `.github/workflows/*`, not `*.yml`: an `.iss` anywhere is a setup script, a `.yml` anywhere is not a workflow, so here the directory decides where the installer rule's extension does"
  - "The plan's plain-markdown and plain-Rust allow cases were not added, because the suite already holds each three times over; the two allow cases written are the ones that redden under the wrong spellings of the rule"
  - "The hole was measured by taking a window out of the workflow's array with the code left alone, not by breaking `ScanTarget::ALL`, because the code file maps to a scoped target and would have run the reading test"
  - "No guard record: the rule reads no file, the suite's own red run is the violation shown to the reading, and no record has ever named a shell suite"

patterns-established:
  - "Red commit naming shell cases as `which-checks::<description>`, three of them, accepted by `red-commit.sh` and held to exactly those three"

requirements-completed: []

coverage:
  - id: D1
    description: "A commit touching any file under .github/workflows/ answers all on a branch, beside a document and beside a version bump"
    requirement: FEEDBACK-03
    verification:
      - kind: unit
        ref: "scripts/which-checks.test.sh::a workflow file on a branch"
        status: pass
      - kind: unit
        ref: "scripts/which-checks.test.sh::a workflow file beside a document"
        status: pass
      - kind: unit
        ref: "scripts/which-checks.test.sh::a workflow file beside a version bump"
        status: pass
    human_judgment: false
  - id: D2
    description: "The rule catches what it is for and nothing else: a document inside .github is still docs_only and a yml outside the workflows folder is still affected"
    requirement: FEEDBACK-03
    verification:
      - kind: unit
        ref: "scripts/which-checks.test.sh::a document inside the .github folder"
        status: pass
      - kind: unit
        ref: "scripts/which-checks.test.sh::a yml file outside the workflows folder"
        status: pass
    human_judgment: false
  - id: D3
    description: "The gate goes red on a workflow that disagrees with ScanTarget::ALL when the workflow alone is staged, which before this rule it did not"
    requirement: FEEDBACK-03
    verification:
      - kind: other
        ref: "bash scripts/check.sh with .github/workflows/accessibility.yml staged minus one window: exit 0 in 64s before the rule, exit 101 in 206s after, naming blocked-senders"
        status: pass
    human_judgment: false
  - id: D4
    description: "Whether to pin the Axe.Windows release and how many windows the scan looks at"
    verification: []
    human_judgment: true
    rationale: "Both are Pratik's, per the plan's blocking checkpoint. Set out below with options, costs and a recommendation; not answered, not built"

duration: 35min to the green commit, about 1h with the documents
completed: 2026-09-14
---

# Phase 06 Plan 06: The scan is reproducible, and a change to it earns its checks Summary

**A commit that changes a file under `.github/workflows/` now runs the whole gate on a branch, so the two tests that read the accessibility workflow run on the commits that could break them. Before this, a workflow with one window taken out of its scan list passed the gate in 64 seconds; after, the same staged break fails in 206 seconds naming the window. Task 2, pinning the scanner and settling the window list, is behind a checkpoint that is Pratik's and is not attempted.**

Branch `a-workflow-change-earns-the-checks-that-read-it`, one red commit, one green commit and one docs commit, `scripts/check.sh all` green on the branch under rustc 1.98.1 in 253 seconds with 7,603 tests passed and 0 failed. Merged into `main` at `9f86ba6f` with the whole gate green again on the merge, 281 seconds. The hash is written here by the follow-up commit, since a summary committed before its own merge cannot name it. Nothing pushed.

## Which clauses of criteria 3 and 4 this closes

Read from `ROADMAP.md` rather than from the plan. Criterion 3 has two clauses: the scan output names which WCAG 2.2 AA criteria it can and cannot judge, and "roughly half" becomes a list. Criterion 4 has two: the interactions only a human pass can cover are a scoped list, and each of the five WebView2 findings is fixed or recorded upstream with the upstream named. **Task 1 closes none of the four.** It is what makes 06-07's list a claim about something a commit cannot silently change under it, which the plan calls "the half that is not prose", and the roadmap's wording does not have a clause for that. Nothing user-visible changed, so no version bump and no changelog entry; a CI workflow is met by nobody using the program.

## What landed

### Task 1: a workflow change earns the checks that could catch it

Red `05ec26a4`, green `dd4934fe`.

**The rule**, at `scripts/which-checks.sh:324-370`, below the installer rule and above the final markdown rule:

```sh
for path in "$@"; do
    case "$path" in
        .github/workflows/*)
            echo all
            exit 0
            ;;
    esac
done
```

Its comment names the two tests in `src/presentation/scan_target.rs` that read `.github/workflows/accessibility.yml`, `test_the_command_line_and_the_workflow_use_the_same_flag` and `test_the_workflow_asks_for_every_target`, says they are unit tests in `src/` so `affected` chose no scoped target for them and they ran on every commit except the ones that could break them, records the measurement below with its date, and says why the folder decides here where the extension decides for `.iss`: an `.iss` anywhere in this tree is a setup script, and a `.yml` anywhere is not necessarily a workflow, so a dependabot file or a tool's own configuration would be charged the whole gate for a test that reads none of them.

**Below the manifest block on purpose.** The version-bump exception hands the softer answer to a commit whose whole manifest diff is this package's own version line, and a rule written inside that branch would let a workflow change that also bumps the version answer `affected`. The suite's version-bump case, using the fixture diff the file already holds, is what would notice; it was red before the rule and is green after.

**The five cases**, quoted, with what each answered before and after:

| description | file list | before | after |
|---|---|---|---|
| `a workflow file on a branch` | `.github/workflows/accessibility.yml` | `affected` | `all` |
| `a workflow file beside a document` | the workflow and `docs/changelog.md` | `affected` | `all` |
| `a workflow file beside a version bump` | the workflow, `Cargo.toml`, `Cargo.lock`, version-only diff | `affected` | `all` |
| `a document inside the .github folder` | `.github/PULL_REQUEST_TEMPLATE.md` | `docs_only` | `docs_only` |
| `a yml file outside the workflows folder` | `.github/dependabot.yml` | `affected` | `affected` |

The last two are the allow half and they earn their place by naming a wrong rule each refuses: `.github/*` would take the pull request template with it, and `*.yml` would take a dependabot file with it. No `.yml` exists outside the workflows folder today; `which-checks.sh` never touches the filesystem, so the answer depends only on the path handed to it, and the case is about the rule's shape rather than a file's existence.

**Every description is a name a commit can carry, and `shell-suite.sh` judged that rather than my eyes.** It exits 70 on an empty description, one holding ` ... ` or one holding a comma, and reports a case of its own for two cases sharing a name. The suite reached `suite_verdict` on both the red run and the green run, which is the condition of its having accepted all five; nothing was eyeballed.

## The hole, measured before it was closed

The plan said to break `ScanTarget::ALL` and watch the gate pass. That would have measured nothing: the constant lives in `src/presentation/scan_target.rs`, which the gate maps to `--lib presentation::scan_target::`, so the reading test would have run and gone red. The defect is that a change to the *workflow* selects no target, so the break has to be on the workflow's side with the code left alone. Recorded as ledger 387.

**Before the rule**, on the branch, with `blocked-senders` removed from the workflow's `$targets` array and that file alone staged:

```
$ bash scripts/which-checks.sh a-workflow-change-earns-the-checks-that-read-it .github/workflows/accessibility.yml
affected
$ cargo test --lib -- presentation::scan_target::
test presentation::scan_target::tests::test_the_workflow_asks_for_every_target ... FAILED
blocked-senders is not in the workflow's target list, so it is never scanned
test result: FAILED. 5 passed; 1 failed
$ bash scripts/check.sh            # no argument, so it reads the index the way the hook does
== rustfmt ==
== clippy ==
== the scripts that decide what runs ==
-- audit
-- check
-- red-commit
-- which-checks
== the tests that reach what changed ==
-- the guards that read the whole tree
tree-reading guards passed. The rest of the suite and the release
build did not run. Run 'scripts/check.sh all' before merging.
check.sh exit 0 in 64s
```

The scoped run chose no target. The test that names the missing window was red on the same tree, and the gate never asked it.

**After the rule**, the same break, the same staging, the same invocation:

```
$ bash scripts/which-checks.sh a-workflow-change-earns-the-checks-that-read-it .github/workflows/accessibility.yml
all
$ bash scripts/check.sh
== rustfmt ==
== clippy ==
== the scripts that decide what runs ==
== security advisories ==
== tests ==
test presentation::scan_target::tests::test_the_workflow_asks_for_every_target ... FAILED
blocked-senders is not in the workflow's target list, so it is never scanned
test result: FAILED. 7174 passed; 1 failed; 1 ignored
check.sh exit 101 in 206s
```

Both breaks were reverted with `git restore --staged` and a per-file `git checkout --` before anything was committed. The workflow is byte-identical to `main`'s; neither accessibility channel was touched and no trigger changed, because the file was never part of a commit.

A hook-shaped run rather than `scripts/check.sh affected` by hand, because the brief and observation log both record that a bare `affected` supplies no changed-file list and runs only the tree-reading guards. With no mode argument, `check.sh` reads `git diff --cached --name-only` and asks `which-checks.sh` itself, which is the hook's path minus the message file.

## What the gate selects for each file type, before and after

Taken on the branch with `bash scripts/which-checks.sh <branch> <files>`; `main` answers `all` for every non-document regardless and is unchanged.

| file list | before | after |
|---|---|---|
| `.github/workflows/accessibility.yml` | `affected`, selecting no scoped target | `all` |
| `.github/workflows/other-platforms.yml` | `affected` | `all` |
| the workflow and `docs/changelog.md` | `affected` | `all` |
| `.github/PULL_REQUEST_TEMPLATE.md` | `docs_only` | `docs_only` |
| `.github/dependabot.yml` | `affected` | `affected` |
| `scripts/msaa-names.ps1` | `affected`, selecting no scoped target | `affected`, selecting no scoped target |
| `scripts/which-checks.sh` | `affected` | `affected` |
| `src/lib.rs` | `affected` | `affected` |
| `docs/changelog.md` | `docs_only` | `docs_only` |

Two things the table says beyond the rule.

**`scripts/msaa-names.ps1` is the MSAA half of the scan, the channel NVDA reads, and it is held by nothing.** `grep -rln msaa-names.ps1 src tests` finds no file. A change to it answers `affected`, selects no scoped target, and runs formatting, clippy, the shell suites and the four tree guards. The workflow depends on its exit-code contract at lines 180 to 184, 0 for every operated control named, 1 for one unnamed, 2 for the walk failing, and no test reads that contract. The rule this plan added does not cover it, correctly: there is no test for a rule to select. Ledger 385, a test to write rather than a rule.

**`release.yml` was already covered by a different path**, and `accessibility.yml` was not. `bash scripts/check.sh --suites-for guards/guards.toml .github/workflows/release.yml` answers `installer`, because 07-07 wrote two guard records naming that file and the suite that reads it, and `check.sh`'s coupling scan follows records to `--test` targets. The same command for `accessibility.yml` answers nothing: `grep -c accessibility.yml guards/guards.toml` is 0, and the two tests that read it are `--lib` tests no record could route to anyway. So the two workflows had two different holes, and this rule closes both the way 07-02's rule closed the installer's, by answering `all` rather than by routing.

## What the suite costs, before and after

`bash scripts/which-checks.test.sh` on its own, 2026-09-14, this machine, 24 logical cores, Git Bash, nothing else building, two runs each way:

| | run 1 | run 2 | case lines |
|---|---|---|---|
| before | 6.46 s | 6.57 s | 45 |
| after | 6.99 s | 7.00 s | 50 |

About half a second for five cases, roughly a tenth of a second each, which is one process start per case on this machine. The whole family of `scripts/*.test.sh` was 108 seconds on 2026-09-10; this suite alone was 34.4 seconds on 2026-09-09 by `CLAUDE.md`'s account and is 7 seconds today, so either the machine state differs a great deal between the two days or the earlier figure was taken under load. Read today's pair as the pair it is, taken twice within a minute, and not as a correction of the earlier one.

The gate cost the rule buys: a workflow-only commit on a branch goes from about 64 seconds to about 206 seconds warm, the difference being the library run, the advisory check and the release build. Paid only by a commit that changes a workflow, and the last such commit before this branch was 07-06's on 2026-09-12.

## Why no guard record, and the census unchanged

`guards/guards.toml` holds 757 records by a TOML reader and holds 757 after this plan; the census at its lines 79 and 80 is untouched. No record was written, for three reasons that are each enough. The rule reads no file, so there is no reading for a break to blind: it is a `case` on a path string. The suite's red run is the violation shown to the reading, which is what a companion guard proves for a file-reading check; here the reading is the suite itself and it was red at `05ec26a4` for exactly the three cases named. And no record has ever named a shell suite: `grep -c 'test\.sh' guards/guards.toml` is 0, `guards.py` applies breaks to source and reads cargo's FAILED lines, and inventing that coupling is not this plan's work. 07-02 reasoned the same way for the `.iss` rule.

## Deviations from plan

**1. [Rule 1] The hole was measured on the workflow's side, not by breaking `ScanTarget::ALL`.** The plan's break lands in a file the gate maps to a scoped target, so it would have shown the gate working rather than the hole. Reasoned above. Ledger 387.

**2. The plan's plain-markdown and plain-Rust allow cases were not added.** `one planning file`, `several docs` and `a readme` at lines 95 to 98 already assert `docs_only` for a markdown-only change, and `one rust file`, `rust beside a doc` and `an integration test` at 101 to 103 assert `affected` for Rust. A fourth copy of each proves nothing the file does not prove. The two allow cases written instead are the ones that go red under the wrong spellings of the rule. 07-02 made the same substitution and recorded it; the plan asked for the generic negatives again. Ledger 386.

**3. Five cases rather than four.** The plan's four plus the version-bump combination, which is the case that decides where the rule may sit and was red before the rule. 07-02 wrote the same fifth for the same reason.

**4. Task 2 not attempted and the checkpoint not answered**, by instruction. The workflow is untouched, so `test_ci_and_this_machine_are_told_to_use_the_same_compiler` had nothing to say and neither accessibility channel could have been dropped.

**5. No `docs/changelog.md` entry and no version bump.** The plan says so and it is right: a commit gate is met by nobody using the program.

Nothing in `scripts/check.sh` or `scripts/which-checks.sh` conflicted with phase 7's edits; the plan's `read_first` line numbers for `which-checks.sh` were exact at `436cdeec`, and `run_the_tests_that_reach_what_changed` is at `check.sh:387` as 07-02 left it.

## The plan's premises, checked against the tree

Premises 1 to 3 and 5 to 7 held exactly. Premise 8's figure for this suite is discussed above. Premise 4 held: eleven in the workflow, ten in `ScanTarget::ALL`, `main` being the eleventh. The checkpoint's context, though, undercounts the windows outside the scan, and that correction is in the next section because it changes one of the answers' costs.

## CHECKPOINT REACHED

**Type:** decision
**Gate:** blocking-human
**Plan:** 06-06
**Progress:** 1/2 tasks complete

### Completed Tasks

| Task | Name | Commit | Files |
|---|---|---|---|
| 1 | a workflow change earns the checks that could catch it | `05ec26a4` red, `dd4934fe` green | `scripts/which-checks.sh`, `scripts/which-checks.test.sh` |

### Current Task

**Checkpoint:** pin the scanner, and how many windows should it look at
**Status:** awaiting decision
**Blocked by:** two questions that are Pratik's, per the plan's `gate="blocking-human"`; not auto-selected

### The two questions, so they can be answered from here

Both are about `.github/workflows/accessibility.yml`, which runs on every push to `main`, scans the running program window by window on both channels, Axe.Windows over UI Automation for what Narrator reads and `scripts/msaa-names.ps1` over MSAA for what NVDA reads, and uploads what it found. 06-07 writes the list of which WCAG 2.2 AA criteria that scan can and cannot judge, fifty-five criteria against Axe.Windows's 155 rules across three of them. Both answers change what that list can honestly say.

#### Question 1: pin the scanner, or leave it fetching the latest release?

Today, line 78 of the workflow asks GitHub for `microsoft/axe-windows`'s latest release on every run and downloads whatever zip that is. The rule set the scan runs is therefore whatever Microsoft shipped most recently, and it can change with no commit here.

| option | what it gives | what it costs |
|---|---|---|
| **A. Pin the release tag** | 06-07's list is a claim about a named rule set anybody can re-run. One fewer binary downloaded and executed in CI whose contents nobody chose, which is a supply-chain matter as well as a reproducibility one. The tag, the date and the reason sit in a comment beside the line | Somebody has to move it, and a newer rule set arrives only when they do. A pin nobody moves is a scanner going quietly stale, and nothing in the tree would say so |
| **B. Leave it unpinned, and date the list** | New rules arrive free. Two lines of the workflow stay as they are | 06-07's list has to carry the version it was read against and the date, and will drift from what the scan really runs. The research read the rule table twice, five days apart, and got 144 and 155; nobody can now say how much of that was upstream moving and how much was a bad parse, which is exactly what an unpinned dependency makes unanswerable. A ledger entry records that the list describes something that can change with no commit here |

**Recommendation: A, and go one step further than the plan: pin the tag and record the zip's SHA-256 beside it.** A tag can be moved by whoever owns the repository; a checksum cannot, and `Get-FileHash` is one line of the same PowerShell step. That makes the scan a claim about a specific binary, not about a name. The staleness cost is real and is answered by writing the date beside the pin and by 06-08 reading the findings: a scanner is bumped when somebody wants to know what a newer one finds, and the date tells them how long it has been. Task 2's instruction to read the tag from the GitHub releases API in the session that writes it, and to quote the endpoint, still holds.

If B is chosen, task 2's disposition for T-06-21 becomes accept, 06-07's list must carry version and date, and ledger 384 already records the drift.

#### Question 2: how many windows should the scan look at?

Today the scan looks at eleven: the main window and ten dialogs, `settings`, `accounts`, `compose`, `reader`, `search`, `filters`, `calendar`, `first-run`, `add-calendar` and `blocked-senders`. Each is a `ScanTarget` variant, a command-line name, an entry in the workflow's array, and a window `wx_app.rs` can open from a fresh profile. Two tests hold the code's list and the workflow's array together, and after task 1 they run on the commits that could break them.

**The plan says nine dialogs are outside the scan. The tree says more.** The research counted 24 `wx_*` modules on 2026-09-12; `git ls-tree` at its own commit shows 27, and today shows 27. Outside the scan today:

- The nine the plan names: `wx_columns`, `wx_conflict_choice`, `wx_destination`, `wx_folder_choice`, `wx_item_form`, `wx_managers`, `wx_reminder_alert`, `wx_thread_view`, `wx_which_days`.
- Two more that build a `Dialog` and existed when the research was written: `wx_send_later`, added 2026-09-06, where somebody chooses when a message goes, and `wx_add_address_book`, added 2026-09-11, the second window that asks for a password to send somewhere other than a mail server, which the `add-calendar` target's own doc comment gives as the reason that one is scanned.
- Three of the five managers in `managers.rs`: `manage_tags`, `manage_signatures` and `manage_contacts`. `filters` and `calendar` are scanned; the other three are the same shape and are not.
- Possibly the five module panels, `wx_calendar_module`, `wx_contacts_module`, `wx_notes_module`, `wx_reminders_module`, `wx_tasks_module`, which live inside the main window and are shown when their module is selected. Whether the `main` scan on a fresh profile sees anything but the mail module was not measured here and would need a scan artifact to answer. Named so 06-07 does not list the main window as covered without asking.

So the plan's "eleven scanned, nine outside" is "eleven scanned, at least fourteen outside" by today's tree, before the five panels are counted either way. The honest floor for any answer is that 06-07's document lists what is outside by name.

| option | what it gives | what it costs |
|---|---|---|
| **A. Leave it at eleven, and name what is outside** | The coverage document says which windows it does not speak for, which is the honest minimum under every option. Task 2 is then the pin alone, and 06-08 judges findings from windows that already scan | Everything above stays unscanned for longer, including the item form, which is where events, contacts, tasks and notes are entered and the one dialog `tests/checkbox_labels.rs` covers |
| **B. Add `wx_which_days` only** | The plan's suggested combination. One variant, one name, one workflow entry, and the dialog the one skipped NVDA test names gets a target | The dialog opens only from deep inside the calendar's edit and delete flow on a real, selected, repeating event, so the target needs a made-up repeating event built at startup the way `reader` opens on a made-up message; that is `wx_app.rs` work, and reachability from a fresh profile is proved only by the scan running, which this phase cannot see until 06-08 reads a CI artifact. It does not un-skip the NVDA test, which needs a real NVDA run either way and which the plan says not to touch |
| **C. Add some** | A stronger list. `wx_item_form`, `wx_send_later` and `wx_add_address_book` are the three with the most controls somebody operates and the most to say about themselves | Each is a variant, a name, a workflow entry and a fixture that opens it; more scan time; more findings for 06-08 to judge one at a time |
| **D. Add all of them** | Most complete | All of C several times over, and some may not be reachable from a fresh profile at all, which is what the skipped NVDA test found for `wx_which_days` |

**Recommendation: A for this phase, with the list of what is outside written into 06-07's document by name, and `wx_item_form`, `wx_send_later` and `wx_add_address_book` named there as the first three to add.** The plan recommends A plus `wx_which_days`, and I would not add that one first: it needs a fixture repeating event nothing else needs, it buys a scan of a small dialog whose findings 06-08 then has to judge, and it does not unblock the skipped test, which needs a real NVDA run. A target added this phase can only be shown to work by a CI run this phase cannot read, so every target added here is one nobody has seen scan. The three I name instead are the ones with the most controls a person operates and, for two of them, a password field. Widening is good work and a plan of its own, after 06-08 has shown what the scan finds on the eleven it already has.

**The combination the plan expects Pratik to give:** most likely A on the scanner with the checksum, A on the windows with the three named for later. If a target is added, name which; each is a variant with a command-line name, a workflow entry and a way of opening it from a fresh profile, and 06-08 is where it is seen to work.

### Awaiting

Say whether to pin, and how many windows. If any target is added, name which.

## What task 2 will need to know, left here so it need not re-derive it

- `ScanTarget::ALL` is `[ScanTarget; 10]` at `scan_target.rs:72`; the workflow's `$targets` at line 100 holds those ten plus `main`. Adding a variant reddens `test_the_workflow_asks_for_every_target` until the array gains it, which is a genuine red half; the pin alone is a configuration change with no test to write first, and the summary should say which happened.
- The commit that touches the workflow answers `all` after this plan. Expect the full gate through the hook, about 206 seconds warm on this machine plus whatever the release build costs cold, and run it rather than working around it.
- `test_ci_and_this_machine_are_told_to_use_the_same_compiler` in `tests/house_style.rs` holds every workflow to 1.98.1, so a workflow edit that touches the toolchain line has something to answer.
- The Axe.Windows tag must be read from `https://api.github.com/repos/microsoft/axe-windows/releases` in the session that writes it, and the endpoint and tag quoted.
- Keep the `throw` at line 80 when the asset is missing, and keep both channels: Axe.Windows over UI Automation and `scripts/msaa-names.ps1` over MSAA. A name that fails on either is a name somebody does not hear.

## Ledger

`.planning/WINDOWS.md` 384 to 387, both halves matching at 387 entries, written through `gsd-tools windows append` with no backslash in any description.

| id | kind | what |
|---|---|---|
| 384 | unmet-truth | the scan still fetches `releases/latest`, so the rule set 06-07 describes can change with no commit here; pinning is task 2 behind the checkpoint |
| 385 | todo | `scripts/msaa-names.ps1` is read by no test and maps to no gate target; its exit-code contract, which the workflow depends on, is unheld |
| 386 | deviation | the plan's generic allow cases replaced by the two that redden under the wrong spellings of the rule |
| 387 | deviation | the hole measured on the workflow's side rather than by breaking `ScanTarget::ALL` |

## Known Stubs

None. The rule is wired: the hook runs `check.sh`, which asks `which-checks.sh`, which now answers `all` for the path, measured on a staged break rather than read.

## Threat Flags

None new. T-06-22 is mitigated by task 1 as the register planned. T-06-21 and T-06-23 stay open until the checkpoint is answered and task 2 runs; ledger 384 carries them.

## What was not done, said plainly

- Task 2: not started. The workflow is byte-identical to `main`'s.
- The scanner is not pinned. What the scan runs today is whatever Microsoft shipped most recently.
- No window was added to the scan.
- No scripted edit touched a tracked file. Every change to `scripts/which-checks.sh` and `scripts/which-checks.test.sh` was Read then Edit; the two temporary edits to the workflow were Edit and a per-file `git checkout --`. Exception set: zero. Carriage returns in each edited file measured with `tr -cd '\r' | wc -c`: 0.
- `roadmap update-plan-progress` was not run; `ROADMAP.md` and `STATE.md` were edited by hand and the diff read. The first docs commit was refused because `STATE.md` holds the plan number twice and I had moved one copy; both now say 6.

## Self-Check: PASSED

`scripts/which-checks.sh`, `scripts/which-checks.test.sh` and this file exist on disk; commits `05ec26a4`, `dd4934fe`, `2b220b66` and `9f86ba6f` are in `git log --all`; `.github/workflows/accessibility.yml` is byte-identical to `436cdeec`'s.

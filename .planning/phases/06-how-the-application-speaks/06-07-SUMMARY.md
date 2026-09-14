---
phase: 06-how-the-application-speaks
plan: 07
status: complete
subsystem: docs
tags: [wcag, accessibility-scan, axe-windows, msaa, coverage, document-guard, commit-gate, checkpoint]

requires:
  - phase: 06-how-the-application-speaks
    provides: "06-06's pinned Axe.Windows v2.4.2, its thirty-one scan targets and seventeen nested dialogs by name, and its finding that no scan has run on either channel since 2026-09-10"
provides:
  - "`docs/wcag-coverage.md`: fifty-five rows, one per WCAG 2.2 Level A and AA success criterion, saying whether Axe.Windows, the MSAA walk and the NVDA suite can produce a finding against it and what is left for a person, written so it does not read as a compliance claim"
  - "`src/presentation/what_the_scans_can_judge.rs`: the three criteria the rule list cites and the one the MSAA walk contributes to, as code; a reading that holds the page's table to them in both directions; companions that plant a wrong row in each direction against the real page; the empty page and the empty list failing rather than passing; the scan's step summary held to the same three"
  - "`.github/workflows/accessibility.yml`: the step summary names 1.3.1, 2.1.1 and 4.1.2 and points at the page for the fifty-two; the header no longer claims the scanner measures contrast"
  - "`scripts/check.sh`: the documents-only run includes the reading, so a commit editing only the coverage page runs the check that reads it"
  - "Five corrected sentences, dated, old wording kept: `docs/accessibility.md`, `docs/IMPLEMENTATION_STATUS.md`, `docs/principles.md`, and two in `CLAUDE.md`"
  - "Three `REQUIREMENTS.md` evidence lines corrected in place, dated, old wording visible"
  - "Both checkpoint answers, recorded with the date"
affects: [06-08, docs, gate, ci]

actuals:
  tokens: 34000
  tasks: 2
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A page a person reads and a check that reads it: the rows live in the document, the few facts a test can hold live in code, and the reading is shown a planted violation in both directions before it is trusted"
    - "A `--lib` test whose subject is a document is placed by what it reads: the gate's documents-only path names it, or it runs on every commit except the one that could break it"
    - "A count taken from a source file is reported with its parts, and a parse whose parts do not sum to its total is not reported"

key-files:
  created:
    - docs/wcag-coverage.md
    - src/presentation/what_the_scans_can_judge.rs
  modified:
    - src/presentation/mod.rs
    - .github/workflows/accessibility.yml
    - scripts/check.sh
    - guards/guards.toml
    - docs/accessibility.md
    - docs/IMPLEMENTATION_STATUS.md
    - docs/principles.md
    - docs/changelog.md
    - CLAUDE.md
    - .planning/REQUIREMENTS.md

key-decisions:
  - "Both a document and a check, on Pratik's answer of 2026-09-14: the fifty-five rows in `docs/wcag-coverage.md`, the three criteria in code, a reading holding them together with a companion that plants a violation"
  - "`REQUIREMENTS.md` corrected in place, dated, old wording visible, on Pratik's answer of 2026-09-14"
  - "The rule count is read from the release's own `axe-windows-rules-2.4.2.md`, not from `main`'s table, and the page says so with the date"
  - "The Section 508 rules are counted as what the file says they are, not refiled under 4.1.2 by substance; the page says both things"
  - "The status page names the three criteria by name rather than number, because a house_style guard reads that page for version numbers and a criterion number looks like one; the guard was right to ask and `tests/house_style.rs` is untouched"
  - "No version bump: a corrected sentence and a new page are a docs pass, which CLAUDE.md says does not bump"

patterns-established:
  - "Search wrapped prose for the rarest word of a phrase, not the phrase: a single-line grep missed the copy that wrapped"

requirements-completed: []

coverage:
  - id: D1
    description: "The scan output names which WCAG 2.2 AA criteria it can judge and points at the list of those it cannot"
    requirement: FEEDBACK-03
    verification:
      - kind: unit
        ref: "presentation::what_the_scans_can_judge::tests::test_the_scan_output_names_the_criteria_the_code_names_and_not_a_fraction"
        status: pass
      - kind: other
        ref: "no CI run has produced the summary since the change; nothing pushed"
        status: unrun
    human_judgment: false
  - id: D2
    description: "Roughly half becomes a list: fifty-five rows, each saying what each channel can and cannot say"
    requirement: FEEDBACK-03
    verification:
      - kind: unit
        ref: "presentation::what_the_scans_can_judge::tests::test_the_page_has_a_row_for_every_criterion_at_level_a_and_aa"
        status: pass
      - kind: unit
        ref: "presentation::what_the_scans_can_judge::tests::test_the_page_says_yes_for_exactly_the_criteria_the_code_names"
        status: pass
      - kind: other
        ref: "guards.toml: the coverage page cannot say a criterion is unjudged while the code says the scan judges it; measured on the whole library and through scripts/guards.sh"
        status: pass
    human_judgment: false
  - id: D3
    description: "The reading reports a planted wrong row in both directions and fails on an empty page or an empty list"
    requirement: FEEDBACK-03
    verification:
      - kind: unit
        ref: "presentation::what_the_scans_can_judge::tests::test_a_fourth_criterion_marked_yes_on_the_page_is_reported"
        status: pass
      - kind: unit
        ref: "presentation::what_the_scans_can_judge::tests::test_one_of_the_three_marked_no_on_the_page_is_reported"
        status: pass
      - kind: unit
        ref: "presentation::what_the_scans_can_judge::tests::test_a_page_with_no_rows_fails_rather_than_passes"
        status: pass
      - kind: unit
        ref: "presentation::what_the_scans_can_judge::tests::test_an_empty_list_in_the_code_fails_rather_than_passes"
        status: pass
    human_judgment: false
  - id: D4
    description: "No product-facing sentence says the scan covers a fraction of WCAG"
    requirement: FEEDBACK-03
    verification:
      - kind: other
        ref: "grep quoted below: every remaining hit is a dated quotation of the old wording"
        status: pass
    human_judgment: true
    rationale: "The brief asked for the old wording to stay visible with the date, so the plan's grep is not empty by design; a person reads each hit"
  - id: D5
    description: "Whether any of the fifty-five criteria is met by this application"
    requirement: FEEDBACK-03
    verification:
      - kind: other
        ref: "nobody has walked a criterion; WINDOWS.md 395 and 396"
        status: unrun
    human_judgment: true
    rationale: "The page says what the scans can look at, not what they have found; whether a criterion is met is a person's, after phase 8"

duration: about 4h30m from branch to the merge, the whole gate included
completed: 2026-09-14
---

# Phase 06 Plan 07: "Roughly half" becomes a list of fifty-five Summary

**"Automated scanning covers roughly half of WCAG" is now a list: the scans can produce a finding against three of the fifty-five WCAG 2.2 Level A and AA success criteria, 1.3.1, 2.1.1 and 4.1.2, the MSAA walk judges the Name part of 4.1.2 alone, and `docs/wcag-coverage.md` has a row for every one of the fifty-five saying which channel can say what about it and what is left for a person.** The three live in code, a reading holds the page to them in both directions and has been shown a planted wrong row each way, and the scan's own summary names them. Nothing on the page says a criterion is met, no scan has run on either channel since the workflow was rewritten, and nobody has walked a criterion against this application.

Branch `roughly-half-becomes-a-list-of-fifty-five` from `main` at `a3d71f8e`. Documents `4d415de3`, red `0ee95483`, green `4abe217d`, corrections `93f8c865`, the planning documents in the commit after this file. Merged into `main` at the commit the follow-up commit names, since a summary committed before its own merge cannot name it. Nothing pushed.

## The checkpoint's two answers, recorded and not re-asked

Both answered by Pratik on 2026-09-14, the recommended option for each.

**The coverage list is both a document and a check.** The fifty-five rows live in `docs/wcag-coverage.md`. The three criteria the scan can produce a finding against live in `src/presentation/what_the_scans_can_judge.rs`, and a reading holds the page's table to them, with a companion proving the reading can see a disagreement. The other fifty-two rows are a person's judgement about applicability and no check holds those.

**`REQUIREMENTS.md` is corrected in place**, dated, with what each sentence used to say left visible. The three sentences were wrong about the code as it stands, not merely out of date. Phase 7 finished on 2026-09-12, so nothing conflicted.

## Both counts, re-taken from sources, with the arithmetic

Taken 2026-09-14.

### 155 rules, from the pinned release's own file

06-06 pinned Axe.Windows `v2.4.2`. That release ships `axe-windows-rules-2.4.2.md`, fetched from `https://github.com/microsoft/axe-windows/releases/download/v2.4.2/axe-windows-rules-2.4.2.md`, 31,107 bytes, SHA-256 `85b7d7b02de0b6f200e745c51b98c360e34ee6112b6675a44553b3b42af33daf`. The file has 174 lines: a heading at line 2, the table header at line 4, the separator at line 5, data rows at lines 6 to 160, a blank at 161, and severity descriptions after. Counted by line range so that no filter on the text of a row could eat one, which is how the 2026-09-12 parse lost fourteen rules whose names begin with "Name":

```
== data rows are lines 6 to 160 (header line 4, separator line 5, blank line 161):
155
== by standard referenced:
  61  WCAG 1.3.1 InfoAndRelationships
  53  Section 508 502.3.1 ObjectInformation
  23  Section 508 502.3.10 AvailableActions
   9  WCAG 4.1.2 NameRoleValue
   9  WCAG 2.1.1 Keyboard
TOTAL 155
== by severity:
  76  Error
  63  NeedsReview
  16  Warning
TOTAL 155
== the 14 rows whose name begins with Name, which the header-skipping filter ate:
14
== any row whose last column is not one of the five?
(none)
== any mention of contrast or color in the rule file?
0
```

61 + 53 + 23 + 9 + 9 = 155. 76 + 63 + 16 = 155. Lines 6 to 160 are 155 lines. Three parses agree with each other and with the 2026-09-12 figure. My own first parse answered 156 with a spurious `---` row, because `NR>4` skipped the header and not the separator; the line-range parse replaced it, and the number reported is the one whose parts sum.

**What the file says that the plan did not.** The rules cite exactly three WCAG criteria, and the seventy-six Section 508 rules include the fourteen `Name*` rules and the seven `LocalizedControlType*` rules, which are the rules anybody would call the name and role checks. The file files them under Section 508 502.3.1 Object Information, not under WCAG 4.1.2. The nine rules under 4.1.2 are about control patterns and sibling uniqueness. The page counts what the file says and says that in substance those twenty-one are about 4.1.2's subject.

**And the file mentions neither contrast nor colour**, which made the workflow's header comment, "contrast failures the scanner can measure", a false claim. Corrected in the green commit.

### 55 criteria, from the specification

`https://www.w3.org/TR/WCAG22/` fetched and parsed for every `Success Criterion N.N.N` heading and the `(Level X)` marker in its section:

```
Success Criterion headings: 87
by level: {'A': 31, 'AA': 24, 'AAA': 31, 'none': 1} sum of the three levels: 86
A + AA = 31 + 24 = 55
NO LEVEL: ('4.1.1', 'Parsing (Obsolete and removed)', 'none')
```

87 = 31 + 24 + 31 + 1. The one with no level is 4.1.1 Parsing, whose section the specification marks "(Obsolete and removed)" and says "This criterion no longer has utility and is removed." So Level A and AA is 55, not the 56 that counts 4.1.1 as it was in WCAG 2.0 and 2.1. The fifty-five numbers on the page were diffed against the parsed list: identical in set and in order.

### The regulations, from the note itself

`https://www.w3.org/TR/wcag2ict-22/`, the W3C Group Note of 11 December 2025, says: "This document does not seek to determine which WCAG 2 provisions (principles, guidelines, or success criteria) should or should not apply to non-web documents and software, but rather, if applied, how they would apply." And: "some local standards such as Section 508 in the U.S., and EN 301 549 in Europe, state that WCAG 2.0 Success Criteria 2.4.1 Bypass Blocks, 2.4.5 Multiple Ways, 3.2.3 Consistent Navigation, and 3.2.4 Consistent Identification do not apply to non-web documents and non-web software. In addition, EN 301 549 states that 2.4.2 Page Titled and 3.1.2 Language of Parts do not apply to non-web software." The page attributes the six that way, four to both and two to EN 301 549 only, and names no conformance claim under either.

## Task 1: the page

`docs/wcag-coverage.md`, one commit, `4d415de3`, answered `docs_only`. Fifty-five rows, counted by the reading's own test and by hand: `grep -cE '^\| [0-9]+\.[0-9]+\.[0-9]+ ' docs/wcag-coverage.md` answers 57, because the regulation table's two rows also begin with a criterion number; the reading scopes to the table whose header has an Axe.Windows column and finds 55. Three rows say yes under Axe.Windows, one under the MSAA walk, two under the NVDA suite (3.3.1, where two tests hear an error sentence, and 4.1.3, where three tests hear an announcement), and every row leaves something for a person.

What the page carries beyond the table, each as the plan asked: the two counts with their parts; what a rule count is not; the MSAA script's header quoted, including the sentence about the UI Automation scan being wrong in both directions; the operated roles the walk counts, listed from `$OPERATED`; the three NVDA tests by name and the skipped fourth; thirty-one windows scanned, 1 + 1 + 5 + 1 + 23, and the seventeen nested dialogs by name; that no scan has run against any of the thirty-one on either channel and why; and what has not happened. Zero em dashes by byte search, zero carriage returns by `tr -cd '\r' | wc -c`, none of the six banned words.

**It does not read as a compliance claim, and that took a sentence at the top and one under the table.** "A yes in the table means the scanner has rules that can fail on that criterion. It does not mean the criterion is met, in any window, by this application." The applies column is called a judgement and the six regulatory answers are separated from the forty-nine that are one reader's.

The changelog entry is under `[Unreleased]`, `### Changed`, points at the page and corrects the older line by addition. **The plan's premise 6 was wrong about where that line is.** It said `docs/changelog.md:10621` "sits in a released-version note". The line is now at 10865 and the only version heading above it is `## [Unreleased]` at line 7; it sits under "Added, earlier in this cycle" inside Unreleased. It was still corrected by addition rather than rewritten, because the reason holds either way: a record of what was believed when it was written should stay one.

## Task 2: the reading, red and green, honestly

Red `0ee95483`, answered `red`, seven tests named in `Fails-until-green:` trailers, each ran and failed with nothing else failing. The plausible wrong implementation of a document-reading check is one that reads nothing and reports nothing, so that is what the red half held: the constants and eight tests over a reading that returned no table and no disagreement. The main check, `test_the_page_says_yes_for_exactly_the_criteria_the_code_names`, passed over it. That is the disarmed guard CLAUDE.md describes, and the seven companions are what make it visible; it is the one test not named in the trailers, and it passing at red is the whole reason the others exist.

Green `4abe217d`, answered `all` because it carries the workflow: 7,620 passed, 0 failed, release build included. The reading finds the table whose header has an Axe.Windows column, reads rows until the first line that is not one, keeps those beginning with three numbers joined by dots, finds a channel's column by the start of its heading, and calls a cell a yes when it begins with "yes". Both directions: a criterion the page marks and the code does not name, and one the code names and the page does not mark. The empty page and the empty list each return their own disagreement rather than nothing.

**The companions plant in the real page, not in a made-up table.** `with_the_cell_changed` rewrites one cell of one row of `docs/wcag-coverage.md` as read from disk, so what is proved is the reading over the shape the page really has: 2.4.7 marked yes under Axe.Windows is reported as the page saying yes; 2.1.1 marked no is reported as the code saying yes; and 1.3.1 marked yes under the MSAA walk is reported for that channel, which is the test that a reading finding the first column twice would fail.

**The scan output is held to the code too.** `test_the_scan_output_names_the_criteria_the_code_names_and_not_a_fraction` reads the workflow's commands, with comments left out because 06-06 met a comment reddening a file-reading test, takes the lines that write the step summary, and requires each of the three criterion numbers on one of them, the page named on one of them, and no command line saying "half of WCAG".

`cargo test --lib -- presentation::what_the_scans_can_judge::` at green: 8 passed. `cargo test --test house_style`: 70 passed. `tests/house_style.rs` was not edited; `git diff main -- tests/house_style.rs` is empty.

## The gate hole, and the one line that closes it

`docs/*.md` maps to no scoped target. The plan said so and asked what covers a commit that edits only the coverage page. The answer before the green commit was: the four tree-reading guards, the documents-only list, and `--lib help_page::`. Not the reading that holds that page to the code, which lived in `src/presentation/` and so ran when its own module changed and when the merge ran everything, and never on the one commit that could make the page disagree with the code.

One line in `check.sh`'s `docs_only` path, `cargo test --lib presentation::what_the_scans_can_judge::`, on the precedent of `help_page::` two lines above it, which exists for the same reason. It costs a few seconds, since the library is already built for `help_page`. **Shown working rather than argued:** the corrections commit `93f8c865` touched only documents, answered `docs_only`, and its log holds the eight `what_the_scans_can_judge` tests passing. Deviation, Rule 2, ledger 400.

## Guard record: one, measured, census 760

The break is the page going stale by hand: the 2.1.1 row's Axe.Windows cell changed to "no" with the code untouched. Applied by Edit, then `cargo test --lib --no-fail-fast -- --test-threads=8` on the whole library: **7,190 passed, 2 failed, 1 ignored, 51 seconds**, and exactly these two failed:

- `test_the_page_says_yes_for_exactly_the_criteria_the_code_names`, saying the code says Axe.Windows can judge 2.1.1 Keyboard and the page does not say yes.
- `test_a_fourth_criterion_marked_yes_on_the_page_is_reported`, because its fixture then carried two disagreements where it expects exactly one.

Two rather than the one the plan expected, and the record says why. Reverted with `git checkout -- docs/wcag-coverage.md`, the file being committed. Then run through `scripts/guards.sh "the coverage page cannot say"`: "all 2 tests named went red, and nothing else did", 95 seconds. `guards/guards.toml` holds **760** records by a TOML reader; the census at lines 79 and 80 reads 192 + 568. The count check did not fire: the new module is named by no earlier record, and the new record names it at 8.

## Five sentences corrected, not four, and two left standing

The plan's grep, re-taken:

```
grep -rn "roughly half\|about half\|half of what WCAG\|half of WCAG" docs/*.md .github/workflows/*.yml src -r CLAUDE.md
docs/IMPLEMENTATION_STATUS.md:184   covers roughly half of WCAG          corrected, by name not number
docs/accessibility.md:12            catches roughly half of what WCAG    corrected
docs/changelog.md:10850             It covers roughly half of WCAG       corrected by addition, left as written
docs/principles.md:67               covers roughly half of WCAG          corrected
.github/workflows/accessibility.yml:14   half of accessibility defects   left standing; its contrast clause corrected
.github/workflows/accessibility.yml:319  covers about half of WCAG       corrected, the scan output
CLAUDE.md:31                        half of WCAG                         corrected: a fifth copy the plan did not count
CLAUDE.md:718                       half of accessibility defects        left standing, the qualification added
```

**The fifth copy.** `CLAUDE.md` guardrail 2 said "the automated scan covers about" at the end of line 30 and "half of WCAG" at the start of line 31. The plan's pattern `roughly half\|about half` is a single-line search and the phrase wrapped. Found by searching for `half of WCAG` alone. Wrong the same way the four product copies were, and corrected the same way, with the date and the old wording and a note that a single-line grep missed it, so the next search for a wrapped phrase starts with the rarest word.

**The two "of defects" sentences were both read and both left**, as the plan asked, because that claim is about the share of defects an automated tool finds and is defensible. The orchestrator asked that CLAUDE.md's copy be corrected with the old wording visible; the way that squares with the plan is that the sentence itself stands and the qualification it lacked follows it: that the figure is about defects, not criteria, and what the criteria number is. The workflow's copy had a second problem the plan did not see, "contrast failures the scanner can measure", which the rule file disproves; that clause is corrected and the defects clause stands.

**What a user reads now on `docs/accessibility.md` that they did not before:** that the scans can produce a finding against three named criteria of fifty-five, that the other fifty-two need a person, a link to the page with the row for each, that the scan reaches thirty-one windows, and that no scan has yet run against them. Before, they read that scanning catches roughly half of what WCAG asks for.

The acceptance grep, `grep -rn "roughly half of WCAG\|about half of WCAG\|half of what WCAG" docs/ .github/ src/`, is not empty outside the changelog: it finds `docs/principles.md:70`, `docs/wcag-coverage.md:7` and `CLAUDE.md:32`. Every hit is a dated quotation of the old wording, kept because the brief asked for the old wording visible. No sentence in the tree asserts the fraction.

## REQUIREMENTS.md, three corrections in place

Each dated 2026-09-14, each naming Pratik's answer, each quoting what the line said.

| Requirement | What it said | What is true |
|---|---|---|
| FEEDBACK-01 | "`per_event` (line 392) and `set_event_channels` (line 424) are both **private**, `fn` and not `pub fn`" | `per_event` is a struct field at `feedback.rs:497` and never was a function; `set_event_channels` is `pub fn` at 533 since 06-01 and is written by the settings screen at `wx_settings.rs:2257` |
| FEEDBACK-02 | "what is left is the strings and only the strings", in `date_display.rs` and one line of `wx_item_form.rs` | Four files: those two, plus `signed_mail.rs` where eight sentences wrote the month with `%B`, and `occurrences.rs` where the repeat-series sentence wrote the seven English day names, neither mentioned |
| FEEDBACK-03 | "line 136 records roughly half of WCAG covered" and "nothing in the tree names which WCAG 2.2 AA criteria the scan can and cannot judge" | Three of fifty-five, with the arithmetic; the page, the module and the step summary all name them; the anchors had moved to 178 and 184 |

My first draft of the FEEDBACK-02 correction named `ui_types.rs` as the fourth file. The tree said otherwise: `ui_types.rs` changed only a doc comment, and the item form's month list is in `wx_item_form.rs`, which the paragraph did cite. Corrected before commit, by reading the two merges' diffs rather than their stats.

## Criteria 3 and 4, clause by clause

From `ROADMAP.md`, not from a paraphrase.

**Criterion 3: "The accessibility scan output names which WCAG 2.2 AA success criteria it can and cannot judge, so 'roughly half' becomes a list."** Two clauses. *The scan output names which it can and cannot judge*: the step summary now names the three it can, says the MSAA walk judges the Name part of one, and says the other fifty-two are a person's with the page named for the row of each. It names the fifty-two by count and by pointer rather than listing them in a step summary, which is the reading taken here and said plainly. Closes structurally; a test holds it; no CI run has produced the summary since the change. *"Roughly half" becomes a list*: `docs/wcag-coverage.md`, fifty-five rows. Closes. **Criterion 3 closes structurally, both clauses, and nothing in it has been run.**

**Criterion 4: "The interactions only a human screen reader pass can cover are written down as a scoped list, and each of the five WebView2 findings is either fixed or recorded as upstream with the upstream named."** The page's last column says what is left for a person on each row, which is material for the scoped list and is not the list; and the five findings are not touched here. **Criterion 4 closes nothing here**; both clauses are 06-08's.

## What the gate selected for each file

| file | `which-checks.sh` | what ran |
|---|---|---|
| `docs/wcag-coverage.md`, `docs/changelog.md` | `docs_only` | fmt, clippy, the shell suites, `--lib help_page::`, the seven document-reading targets; after the green commit, the reading too |
| `src/presentation/what_the_scans_can_judge.rs`, `mod.rs` | `red`, then part of `all` | at red, `--lib presentation::` and `--lib presentation::what_the_scans_can_judge::` and the four tree guards, held to the seven named |
| `.github/workflows/accessibility.yml` | `all` | the whole gate on the green commit, 7,620 tests and the release build |
| `scripts/check.sh`, `guards/guards.toml` | in the green commit's `all` | the shell suites in every mode; the six record checks in `house_style` |
| `docs/accessibility.md`, `docs/IMPLEMENTATION_STATUS.md`, `docs/principles.md`, `CLAUDE.md` | `docs_only` | as the first row, now including the eight coverage tests |
| `.planning/*.md`, `06-07-SUMMARY.md` | `docs_only` | as the first row |

`scripts/check.sh all` was run once on the branch before the merge, not piped, and the merge commit ran the hook again; both figures are in the follow-up commit.

**The gate refused nothing on the code commits.** It refused the corrections commit once, by hand before the commit rather than in the hook: `test_no_status_page_names_a_version_the_code_does_not_ship` read 1.3.1, 2.1.1 and 4.1.2 on `docs/IMPLEMENTATION_STATUS.md` as versions the code does not ship. The guard reads that page for version numbers and a criterion number has the shape of one. The page names the three by name instead and says why, and the guard's file is untouched. Ledger 399.

## Deviations from plan

**1. [Rule 2] One line in `scripts/check.sh`.** Not in the plan's file list. Without it the check the plan asked for ran on every commit except the one that could break the page it reads. Shown working on `93f8c865`. Ledger 400.

**2. Five corrected sentences, not four.** `CLAUDE.md:30-31` wrapped. And CLAUDE.md's defects sentence gained a qualification where the plan said leave it, on the orchestrator's instruction, with the sentence itself standing. Ledger 401.

**3. [Rule 1] The workflow header's contrast claim.** The plan named line 14 as "leave, defensible". Its second half claimed the scanner measures contrast; the rule file has no rule mentioning contrast or colour. The defects clause stands, the contrast clause is corrected with the date.

**4. Two tests red under the guard record, not one.** Measured rather than predicted, and the record says why.

**5. The status page names criteria by name.** A guard read the numbers as versions.

**6. The changelog premise.** Line 10621 had moved to 10865 and sits under `[Unreleased]`, not in a released-version note. Corrected by addition as the plan said, for a reason that holds either way.

**7. One whole-file revert, `git checkout -- docs/wcag-coverage.md`, after the planted break**, the file being committed at the time. Every other change to a tracked file was Read then Edit or Write. The rule file, the specification and the note were fetched to the scratchpad and parsed there, never executed, and no tracked file was written by a script. **Exception set for scripted rewrites: zero.** Carriage returns in every touched file: 0 by `tr -cd '\r' | wc -c`. `cargo fmt` reformatted the new module once, which is the project's formatter and runs in the gate.

**8. No `docs/changelog.md` version bump.** A new page and corrected sentences are a docs pass, which CLAUDE.md says does not bump, and the plan said the same. The changelog entry is there because a page a user reads changed.

**9. `roadmap update-plan-progress` and `state advance-plan` were not run**; `ROADMAP.md`, `STATE.md` and the phase README were edited by hand and the diff read, as 06-06 did, because the orchestrator said the tool is broken. `progress.completed_plans` is 96, counted as `*-SUMMARY.md` on disk with this file.

## What 06-08 inherits

- The page's last column, fifty-five cells of "what is left for a person", is the raw material for criterion 4's scoped list and is not that list.
- The scan's step summary names the three criteria the first time it runs. That run is 06-08's first artifact and the first evidence the summary reads as intended.
- The applies column is one reader's judgement, ledger 396; the row a person disagrees with is the row to correct, dated.
- The NVDA README under-claims its own suite, ledger 398, out of scope here.
- The version-reading guard cannot tell a criterion number from a version, ledger 399; on a status page, name criteria by name.

## Ledger

`.planning/WINDOWS.md` 395 to 401 written through `gsd-tools windows append`, both halves at 401, no backslash in any description, no carriage return.

| id | kind | what |
|---|---|---|
| 395 | unrun-verify | fifty-two criteria only a person can judge, and nobody has walked any |
| 396 | todo | the applies column is one reader's judgement about applicability, unchecked against this application |
| 397 | todo | the seventeen nested dialogs are outside both channels; the page depends on 394 |
| 398 | todo | the NVDA README names two tests where four exist, found out of scope |
| 399 | todo | the version-reading guard reads a criterion number as a version; the status page names criteria by name |
| 400 | deviation | one line in `check.sh`'s documents-only path |
| 401 | deviation | two CLAUDE.md copies where the plan counted one; the defects sentence qualified rather than left |

## Known Stubs

None. The page is wired to a check, the check to the gate, the constants to the page and the workflow, and each was seen to fail.

## Threat Flags

None new. T-06-25 is mitigated: the grep is quoted and every remaining hit is a dated quotation. T-06-26 is mitigated further than the register planned, because the gate now runs the reading on a page-only commit. T-06-27: read from the pinned tag, with the file's hash recorded above. T-06-28: the note quoted saying it does not decide, and the regulations named. No package was added; the three fetches were read and parsed in the scratchpad, never executed.

## What was not done, said plainly

- No scan has run against any window on either channel. Nothing pushed.
- Nobody has walked any of the fifty-five criteria against this application. The page is a reading of each criterion against what a mail client does.
- The five WebView2 findings are untouched; the scoped manual list is not written. Both are 06-08's.
- The NVDA README's own description of its suite is wrong and was not corrected here.

## Self-Check: PASSED

`docs/wcag-coverage.md`, `src/presentation/what_the_scans_can_judge.rs`, `.github/workflows/accessibility.yml`, `scripts/check.sh`, `guards/guards.toml`, `docs/accessibility.md`, `docs/IMPLEMENTATION_STATUS.md`, `docs/principles.md`, `docs/changelog.md`, `CLAUDE.md`, `.planning/REQUIREMENTS.md` and this file exist on disk; commits `4d415de3`, `0ee95483`, `4abe217d` and `93f8c865` are in `git log --all`.

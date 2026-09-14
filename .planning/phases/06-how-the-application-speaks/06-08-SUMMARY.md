---
phase: 06-how-the-application-speaks
plan: 08
status: complete
subsystem: accessibility-scan
tags: [accessibility-scan, axe-windows, msaa, webview2, findings, manual-pass, ledger, tdd]

requires:
  - phase: 06-how-the-application-speaks
    provides: "06-06's thirty-one targets and pinned scanner, which is what ran; 06-07's coverage page, which is where the findings section sits"
provides:
  - "docs/wcag-coverage.md: the run of 2026-09-14, twenty-nine findings each with a row, the WebView2 attribution from the artifact, what would be filed and where, three findings about the scan itself, and what the scan did not reach"
  - "docs/manual-accessibility-pass.md: seventy-six items across six categories, each with its source and its technology, saying at the top that none of it has happened"
  - "src/presentation/editor_document.rs: the editor page has a title, so WebView2 does not name its host and root after the page's own address"
  - "src/presentation/wx_managers.rs and wx_account_manager.rs: status lines built empty rather than with a space"
  - "tests/no_label_is_only_a_space.rs: a fifth whole-tree guard, in the gate's list"
  - ".github/workflows/accessibility.yml: the count reads the singular Axe prints, held by a test in scan_target.rs"
  - "docs/IMPLEMENTATION_STATUS.md and docs/changelog.md: the count of five replaced, and corrected by addition, with why it moved beside the number"
affects: [08, ci, gate]

actuals:
  tokens: 21400
  tasks: 2
  commits: 10

tech-stack:
  added: []
  patterns:
    - "A finding attributed upstream is attributed from the artifact's tree and provider descriptions, never from a class name"
    - "A guard that reads the whole tree for a copied shape is its own target when the obvious home is counted by many records"
    - "A companion that plants an absence removes the entry wherever it sits, not by matching its neighbour"

key-files:
  created:
    - docs/manual-accessibility-pass.md
    - tests/no_label_is_only_a_space.rs
  modified:
    - docs/wcag-coverage.md
    - docs/IMPLEMENTATION_STATUS.md
    - docs/changelog.md
    - src/presentation/editor_document.rs
    - src/presentation/wx_managers.rs
    - src/presentation/wx_account_manager.rs
    - src/presentation/scan_target.rs
    - .github/workflows/accessibility.yml
    - scripts/check.sh
    - tests/the_words_that_say_nothing.rs
    - guards/guards.toml
    - Cargo.toml
    - Cargo.lock
    - .planning/WINDOWS.md

key-decisions:
  - "Nine of the twenty-nine fixed here, three RED/GREEN pairs: the editor page's title, the three status lines built with a space, and the workflow's counting pattern. Each small, each test first, each confirmable only by the next run on main"
  - "The fourteen spinner and list text fields are ledgered with two recipes, direct annotation through IAccPropServices or a visible label before each field, not fixed: neither is small, neither can be verified here, and the annotation route adds three Win32 features"
  - "-First 1 left in the workflow and ledgered, because nothing in the tree can run that block to prove a sum, and the brief said ledger rather than fix untested"
  - "The whitespace guard is its own target rather than two tests in house_style, which twenty-one records count"
  - "Version 0.123.1: a name somebody can hear changed"

patterns-established:
  - "The manual pass groups two hundred and fifty-seven unheard things into walks, each item naming the ledger entries it closes, rather than listing entries"

requirements-completed: [FEEDBACK-03]

coverage:
  - id: D1
    description: "Every finding the scan produced has its own row with element, window, channel, rule and disposition"
    requirement: FEEDBACK-03
    verification:
      - kind: other
        ref: "docs/wcag-coverage.md, the table under What the run of 2026-09-14 found, twenty-nine rows against a11y-findings-by-window.txt's twenty-nine lines"
        status: pass
    human_judgment: true
  - id: D2
    description: "The WebView2 attribution is confirmed against the artifact rather than the names"
    requirement: FEEDBACK-03
    verification:
      - kind: other
        ref: "el.snapshot of the compose target: the six sit under Chrome_WidgetWin_1 whose parent is wxWindowNR -31900 'Message body'; every one reports msedge.dll as provider; src/, scripts/ and .github/ hold none of the six class names"
        status: pass
    human_judgment: false
  - id: D3
    description: "The count of five is replaced in the status page and corrected by addition in the changelog, with why it moved beside the number"
    requirement: FEEDBACK-03
    verification:
      - kind: other
        ref: "grep -n 'Five accessibility scan findings' docs/IMPLEMENTATION_STATUS.md finds nothing; docs/changelog.md Unreleased Changed, first entry"
        status: pass
    human_judgment: false
  - id: D4
    description: "The counting pattern reads all three sentences Axe prints"
    requirement: FEEDBACK-03
    verification:
      - kind: unit
        ref: "presentation::scan_target::tests::test_the_workflow_counts_one_error_as_one_and_not_as_none, red at 65680c4c, green at 05c15bf0"
        status: pass
      - kind: unit
        ref: "presentation::scan_target::tests::test_the_reading_can_see_a_pattern_that_misses_the_singular"
        status: pass
    human_judgment: false
  - id: D5
    description: "The manual list is scoped, by category, each item with its source and its technology, and says none of it has happened"
    requirement: FEEDBACK-03
    verification:
      - kind: other
        ref: "docs/manual-accessibility-pass.md, seventy-six items, first paragraph; cargo test --test house_style --test docs_links"
        status: pass
    human_judgment: true
  - id: D6
    description: "Nothing this phase produced claims a manual pass has been done"
    requirement: FEEDBACK-03
    verification:
      - kind: other
        ref: "the page's first sentence; WINDOWS.md 430, unrun-verify"
        status: pass
    human_judgment: true

metrics:
  duration: "about 3 hours 50 minutes, 2026-09-14 12:00 to 15:50 UTC"
  completed: 2026-09-14
---

# Phase 6 Plan 8: The findings the scan really produces, judged one at a time, and the list only a person can walk

**Twenty-nine findings from the first scan of thirty-one windows on two channels, each with a row and a disposition: nine fixed test-first on the branch, four WebView2's own with the upstream named, fifteen this program's and ledgered, one unjudged. The scan's own undercount fixed and held by a test. The manual pass written as seventy-six items and not walked.**

Branch `twenty-nine-findings-each-with-a-row`, merged into `main` at `67437b79` with the whole gate green on the merge, 7,656 passed and none failed. Version 0.123.1.

## How the checkpoint was discharged

The plan's checkpoint asked Pratik to dispatch the Accessibility workflow. What happened instead: on Pratik's word, `main` was pushed to `origin` on 2026-09-14 at 13:29 UTC, 263 commits, `1aba3a5b..db98094c`, and the workflow ran on the push, which is one of its own triggers. No agent dispatched anything; the push was Pratik's decision and the run was the workflow doing what its `on:` block says. The plan's premise 3, that a feature-branch push does not run the scan, is true and did not apply: the run was on `main`.

Run 34849526207, `main` at `db98094c`, concluded success, every step green. CI run 34849526286 went green on all seven jobs in the same push. Axe.Windows 2.4.2, the zip's SHA-256 matching the pin, "version 2.4.2" printed on every target. All thirty-one targets wrote a result file and completed their MSAA walk; `$broken` was empty.

## What the run found, and what each finding became

| Where | UI Automation | MSAA unnamed | What became of them |
|---|---|---|---|
| Compose | 6 | 0 | 2 fixed (the page's title), 4 WebView2's own |
| Account Manager | 3 | 0 | 2 fixed (status line and the grip named after it), 1 ledgered (empty IMAP cell of the fixture account) |
| Contact Manager | 2 | 0 | 2 fixed |
| Filters, Tags, Signatures | 1 each | 0 | 3 fixed, one line of code |
| Edit Event | 9 | 8 | 8 ledgered (spinner text fields and list value elements), 1 unjudged (Category's ExpandCollapse) |
| When should this message go? | 6 | 4 | 6 ledgered |
| The other 23 | 0 | 0 | nothing |

Twenty-nine on UI Automation, twelve on MSAA; ten of the twelve are among the twenty-nine and two are not, the Start time Hour and End time Hour fields, which UI Automation named after the static before them, a present and wrong name no rule sees. Both ledgered.

**Fixed, three RED/GREEN pairs, each its own commit:**

1. `b6b4046c` red, `ec98fdd7` green: `editor_document.rs` carries `<title>Message body</title>`. WebView2 named the host window and the page root after the page's address, which was the whole page as a data: URL, base64 and all, over 4,000 characters. Rows 1 and 2. Changelog entry, version 0.123.1, one record re-measured.
2. `8bcf585b` red, `d26382e9` green: the three status lines built with `" "` are built with `""`. A space was the control's name on both channels and Windows named the resize grip after it. Rows 8 to 14, seven findings. The guard is `tests/no_label_is_only_a_space.rs`, a fifth target in `check.sh`'s whole-tree list, with a companion proving the reading sees the shape and a record measured red on its suite. Census 575, 767 records.
3. `65680c4c` red, `05c15bf0` green: the workflow's pattern reads `1 error was found`. Two tests in `scan_target.rs`, one reading the pattern from the workflow and one showing the old pattern missing the singular; the pattern also checked once in PowerShell's own regex engine from a scratch script. Two records re-measured.

Each fix is confirmed only when the next run on `main` reads the tree. Nothing was pushed.

**Ledgered, not fixed:** rows 7, 15 to 22, 24 to 29, and the two hour fields, ledger 407 to 415 and 418 to 425. The spinner family is one cause: `set_accessible_name` on a `SpinCtrl` names the up-down window, and the buddy edit where focus lands is a separate window with no wxWindow behind it. Ledger 408 carries both recipes. Direct annotation: `UDM_GETBUDDY` on the handle `get_handle` returns, then `IAccPropServices::SetHwndPropStr` with `PROPID_ACC_NAME`, which the Annotation proxy in every provider description carries to both channels; needs three more `windows` crate features and can be verified only by the scan. Or a visible static label before each field, which the platform hands to the field on both channels and which the tree already shows working for the hour fields, at the cost of a layout change in two dialogs. Neither is small enough to do without a run to read, and the brief said ledger with what it takes.

**Unjudged:** row 23, ledger 416. Every combo box in Edit Event is exposed through the MSAA proxy because each carries a `FixedName`, and the proxy offers ExpandCollapse to none; only Category was flagged. The one difference in the tree is that it has no child showing a value, and it is six pixels tall. Judging it needs the scanner's rule condition or a scan on a taller screen. The six-pixel height is its own finding, ledger 417: the form runs off a 768-pixel screen and does not scroll.

## The WebView2 attribution, and how it was confirmed

Not from the class names. `a11y-artifact/compose/*.a11ytest` unpacked to `el.snapshot`, walked with a script in the scratchpad: the six elements sit under the pane `Chrome_WidgetWin_1`, whose parent is a pane of class `wxWindowNR` with automation id `-31900` and the name "Message body", the wxWebView wrapper this program builds. Everything from `Chrome_WidgetWin_1` down reports `Unidentified Provider (unmanaged:msedge.dll)`, has no automation id, and has framework id `Chrome` for the views. `grep -rn` for the six class names over `src/`, `scripts/` and `.github/` finds none of them. So the four zero-size views are the Edge runtime's own objects; nothing in this tree sets a rectangle on them.

The two names were different. Their value was this program's page address, and Chromium falls back to the address when a page has no title; the reader's blank page is short enough not to trip the 512 rule, and the reader target's window is a rich edit, so the composer was the only place it showed. That is why rows 1 and 2 are fixed here and 3 to 6 are not. Whether a title really replaces the address in the host's name is a claim the next scan tests; the changelog and the coverage page both say so.

**Upstream:** `MicrosoftEdge/WebView2Feedback`, for all four. What would be filed is written on the coverage page: four Views inside an embedded WebView2 reporting `IsOffscreen = false` and a null `BoundingRectangle`, which Axe.Windows 2.4.2 reports as an error on any application that embeds the control, with the six element dumps as evidence. Whether an existing issue covers zero-size views was not checked: this session has no way to read GitHub. Issue 2330 there, which the research found, is about screen readers and WebView2 broadly and is not about this. Nothing filed.

The artifact carries nothing of anybody's: every address in it is `example.com`, and the names are the fixtures'. T-06-32 checked rather than assumed.

## The three findings about the scan itself

1. **The singular.** Fixed and tested, above. Ledger 429, closed the same commit.
2. **`Select-Object -First 1`.** Left, with a comment beside it saying why, ledger 426. The reader wrote `_1_of_2` and `_2_of_2`, both 0, so nothing was lost this run. A test would need the counting moved into a script under `scripts/` with a suite of its own before the line changes.
3. **The six module targets are one scan repeated.** Judged from the six `msaa-names.json` files, 91 identical distinct names each; the walk reads no `accState`, so a hidden panel is walked like a shown one. Ledger 427 with the recipe: skip a subtree whose state has `STATE_SYSTEM_INVISIBLE`, print how many were skipped. Not widened into, because `scripts/*.ps1` maps to no gate target and the change wants a shell case first.

Also from the tree and not from any rule: the preview's WebView2 document was in no target's tree, so the rendered message, the one place a sender's structure has to survive, was judged by nothing. Ledger 428, and item A10 of the manual pass.

## Task 2: the manual pass

`docs/manual-accessibility-pass.md`, seventy-six items: 41 for screen readers, 9 for low vision and colour, 8 for physical and motor, 10 for learning and cognitive, 5 for hearing, 3 for vestibular and photosensitivity. The first sentence is "None of this has happened."

Three sources, each item naming its own in brackets: the coverage page's rows whose last column is a person's; the ledger's open `unrun-verify` entries, 257 on 2026-09-14 counted from the JSON half, grouped into walks with each walk naming the entries it closes; and the six categories, where neither had asked. The three running NVDA tests are named with what each hears, the skipped `which-days-focus-and-tick` is named with why it is still skipped, and it is item A25.

Why NVDA and Narrator are not interchangeable here, as the page says it: NVDA reads MSAA, the only channel `set_accessible_name` writes; Narrator reads UI Automation, where Windows' own provider for a native control answers with the window text or the label beside it. The scan of the same day is the proof on the page: the Edit Event hour fields are named "Start time" on one channel and nothing on the other, and the day fields nothing on both while the arrows beside them say "Starts Day". Every item says NVDA, Narrator, Both, Eyes, Keyboard, Ears or Tool.

The two pages link to each other in three places. Changelog entry under Added. Ledger 430.

## The count of five

`docs/IMPLEMENTATION_STATUS.md` corrected in place: twenty-nine from thirty-one windows on two channels, with the sentence that five was one window on one channel and the number is larger because the scan looks at more. `docs/changelog.md` corrected by addition under `[Unreleased]`, Changed, with the same sentence; line 10668's shipped entry untouched. `grep -n "Five accessibility scan findings" docs/IMPLEMENTATION_STATUS.md` finds nothing.

## Counts, before and after

| What | Before | After |
|---|---|---|
| `.planning/WINDOWS.md` highest id | 406, counted, not the 409 the brief guessed | 430; 407 to 430 written, 429 closed |
| `guards/guards.toml` records by a TOML reader | 766 | 767; census line 574 to 575 |
| Records re-measured | | 3, each still exactly what it names |
| Version | 0.123.0 | 0.123.1 |
| Tests in the whole gate | 7,649 | 7,656 passed, 0 failed |

## What the gate selected for each file

| Files | What the hook ran |
|---|---|
| `src/presentation/editor_document.rs` alone, red | fmt, clippy, the script suites, `--lib presentation::editor_document::` held to the two named, the whole-tree guards |
| the same with `docs/changelog.md`, `Cargo.toml`, `Cargo.lock`, `guards/guards.toml`, green | the same scoped run; the manifest diff was only the package's own version, so the gate did not answer `all` for it |
| `tests/no_label_is_only_a_space.rs` alone, red | fmt, clippy, the suites, `--test no_label_is_only_a_space` held to the two named, the whole-tree guards |
| `wx_managers.rs`, `wx_account_manager.rs`, `scripts/check.sh`, `guards/guards.toml`, `tests/the_words_that_say_nothing.rs`, `docs/changelog.md`, green | `--lib presentation::wx_account_manager::` and `::wx_managers::`, `the_words_that_say_nothing`, `manager_dialog_labels` and `manager_delete_stays_open` coupled by the records, the whole-tree guards |
| `src/presentation/scan_target.rs` alone, red | `--lib presentation::scan_target::`, the whole-tree guards |
| `.github/workflows/accessibility.yml`, `guards/guards.toml`, green | the whole gate: audit, 7,225 library tests and every target, the release build |
| `docs/*.md`, `.planning/WINDOWS.md`, both document commits | `docs_only`: fmt, clippy, the suites, the document-reading targets |
| the merge | the whole gate again, 7,656 passed |

`scripts/check.sh all` was run once on the branch before the merge, its output redirected to a file and its own exit status read, never piped: 0, 7,656 passed. The `keyring` race of ledger 374 did not appear.

**The gate refused twice, both on green commits, both the count check.** Once for the record the editor test moved, once for the new record whose `file` had not been counted. Each time the remedy it printed was run and the commit remade. It refused a third thing on the second green commit: `test_the_reading_of_what_the_gate_runs_can_see_this_target_missing` in `the_words_that_say_nothing.rs`, which took its own name out of the whole-tree list by replacing ` name)`, the name only while it is last. Widened to match the name wherever it sits, deviation 2 below.

## Deviations from plan

**1. [Rule 1] Three findings fixed in `src/` and one in the workflow, four RED/GREEN pairs where the plan expected two documents-only commits.** The plan said fix what is small; these were. Each is its own commit, test first, and this summary says which. The version bump the plan said was not owed is owed by the first, since a name somebody can hear changed.

**2. [Rule 1] `tests/the_words_that_say_nothing.rs`, five lines.** Its companion assumed its target was last in the gate's list and broke when a fifth was appended, blaming the reading. Fixed to remove the name wherever it sits, with the reason in a comment. Observation logged.

**3. [Rule 2] `scripts/check.sh`, the whole-tree list.** A fifth guard target, so the whitespace guard runs on the commits that could break it, with a comment saying why.

**4. `-First 1` not fixed.** The brief allowed a fix with a test; no test reaches the block; ledgered with the recipe and said in the workflow comment.

**5. The ledger's before count was 406, not the 409 the brief said.** Counted from the table and the JSON.

**6. The brief's "roadmap update-plan-progress is broken":** ROADMAP.md, STATE.md and the phase README edited by hand, diff read.

**7. Scripted edits: exception set zero.** Every tracked file was changed with Read then Edit or Write. The artifact was unpacked and walked with a script in the scratchpad; the PowerShell pattern was checked from a scratch script; `cargo fmt` reformatted `scan_target.rs` once, which is the project's formatter and runs in the gate; `gsd-tools windows append` wrote the ledger, which is the sanctioned writer. Carriage returns in every touched file: 0 by `tr -cd '\r' | wc -c`. Em dashes in every touched document: 0 by a byte search.

**8. No issue filed anywhere.** What would be filed and where is on the coverage page.

## Ledger

`.planning/WINDOWS.md` 407 to 430 through `gsd-tools windows append`, no backslash in any description, both halves matching, 429 marked fixed.

| id | kind | what |
|---|---|---|
| 407 | todo | Account Manager row 7, the empty IMAP Server cell of the fixture account |
| 408 to 410 | todo | Edit Event rows 15 to 17, the Starts Day, Starts Year and Start time Minute text fields; 408 carries the two recipes |
| 411 | todo | Edit Event row 18, the Start time AM or PM value element |
| 412 to 414 | todo | Edit Event rows 19 to 21, the Ends fields |
| 415 | todo | Edit Event row 22, the End time AM or PM value element |
| 416 | todo | Edit Event row 23, Category's ExpandCollapse, unjudged |
| 417 | todo | Edit Event runs off a 768-pixel screen, four fields six pixels tall, found in the tree |
| 418 to 423 | todo | send-later rows 24 to 29 |
| 424, 425 | todo | the two hour fields named after the static before them, present and wrong, no rule fired |
| 426 | todo | scan-level 2, `-First 1` |
| 427 | todo | scan-level 3, the six module targets walk hidden panels |
| 428 | unrun-verify | the rendered message was in no target's tree |
| 429 | todo, fixed | scan-level 1, the singular miscount |
| 430 | unrun-verify | the manual pass is written and unwalked |

## Known Stubs

None. The title is in the page the composer loads; the empty labels are in the three builders every manager window uses; the guard is in the gate's list and was seen red; the pattern is in the workflow's own line and was seen red. Nothing built here waits on a caller.

## Threat Flags

None new. T-06-29 mitigated: the attribution is from the artifact's providers and parents, and this summary says how. T-06-30 mitigated: the sentence about one window on one channel sits beside the new number in both places. T-06-31 mitigated: seventy-six items, by walk, three named sources, technology per item, the automated coverage named. T-06-32 checked: `example.com` throughout. T-06-SC: no package added; three `windows` features were considered for the spinner fix and not added.

## What was not done, said plainly

- Nothing was pushed, so no run has confirmed any of the nine fixes. The next push to `main` is what does.
- The fourteen spinner and list text fields still have no name where focus lands. The dialogs that ask when a message goes and what an event is called are, on the channel NVDA reads, six and twelve unnamed fields with named arrows beside them.
- The Category finding is not judged.
- `-First 1` and the hidden-panel walk are recorded, not fixed.
- Nobody has walked any of the seventy-six items. Nobody has heard any of this.
- No issue is filed with `MicrosoftEdge/WebView2Feedback`, and whether one already exists for the zero-size views is not known.

## Self-Check: PASSED

Files: `docs/manual-accessibility-pass.md`, `tests/no_label_is_only_a_space.rs`, `docs/wcag-coverage.md` present. Commits `b6b4046c`, `ec98fdd7`, `8bcf585b`, `d26382e9`, `65680c4c`, `05c15bf0`, `13251ca9`, `5719f589`, `67437b79` present in `git log --all`.

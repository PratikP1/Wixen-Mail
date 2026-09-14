---
phase: 06-how-the-application-speaks
plan: 06
status: complete
subsystem: infra
tags: [commit-gate, which-checks, accessibility-scan, workflow, axe-windows, msaa, scan-target, checkpoint]

requires:
  - phase: 07-installing-updating-and-what-is-stored
    provides: "07-02's `*.iss` rule in `scripts/which-checks.sh`, the shape task 1's rule copies; 07-07's two records naming `release.yml`, the shape task 2's two records copy"
provides:
  - "`scripts/which-checks.sh`: any path under `.github/workflows/` answers `all` on a branch, below the manifest block, keyed on the folder"
  - "`.github/workflows/accessibility.yml`: Axe.Windows v2.4.2 pinned by tag and by the zip's SHA-256, thirty-one windows in the target array, a check that the application is still running before each scan, `--alwayssavetestfile`, and any MSAA exit other than 0 or 1 recorded as a walk that failed"
  - "`src/presentation/scan_target.rs`: thirty targets, `WINDOW_NOT_OPEN`, `OnReturn`, and five tests that read the workflow or the names"
  - "`src/presentation/scan_fixtures.rs`: the made-up data six windows open on, each with a test for the property that decides whether the window opens"
  - "`scripts/msaa-names.ps1`: every visible top-level window the process owns is walked, and a failed walk exits 2"
  - "Both checkpoint answers, recorded with the date and Pratik's words"
affects: [06-07, 06-08, gate, ci]

actuals:
  tokens: 14468
  tasks: 2
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A modal target's call returning during a scan means the window is not open, and the program leaves with a code the workflow reads as not scanned rather than let the scan walk the main window"
    - "A window that refuses to open on nothing opens on a fixture in `scan_fixtures`, and the fixture's test asserts the one property the window refuses without"
    - "A third-party binary in CI is pinned by tag and by the hash of the file, with the endpoint, the date and the reason beside it"

key-files:
  created:
    - src/presentation/scan_fixtures.rs
  modified:
    - scripts/which-checks.sh
    - scripts/which-checks.test.sh
    - .github/workflows/accessibility.yml
    - src/presentation/scan_target.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_send_later.rs
    - scripts/msaa-names.ps1
    - guards/guards.toml

key-decisions:
  - "Pin by tag and by SHA-256, both read in this session: `v2.4.2` from `https://api.github.com/repos/microsoft/axe-windows/releases`, hash `aeca43f41c89b3ffb1db84011539e609ecd7cb3badd6e78fada2ada327d10a64` from sha256sum and Get-FileHash agreeing"
  - "Thirty targets, not the nine or fourteen the plan and its checkpoint counted: fourteen top-level dialogs read from the tree, the bare main window, and its five other module panels; every one opened on a fresh profile here and none is ledgered as unreachable"
  - "`main` is left as the frame under the first-run question, because that is what a fresh profile meets; `mail-module` is the bare window"
  - "The MSAA script walks every top-level window rather than .NET's main window, because the CI log showed three dialogs with the same census and the main window is the one window a dialog can never be"
  - "`--alwayssavetestfile`, because the CLI's own help says the file is written only when errors are found and the workflow's no-file check had reported seven clean windows as failed scans"

patterns-established:
  - "A red test that reads a file matches the file's commands and not its comments, or the comment explaining the fix reddens it"

requirements-completed: []

coverage:
  - id: D1
    description: "A commit touching any file under .github/workflows/ answers all on a branch"
    requirement: FEEDBACK-03
    verification:
      - kind: unit
        ref: "scripts/which-checks.test.sh::a workflow file on a branch"
        status: pass
    human_judgment: false
  - id: D2
    description: "The scanner is one named release and one hashed binary"
    requirement: FEEDBACK-03
    verification:
      - kind: unit
        ref: "presentation::scan_target::tests::test_the_workflow_pins_the_scanner_to_a_release_and_its_checksum"
        status: pass
      - kind: other
        ref: "guards.toml: the scanner's checksum is compared with a number written down and not with itself, measured red on the whole library"
        status: pass
    human_judgment: false
  - id: D3
    description: "Every window a fresh profile can reach has a target, and the workflow asks for every target"
    requirement: FEEDBACK-03
    verification:
      - kind: unit
        ref: "presentation::scan_target::tests::test_every_window_a_fresh_profile_can_reach_has_a_name"
        status: pass
      - kind: unit
        ref: "presentation::scan_target::tests::test_the_workflow_asks_for_every_target"
        status: pass
      - kind: other
        ref: "thirty targets each started on a throwaway profile on this machine, the process's visible top-level windows listed, the named window present for every one"
        status: pass
    human_judgment: false
  - id: D4
    description: "A target whose window is not open is reported as not scanned rather than scanned"
    requirement: FEEDBACK-03
    verification:
      - kind: unit
        ref: "presentation::scan_target::tests::test_the_workflow_asks_whether_the_application_is_still_running_before_it_scans"
        status: pass
      - kind: other
        ref: "About window closed from outside with WM_CLOSE while the app waited on it: HasExited true, ExitCode 3"
        status: pass
    human_judgment: false
  - id: D5
    description: "Both channels read each target in CI"
    requirement: FEEDBACK-03
    verification:
      - kind: other
        ref: "no CI run since 2026-09-10; nothing pushed; 06-08 reads the first artifact"
        status: unrun
    human_judgment: true
    rationale: "A target can only be seen to work by a CI run this phase cannot read, and the MSAA walk cannot be run against this application on this machine while NVDA is running"

duration: task 1 about 1h; task 2 about 2h45m to the green commit, about 3h30m with the documents
completed: 2026-09-14
---

# Phase 06 Plan 06: The scan is reproducible, and a change to it earns its checks Summary

**The accessibility scan now runs one named, hashed Axe.Windows binary over thirty windows where it ran whatever was newest over eleven, a change to the workflow earns the whole gate on a branch, and three defects in the scan itself that the plan did not know about were measured from the last CI log and the running program and fixed: the scanner wrote no file for a clean window and the workflow called that a failed scan, the MSAA channel had never read a dialog, and a dialog that failed to open was scanned as the main window and passed.** None of the thirty has been scanned by CI, because nothing has been pushed since 2026-09-10; every one has been opened on a fresh profile on this machine and its window seen.

Task 1 on branch `a-workflow-change-earns-the-checks-that-read-it`, merged into `main` at `9f86ba6f`. Task 2 on branch `a-pinned-scanner-and-every-window-it-can-reach`: red `741d2b36`, green `fd661401`, the whole gate green on the green commit in 281 seconds under rustc 1.98.1 with 7,612 tests passed and 0 failed, the release build included; `scripts/check.sh all` run again on the branch after the documents commit, 7,612 passed in 295 seconds. Merged into `main` at `ca88d833` with the whole gate green again on the merge, 302 seconds, written here by the follow-up commit since a summary committed before its own merge cannot name it. Nothing pushed.

## The checkpoint's two answers, recorded and not re-asked

Both answered by Pratik on 2026-09-14.

**Pin the scanner: yes, with the hash.** Pin the Axe.Windows release tag and record the zip's SHA-256 beside it, so the coverage list is a claim about a binary rather than about a name somebody can move. The task-1 executor proposed the hash and it was adopted.

**How many windows: all of them.** He was first told nine and chose all nine. Told the corrected count of at least fourteen and that some might not be reachable from a fresh profile, he said: **"Yes, all of them, and record any that can't be reached."**

## Which clauses of criteria 3 and 4 this closes

Read from `ROADMAP.md`. Criterion 3: the scan output names which WCAG 2.2 AA criteria it can and cannot judge, and "roughly half" becomes a list. Criterion 4: the interactions only a human pass can cover are a scoped list, and each of the five WebView2 findings is fixed or recorded upstream with the upstream named. **This plan closes none of the four clauses.** It is the ground under 06-07's list: the list can now name the rule set it was read against, `v2.4.2` and its `axe-windows-rules-2.4.2.md`, and say which thirty windows the scan looks at and which seventeen nested dialogs it does not. Criterion 4's five findings need an artifact, and the first artifact from this scan is 06-08's.

## Task 2: what landed

### The pin

`https://api.github.com/repos/microsoft/axe-windows/releases?per_page=5` was called on 2026-09-14 and answered `v2.4.2` (2024-11-01), `v2.4.1`, `v2.4.0`, `v2.3.1`, `v2.3.0`; `releases/latest` answered `v2.4.2`. Each release ships `AxeWindowsCLI-<version>.zip`, `.msi`, and `axe-windows-rules-<version>.md`, the rule list for exactly that version, which is what 06-07 should read against rather than `main`'s `RulesDescription.md`. The zip is 34,751,526 bytes and hashes to `aeca43f41c89b3ffb1db84011539e609ecd7cb3badd6e78fada2ada327d10a64` by `sha256sum` and by `Get-FileHash -Algorithm SHA256`, which agreed. `AxeWindowsCLI.exe` sits at the zip's root, where the existing `Expand-Archive` puts it.

What pinning with a hash took: eleven lines of PowerShell replacing four. The tag, the zip name derived from it, the expected hash, `Invoke-WebRequest` of the release's download URL, `Get-FileHash`, a `throw` naming both hashes when they differ before anything is expanded, and a `throw` if the exe is not at the root after expansion. The comment beside it names the endpoint, the date, the reason, and how to move the pin. The last CI run had installed `AxeWindowsCLI-2.4.2.zip` from `releases/latest`, so the pinned binary is the one that ran on 2026-09-10.

### The count, from the tree

`ls src/presentation/wx_*.rs` is 27 modules. Not all build a window: `wx_app` is the frame, `wx_context_menu` and `wx_tray` are menus, five `wx_*_module` files are panels inside the frame. Counting `Dialog::builder(` sites with a word boundary, so `MessageDialog`, `FileDialog`, `DirDialog` and `TextEntryDialog` are excluded, gives **38 sites**. One site, `wx_managers.rs:414` `make_shell`, serves the filter, tag and signature managers, so 38 sites are **40 dialog windows**. They split three ways:

- **9 scanned before this plan**: Account Manager, Add a calendar, Search Messages, Blocked Senders, Calendar, Compose, Before you start, Settings, Filter Manager. With the main frame and the reader frame, the eleven targets.
- **14 top-level windows not scanned**, each with an entry point of its own: Columns, Which copy do you want to keep, the destination picker, Folders to keep up to date, the item form, Contact Manager, Reminder, Conversation, This event repeats, When should this message go, Add an address book, Tag Manager, Signature Manager, About Wixen Mail.
- **17 nested**, opened only from inside one of the above: the account edit dialog, Confirm Delete in the Calendar window, Check Spelling, Insert Table and Preview Before Send in the composer, the contact edit dialog and its Add Email Address, Add Phone Number, Add Address and Add Custom Field, the rule, filter, tag and signature edit dialogs, wait-for-an-answer, choose-from-list, and ask-for-a-name.

9 + 14 + 17 = 40. The task-1 summary's "at least fourteen" was 9 + 2 + 3 and double-counted the contact manager, which is both `wx_managers` in the nine and `manage_contacts` in the three; its thirteen distinct windows plus About, which is in `wx_app.rs` and no list had, is the fourteen here. Beyond the dialogs, the main frame has six module panels and `main` shows one of them.

**So thirty targets: 10 that existed, 14 top-level dialogs, and 6 module panels.** With `main`, thirty-one entries in the workflow's array. `ScanTarget::ALL` is `[ScanTarget; 30]` and the array holds those thirty names plus `'main'`; `test_the_workflow_asks_for_every_target` holds them together.

### Every one opened, and none is ledgered as unreachable

Each target was started on a throwaway profile on this machine with the debug binary, the way the workflow starts it, and after five seconds the process's visible top-level windows were listed with `EnumWindows` and `GetWindowText`. Every dialog target showed its dialog, owned, above `Wixen Mail`; every module target retitled the frame and showed no dialog.

| target | window seen | opened on |
|---|---|---|
| `columns` | Columns | the inbox's default layout |
| `which-copy` | Which copy do you want to keep? | two copies of one contact that disagree on one field, with one field only the provider names |
| `destination` | Move the message to | two accounts with an Inbox and an Archive each |
| `folder-choice` | Folders to keep up to date: work@example.com | three folders, one holding every message |
| `new-event` | Edit Event | the real path: `new_pim_item(ItemKind::Event)`, filed on this computer since a fresh profile has no account |
| `contacts`, `tags`, `signatures` | Contact Manager, Tag Manager, Signature Manager | the real managers, the same shape as `filters` |
| `reminder` | Reminder | a reminder half an hour late, said and sounded first as a real one is |
| `conversation` | Conversation | three messages, a reply under the first and a reply under that |
| `which-days` | This event repeats | an event repeating every week, kept on this computer |
| `send-later` | When should this message go? | the main frame as parent; the function became generic over `WxWidget` the way `wx_item_form::ask_for` is |
| `add-address-book` | Add an address book by its address | nothing, as `add-calendar` |
| `about` | About Wixen Mail | nothing |
| `mail-module`, `calendar-module`, `contacts-module`, `reminders-module`, `tasks-module`, `notes-module` | Wixen Mail, Calendar - Wixen Mail, Contacts - Wixen Mail, Reminders - Wixen Mail, Tasks - Wixen Mail, Notes - Wixen Mail, no dialog | the frame's own module switch, handed into `open_for_scanning` as a closure |

The eleven that existed were run the same way and each showed its window too, with one thing to say about `main`: on a fresh profile with no target it shows **Before you start**, the first-run question, over the frame, because that question is skipped only when a target is given. So `main` has always been the frame under a modal, and the bare main window had never been scanned. `mail-module` is the bare window. That is the one target beyond the count Pratik answered, ledger 393.

Fixtures live in `src/presentation/scan_fixtures.rs`, six functions, each with a test for the one property its window refuses to open without: a conversation with a reply, copies that disagree, two branches with places filed under the right account, a folder that holds every message, a late reminder, an event that repeats. In the red commit each returned the data its window refuses, an empty list or an event with no repeat, so each test failed for the reason it exists.

### The window that is not open is reported, not scanned

Nearly every target is modal, so `open_for_scanning` does not return until the window closes, and the workflow kills the process while it is up. The call returning therefore means the window is not on screen, and the process used to sit there with the main window up while the scan walked it and reported a pass for a dialog nobody looked at. Now `open_for_scanning` answers `OnReturn::WindowClosed` or `WindowStillUp`, the call site leaves with `WINDOW_NOT_OPEN`, exit code 3, on the first, and the workflow asks `$app.HasExited` before it scans and says which of two things happened: code 3, "the application said the window was not open and left, so there was nothing to scan", or any other code, "exited with code N before the scan". Proved by starting `--scan-target about`, posting `WM_CLOSE` to the About window from another process, and reading the process back: `HasExited=True ExitCode=3`.

Six of the new arms can refuse without a window: the four managers and the item form through `manager_account` and `new_item::destination`, the destination picker on empty branches, the which-days question on an event that does not repeat. Each was opened here and none refused; the exit code is what says so in CI if one ever does.

### Three defects the last CI run showed, fixed and measured

`gh run view 34467657771 --log`, the accessibility run of 2026-09-10 at `1aba3a5b`, conclusion "success" because the job is `continue-on-error`.

**1. Seven of eleven windows were "not scanned" because they were clean.** The log reads `main : no result file, so the scan itself failed (exit 0)` and the same for settings, search, calendar, first-run, add-calendar and blocked-senders, each a line after the CLI printed `0 errors were found`. `AxeWindowsCLI.exe --help` for v2.4.2 says: "By default, the test file is saved only if errors are found." The workflow's check keyed on the file's presence, so it was inverted for exactly the clean case. `--alwayssavetestfile` is now passed, and no file means what the message says. Ledger 391.

**2. The MSAA channel had never read a dialog.** The same log reads `MSAA walk: 1797 elements, 1058 of them operated, 0 without a name` for accounts, compose and filters alike. Three windows with different controls cannot have one census. `scripts/msaa-names.ps1` walked `$process.MainWindowHandle`, which .NET defines as the first visible top-level window with no owner, and a wxWidgets dialog is owned by the frame it opened from. Listed here with Settings open: `Settings` owned, `Wixen Mail` not, and `MainWindowHandle` is the second. So the channel NVDA reads, the only one `set_accessible_name` writes to, had walked the frame for every dialog target since 2026-07-31. The script now enumerates every visible top-level window the process owns and walks each, with the window's title leading every path. And a failed walk now exits 2: under the script's own `$ErrorActionPreference = 'Stop'`, `Write-Error` terminated the script before the `exit 2` after it, and PowerShell left with 1, the code for an unnamed control, measured by asking for a process that does not exist. Proved against `notepad.exe`: "Walked 1 window(s): 'Untitled - Notepad'", 299 elements, one unnamed, exit 1; and against process 1: exit 2. The workflow now records any MSAA exit other than 0 or 1 as a walk that failed, naming the code, where before only 2 was asked about and everything else read as clean. Ledger 392.

**3. Not proved against this application here.** Walking any Wixen Mail window over MSAA on this machine crashes PowerShell with `STATUS_STACK_BUFFER_OVERRUN`, exit `-1073740791`, under pwsh 7.6.6 and Windows PowerShell 5.1 alike, on the main window alone, and before the enumeration change. NVDA is running on this machine; CI has no screen reader and walked 1797 elements without crashing. Not diagnosed, and NVDA was not stopped to find out. Ledger 389 and 390. If CI meets it, the workflow now names it rather than passing.

## Both channels, per target

UI Automation: `AxeWindowsCLI --processid` scans every window the process owns, which is why the eleven dialogs were ever scanned on that channel at all; the thirty are the same shape. MSAA: every visible top-level window, after this plan, and never before it. No target has been read by either channel in CI since this plan, and the MSAA half could not be run against this application here. The gap is named, not closed: 06-08's first artifact is the first evidence for either channel on any of the thirty.

## Red and green, honestly

Red `741d2b36`, ten tests named in `Fails-until-green:` trailers, each ran and failed with nothing else failing, and the gate said so in `red` mode. Green `fd661401`.

Two red fixtures were changed at green, both re-read once the code existed, and both re-run against the old workflow afterwards to show they were still red for the old reason:

- `test_the_workflow_pins_the_scanner_to_a_release_and_its_checksum` read the whole file, and the comment explaining why `releases/latest` was wrong reddened it. It now reads lines that are not comments, and asks for `$tag = 'v` rather than a literal tag in a URL, since the URL is built from the tag. Against `9f86ba6f`'s workflow, swapped in whole with `git show` and back with `cp`, byte-identical on restore: red.
- `test_every_window_a_fresh_profile_can_reach_has_a_name` gained `mail-module`, found while running the targets rather than reading them.

The source-reading test in `wx_app.rs` reads an arm to its end rather than its first line, because rustfmt wraps a seven-argument call and `NewEvent`'s `cache` is two lines down. It names four new arms.

## Guard records: two, measured, census 759

Both records break `.github/workflows/accessibility.yml` and name a `--lib` test, on the shape of 07-07's two records for `release.yml`. Each measured by applying the break by hand and running `cargo test --lib --no-fail-fast` at eight threads on the whole library:

| record | break | red | run |
|---|---|---|---|
| a window the program can open for the scan is one the workflow asks for | `'which-days'` out of the array, variant left | `test_the_workflow_asks_for_every_target` | 7,183 passed, 1 failed, 54 s |
| the scanner's checksum is compared with a number written down and not with itself | `$expected` set to the download's own hash | `test_the_workflow_pins_the_scanner_to_a_release_and_its_checksum` | 7,183 passed, 1 failed, 53 s |

`guards/guards.toml` holds **759** records by a TOML reader, 757 before; the census at lines 79 and 80 reads 192 + 567. The count check did not fire: `wx_app.rs` holds 196 tests before and after, `scan_target.rs` is named by no earlier record, and the two new records name it at 9.

## What the gate selects for each file, on this branch

| file | `which-checks.sh` | what runs |
|---|---|---|
| `src/presentation/scan_target.rs`, `scan_fixtures.rs`, `wx_app.rs`, `wx_send_later.rs`, `mod.rs` | `affected` | `--lib presentation::<module>::` each, `presentation::` for `mod.rs`, and seven suites coupled through `guards.toml` to `wx_app.rs`: a_whole_folder_moves_both_bounds, one_sign_in_per_piece_of_work, nothing_leaves_the_outbox_unasked, the_conflict_choice_can_be_heard, nothing_sends_a_flag_change_unasked, the_list_warning_reads_the_message, columns_belong_to_the_folder_they_were_arranged_in |
| `.github/workflows/accessibility.yml` | `all` | the whole gate, task 1's rule working |
| `scripts/msaa-names.ps1` | `affected`, selecting nothing | ledger 385, unchanged by this plan |
| `guards/guards.toml` | `affected`, selecting nothing | the six record checks in `house_style` ran under `all` |

The red commit answered `affected` and ran the scoped set above in 214 seconds of a refused first attempt and then in the run that took it. The green commit answered `all`, as the plan said it would.

**The gate refused the green commit once.** `test_nothing_says_a_new_installation_changes_nothing_while_it_changes_contacts` in `house_style` read the workflow's new comment, "whatever Microsoft shipped most recently and could change with no commit here", as a claim that a new installation changes nothing, on "shipped" beside "no" and "change". Reworded to "had published most recently, and it could move without a commit here"; the reading was right to ask and wrong about this sentence, and it is the same shape as the pin test reddening on its own explanation.

## Deviations from plan

**1. [Rule 1] `--alwayssavetestfile`.** The plan named no such flag; the CI log and the CLI's help did. Ledger 391.

**2. [Rule 2] `scripts/msaa-names.ps1` walks every top-level window and exits 2 on failure.** Not in the plan's files. The channel NVDA reads had never seen a dialog, measured from three identical censuses in the CI log. Ledger 392.

**3. `mail-module`, a thirty-first entry.** Found by running `main` and seeing Before you start over it. Ledger 393.

**4. `wx_send_later::ask_when_to_send` is generic over its parent.** The composer's dialog was the only parent it took, and one scan is one window; two signatures changed, nothing else in the file.

**5. Two guard records rather than the plan's one**, because the checksum is the change guardrail 4 is about and its plausible regression, comparing the download with itself, leaves every visible token in place.

**6. The plan's fixture stubs at red returned the wrong data rather than nothing**, so each fixture test failed for its own reason and clippy saw no dead code; the module's functions are `pub`, read by `open_for_scanning`.

**7. Two whole-file swaps of the workflow**, `git show HEAD:... >` and `cp` back, to show the refined tests red against the old file. Not scripted line edits; restored byte-identical and checked with `cmp`. Every other change to a tracked file was Read then Edit or Write. Exception set for scripted rewrites: zero. Carriage returns in every touched file: 0 by `tr -cd '\r' | wc -c`.

**8. No `docs/changelog.md` entry and no version bump.** A scan target is met by nobody using the program, and so is a workflow.

Nothing in `scripts/check.sh` or `scripts/which-checks.sh` was touched by task 2, so nothing conflicted with phase 7.

## What 06-07 inherits

- The rule set is `v2.4.2`, and its own rule list is `axe-windows-rules-2.4.2.md` on the release; read that, not `main`'s table.
- Thirty-one windows in the scan: the thirty targets and `main`, which is the frame under the first-run question. Seventeen nested dialogs are outside it, listed above and in ledger 394. Say both.
- `scripts/msaa-names.ps1`'s header still says what the two channels can and cannot judge, and now says it walks every window.

## What 06-08 inherits

- The first CI run of this workflow is the first scan of any of the thirty on either channel, and the first honest run of the clean-window case on UIA. Expect more findings than five, and expect the per-window "not scanned" list to be empty for the first time or to name a real failure.
- If the MSAA walk crashes in CI as it does here, the workflow names the window and the code. Ledger 390 holds the local measurement.
- `wx_which_days` has a target. The skipped NVDA test at `nvda-tests/tests/which-days-focus-and-tick.test.js` said the skip existed because there was no target; the skip is untouched, because the test needs a real NVDA run either way and an un-skipped test that has never passed is a check nobody reads.

## Ledger

`.planning/WINDOWS.md` 388 to 394 written through `gsd-tools windows append`, both halves at 394, no backslash in any description; 384 marked fixed through `windows fixed`, both halves moved together.

| id | kind | what |
|---|---|---|
| 384 | unmet-truth, **fixed** | the scan fetched `releases/latest`; pinned |
| 388 | unrun-verify | thirty targets opened here, none scanned by CI |
| 389 | unrun-verify | the MSAA enumeration proved on notepad and on a missing process, not on this application here |
| 390 | todo | the MSAA walk crashes PowerShell on this machine with NVDA running |
| 391 | deviation | `--alwayssavetestfile` |
| 392 | deviation | `msaa-names.ps1` walks every window and exits 2 |
| 393 | deviation | `mail-module` beyond the count answered |
| 394 | todo | seventeen nested dialogs outside the scan, one entry for the layer, with why |

## Known Stubs

None. Every target is wired end to end: a variant, a name the parser accepts, an arm that opens the window, and an entry in the workflow's array, and each was seen open.

## Threat Flags

None new. T-06-21 and T-06-23 are mitigated by the pin with its hash and the record that guards it. T-06-24 is mitigated further than the register planned: the program itself leaves when the window is not open. No package was added.

## What was not done, said plainly

- No CI run. Nothing pushed. Every "works" above is a window seen open on this machine, not a scan read.
- The MSAA channel was not run against this application here, because it crashes PowerShell with NVDA running.
- Seventeen nested dialogs are outside the scan.
- The skipped NVDA test is still skipped.
- `roadmap update-plan-progress` was not run; `ROADMAP.md`, `STATE.md` and the phase README were edited by hand and the diff read.

## Self-Check: PASSED

`src/presentation/scan_fixtures.rs`, `src/presentation/scan_target.rs`, `.github/workflows/accessibility.yml`, `scripts/msaa-names.ps1`, `guards/guards.toml` and this file exist on disk; commits `05ec26a4`, `dd4934fe`, `9f86ba6f`, `741d2b36`, `fd661401`, `ba18ca51` and `ca88d833` are in `git log --all`.

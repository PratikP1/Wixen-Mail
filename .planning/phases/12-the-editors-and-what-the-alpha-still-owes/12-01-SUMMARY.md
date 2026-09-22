---
phase: 12-the-editors-and-what-the-alpha-still-owes
plan: 01
subsystem: NVDA tests, the formatted message window, guards, requirements
tags: [nvda-case, found-20, issue-80, run-35520201976, page-window, focus, activation, ledger]

requires:
  - phase: 11-reading-and-the-list
    provides: "11-02: a failing NVDA case fails the run, so this red is a red CI; 11-11.1: the page's link listener, the route by the setting, and the case that presses the key a person presses"
provides:
  - "tests/the_page_window_keeps_the_document_focused_when_it_comes_back.rs: the real page window built on scan_fixtures::page_conversation, activated, deactivated with the keyboard taken away, and activated again, with what Windows says has the keyboard read at each step and printed; a companion over a window whose control is a text control"
  - "src/presentation/wx_app.rs: show_conversation_as_page public so the reading can build the window, and nothing else"
  - "nvda-tests/helpers/launch-app.js: foregroundWindow(), focusedElement(), waitForForeground(pid, title); activateWindow kept and its doc corrected to say what its answer means"
  - "nvda-tests/tests/a-link-opens-where-the-setting-says.test.js: comeBackToThePageWindow() with Alt+Tab, the wait for the front and activateWindow as the fallback; whereTheNextKeyWillGo(); six record fields, three per return"
  - "nvda-tests/README.md: a section saying an activation call's answer is not evidence, with run 35520201976 named"
  - "guards/guards.toml: 1,030 records, census 798 + 232; one new record on wx_app.rs at 1 red, measured"
  - ".planning/REQUIREMENTS.md: FOUND-20 ticked on its two [D] lines, its evidence line corrected, its row Complete"
  - ".planning/WINDOWS.md: 567 open for the run at the next push"
affects: [12-02 onward, which merge onto a main whose next push runs the corrected case; 12-12, which reads FOUND-20's [S] line when that push has run]

actuals:
  tokens: 16130
  tasks: 3
  commits: 4

tech-stack:
  added: []
  patterns:
    - "A reading that cannot drive the system into a state sets up the state's exact precondition and calls the entry point the system calls, then says in its own header what it therefore does not read; the tree already had one such reading for a key that cannot be sent, and this is the second, for an activation that cannot be asked for"
    - "A step that hands control to another process is where a case's evidence must be strongest, and an API's success value is the weakest evidence there is: record what the platform says the state is, not what the call said it did"

key-files:
  created:
    - tests/the_page_window_keeps_the_document_focused_when_it_comes_back.rs
  modified:
    - src/presentation/wx_app.rs
    - nvda-tests/helpers/launch-app.js
    - nvda-tests/tests/a-link-opens-where-the-setting-says.test.js
    - nvda-tests/README.md
    - guards/guards.toml
    - .planning/REQUIREMENTS.md
    - .planning/WINDOWS.md

key-decisions:
  - "The reading sends WM_ACTIVATE rather than moving the foreground, because no process started from this machine's harness can: GetForegroundWindow answers 0 on that desktop for PowerShell as well as for a test. The plan's shape (a second frame raised over the page window, SetForegroundWindow, then back) was built first and gave five steps of nothing at all, which is what found the limit"
  - "The deactivation takes the keyboard away as well as sending the message, and the reading asserts that it went. Without it the keyboard stayed where it was, wx restored nothing because a descendant still held it, and the reading passed while asserting nothing; it did pass, exactly once, before that was seen"
  - "A guard record was written although no handler was added, against the plan's 'no record, no handler'. The break is in this tree's own code, a control put in front of the browser, and without a record naming wx_app.rs the gate would run this reading on every commit except the ones that could break it"
  - "show_conversation_as_page is public rather than the reading rebuilding the window's shape: a reading over a reconstruction measures the reconstruction, and the scan target's own route into it needs a whole running application"
  - "The second return is recorded and never asserted on: no key follows it, so a window that did not come back there says nothing about the product, and failing the case on it would hide the one assertion that does"
  - "No changelog entry, because nothing a person can see changed"

patterns-established:
  - "A live-window reading that passes on its first run is asked what precondition it failed to set up, before it is believed (this one restored nothing and said so nowhere)"
  - "A plan that names a mechanism for a test harness names it as a spelling and not as a capability; Guidepup presses a chord on Windows through WScript.Shell.SendKeys, which the plan's Alt+Tab decision did not check"

requirements-completed: [FOUND-20]

coverage:
  - id: D1
    description: "The page window gives the keyboard back to the browser when it is activated, and again when it comes back after losing it"
    requirement: FOUND-20
    verification:
      - kind: integration
        ref: "tests/the_page_window_keeps_the_document_focused_when_it_comes_back.rs#test_the_document_keeps_focus_when_the_page_window_comes_back"
        status: pass
      - kind: integration
        ref: "tests/the_page_window_keeps_the_document_focused_when_it_comes_back.rs#test_the_reading_sees_a_control_that_is_not_the_browser"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'the browser is the first thing in the page window that takes the keyboard' at 1 red, measured 2026-09-22 by scripts/guards.sh --remeasure, rebuild 16 s and run 4 s"
        status: pass
      - kind: command
        ref: "cargo test --test the_page_window_keeps_the_document_focused_when_it_comes_back -> 2 passed; cargo test --lib presentation::wx_app:: -> 199 passed"
        status: pass
    human_judgment: false
  - id: D2
    description: "The case comes back the way a person does, waits for Windows to say the window is in front, and writes down where its next key will go"
    requirement: FOUND-20
    verification:
      - kind: command
        ref: "node --check on the case and on launch-app.js -> ok; node nvda-tests/node_modules/jest/bin/jest.js --listTests --config nvda-tests/jest.config.js | grep -q a-link-opens-where-the-setting-says -> found; grep -c 'Alt+Tab' on the case -> 7; grep -c 'function foregroundWindow\\|function focusedElement\\|function waitForForeground' launch-app.js -> 3; grep -c 'Answers whether Windows agreed' launch-app.js -> 0; grep -c 35520201976 on the case and the README -> 1 each"
        status: pass
    human_judgment: true
    rationale: "The case is read and not run here, by nvda-tests/README.md's rule; the next push of main runs it, ledger 567"
  - id: D3
    description: "FOUND-20 ticked on what the tree holds, its evidence corrected, the ledger told"
    requirement: FOUND-20
    verification:
      - kind: command
        ref: "grep -c '^- \\[x\\] \\*\\*FOUND-20' .planning/REQUIREMENTS.md -> 1; cargo test --test the_planning_files_agree_with_themselves -> 16 passed; --test house_style -> 74; --test the_words_that_say_nothing -> 9"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the corrected case is green on the runner, which way brought the window back, and whether the keyboard was in the document when the second K went"
    requirement: FOUND-20
    verification: []
    human_judgment: true
    rationale: "The next push of main is Pratik's; these cases never run on a machine somebody is using; ledger 567"

duration: 56min
completed: 2026-09-22
status: complete
---

# Phase 12 Plan 01: The page window's keyboard measured, and the NVDA case comes back the way a person does Summary

**The product was not at fault and that is now measured rather than argued. The formatted
message window gives the keyboard back to the browser every time it is activated, including
after a deactivation that takes the keyboard away, so a screen reader's browse mode has a
document and a next-link key is a next-link key. Run 35520201976 failed because the case
trusted `AppActivate`'s answer, which means a window was found and asked for and not that it
came to the front. The case now presses Alt+Tab the way a person does, waits until Windows
names the page window as the window in front, falls back to `activateWindow` once, and writes
down the foreground and the focused element before every key. Nothing on the runner has run:
the green is the next push of `main`, which is Pratik's, and it is ledger 567.**

## Performance

- **Duration:** 56 min from the branch at 12:18:44Z on 2026-09-22 to the merge's hook
  finishing at 13:14:36Z; the reading of the phase README, the plan, `CLAUDE.md`, the
  workflow, the plan check and 11-02's and 11-11.1's summaries began about 12:09Z, and this
  summary and the planning files came after. Of that, about 3 min was the live reading taken
  six times while it was being got right (12:25:32Z to 12:34:26Z), 40 s the guard runner
  (12:36:02Z to 12:36:42Z), 12 min 28 s the three hook runs on the branch (373 s, 184 s,
  191 s), 476 s the whole gate from 12:58:14Z to 13:06:10Z, green on its first run, and 469 s
  `main`'s hook at the merge from 13:06:47Z to 13:14:36Z, also green on its first. 0 s waiting
  for the desktop: no live-window test went red on any run.
- **Started:** 2026-09-22T12:18:44Z
- **Merged:** 2026-09-22T13:14:36Z at `e29c514b`
- **Tasks:** 3
- **Files modified:** 8, one created; `Cargo.toml` and `Cargo.lock` untouched, no crate and no
  npm package added (T-12-SC)
- **Actuals:** `tokens: 16130` is `git diff c31ca43f..e29c514b | wc -c`, 64,519 characters over
  four; the estimate's `tokens` was 29,000 and `raw_tokens` 90,000, so the factor is 0.56
  against the former and 0.18 against the latter

## When a live window was on this machine's screen, and what it was

Eight runs between 12:25Z and 12:37Z, each a test process building its own windows and each
under a second of window time: six of
`cargo test --test the_page_window_keeps_the_document_focused_when_it_comes_back` and two
inside `scripts/guards.sh`. The whole gate at 12:58Z and the merge's hook at 13:06Z each ran
the live targets again as part of `--all-targets`. The release binary and the installed binary
were never started, the tester's profile was never read, NVDA was not stopped, reconfigured or
driven, and no `npm test` ran. None of these windows took the foreground, for the reason the
next section gives, so none of them took the screen from anybody.

## What the reading found, and the three classes

`tests/the_page_window_keeps_the_document_focused_when_it_comes_back.rs` builds the real page
window through `wx_app::show_conversation_as_page` on `scan_fixtures::page_conversation`, the
fixture the `page` scan target opens and the NVDA case reads, waits for WebView2 to have made
the browser, and then reads what Windows says has the keyboard at three moments. The classes,
printed by the run itself:

| Step | The keyboard | Inside the page window |
|---|---|---|
| activated | `Chrome_WidgetWin_1` | yes |
| deactivated | nothing on the run's thread | no |
| activated again | `Chrome_WidgetWin_1`, the same handle | yes |

So the chain premise 2's second cause named holds on this tree:
`wxTopLevelWindowMSW::OnActivate` calls `DoRestoreLastFocus`, `wxSetFocusToChild` calls the
WebView's `SetFocus`, and `wxWebViewEdge::OnSetFocus` calls `MoveFocus` on the controller,
which puts the keyboard inside the document rather than on the host. **The product did not
change.** The only line that moved in `src/` is `show_conversation_as_page`'s visibility, with
a doc paragraph saying why, so no changelog entry is owed and `docs/changelog.md`, which this
plan's `files_modified` lists because the rule puts it there for a user-visible change, ended
the plan untouched.

The companion over a window whose one control is a `TextCtrl` answers `Edit` at both steps,
which is what says the reading can tell a browser from anything else rather than always
finding one or never finding anything.

## What the tree contradicted

Every command in the plan's five premises was re-run against `main` at `c31ca43f` before the
branch. Premise 1 held word for word: the run's badge, its log, its stack at the case's
line 151, and the artefact's `otherWindowsAfterTheFirstEnter` with Edge in it,
`ownWindowsAfterTheFirstEnter` with the page window's title, and `pageWindowPutBackInFront`
`true`. Premise 3 held: `activateWindow` at `launch-app.js:132-139`, `Alt+Tab` in no case (0),
"What the log holds" at `README.md:106`. Premise 4's line numbers held: `show_conversation_as_page`
at `:23517`, `page.set_focus()` at `:23640` and `:23830`, no activation handler anywhere under
`src/presentation`, `fn GetFocus` at the marker target's `:100`, 20 tests in the link target
and 199 in `wx_app.rs`. The first `expect` is at the case's line 141, which is what the plan
says after the checker's warning 1, and `sed -n 139,141p` shows 139 and 140 are its comment.
Four things differed:

1. **Neither `page.set_focus()` site is "when the window opens".** The plan and FOUND-20's
   evidence line both say the two sites are "when the window opens and when the message comes
   back". `:23640` is Alt+A back from the attachments list and `:23830` is the key back from
   the security warning bar; both are ways back to the message from a control beside it, and
   nothing in this function gives the page focus when the window opens. What does is wx:
   `frame.show(true)` activates the window and `OnActivate` hands the keyboard to the first
   child that takes it. That is not a smaller version of the plan's claim, it is the whole
   reason the reading has anything to measure, and FOUND-20's evidence line is corrected with
   the correction left visible.
2. **No process started from this harness can move the foreground.** `GetForegroundWindow`
   answers 0 on this desktop, for a PowerShell one-liner as well as for a test, so
   `SetForegroundWindow` does nothing and `GetFocus` answers 0 for a thread that is never
   active. The plan's shape, a second bare frame raised over the page window and the page
   window given the foreground again, was built first and produced five steps of "no window
   was in front, nothing had the keyboard". The reading now sets up the precondition and sends
   `WM_ACTIVATE`, which is the message the system sends and after which everything is wx's
   code and this window's, the shape
   `tests/a_settings_page_reached_from_inside_a_page_gives_focus_to_its_first_control.rs`
   already uses for a key it cannot send. What is therefore not read here is whether Windows
   delivers that message, which it does for every window it activates.
3. **Guidepup presses a chord on Windows through `WScript.Shell.SendKeys`.** `parseKey` turns
   `Alt+Tab` into `%{TAB}` and `sendKeys` runs it from a VBScript host. The plan checked how a
   chord is spelled and not whether this one reaches the task switcher, which is not certain.
   The plan's own fallback to `activateWindow` and its record of which way worked are what
   carry this, and the case says so in its header; the ledger entry says what the next run
   will settle.
4. **`jest --listTests` needs the package's own config from the repository root.** The plan's
   `--rootDir nvda-tests` exits on "Could not find a config file". `--config
   nvda-tests/jest.config.js` lists the six files, the case among them. A command in a plan is
   a claim about that plan, and this one had never been run.

## Honest RED and GREEN

Three commits on branch `the-nvda-case-comes-back-the-way-a-person-does` from `main` at
`c31ca43f`, and no red commit, because nothing was red.

`7879e40d`, task 1, 373 s. **The reading was green on arrival and that is the finding**, the
way 11-11.1's probe was a measurement rather than a rule. The plan wrote the red half as
conditional for exactly this reason: "fixed only if the reading is red". So there is no
`Fails-until-green:` trailer, no stub, and no handler.

A reading that has never been red proves nothing, so it was taken red by hand before it was
believed. A `TextCtrl` added to `show_conversation_as_page`'s sizer in front of the browser
sends the keyboard to `Edit` at every step;
`test_the_document_keeps_focus_when_the_page_window_comes_back` went red naming its own cause,
"the page window's document did not take the keyboard when the window was activated", and
`test_the_reading_sees_a_control_that_is_not_the_browser` stayed green, which is the pair that
says the reading is about the browser and not about finding any control at all. The break was
undone by hand, not by `git checkout`, so the visibility change beside it was not lost. That
break is now the guard record.

**The reading passed once while asserting nothing, and that is worth leaving on the record.**
The first version sent `WM_ACTIVATE(WA_INACTIVE)` and nothing else. Windows clears a thread's
focus when the thread stops being active; a message sent by hand does not, so the WebView
still held the keyboard, `OnActivate` found a descendant already holding it and restored
nothing at all, and every step read `Chrome_WidgetWin_1` including the deactivated one. The
run said "ok" twice. What found it was reading the printed line rather than the word "ok": the
deactivated row should not have had the keyboard. The reading now clears the focus the way a
deactivating thread does and asserts that it went, in both the page window's half and the
companion's, so the coming-back step cannot pass on a keyboard that never moved. A measurement
printed on the passing path is what made that visible, which is why it is printed.

`7f5ed603`, task 2, 184 s. JavaScript, which runs nowhere local by `nvda-tests/README.md`, so
the commit names the run in its message in place of a trailer: "Red on the runner: run
35520201976, the case's second K". What could be checked here was: `node --check` on the case
and on the helper, and `jest --listTests` still finding the case.

`adeff06f`, task 3, 191 s, documents only.

Under the TDD gate's terms there is no `test(12-01)` followed by `feat(12-01)` for a behaviour
this plan added, because it added none. `test(12-01)` is task 1, which adds a reading and the
visibility it needs; `fix(12-01)` is task 2, which changes a test harness and no product code.

## Guard records

1,029 records by the TOML reader before, 1,030 after: one new, none rewritten, none retired;
the census at `guards.toml:84` from 798 + 231 to 798 + 232. Measured in the foreground by
`scripts/guards.sh --remeasure`, `WIXEN_TEST_THREADS` untouched, the counts written by the
runner.

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| the browser is the first thing in the page window that takes the keyboard (new, `suite` the new target) | `wx_app.rs` | a `TextCtrl` built and added to the sizer in front of the page | 1, "the one test named went red, and nothing else did": the reading that activates the window | rebuild 16 s, run 4 s |

The first draft of the red list was a prediction and the runner agreed with it. Counts
written: `wx_app.rs` 199, the new target 2.

**A record was written although the plan said there would be none.** The plan's reasoning was
that a record needs a break in the tree's own code and, with no handler added, there would be
none. That is wrong about this window: the break that reddens the reading is a control put in
front of the browser, which is this tree's code and is the shape of the regression somebody
adding a control to this window would really make. Without a record naming `wx_app.rs`, the
gate would reach this reading only when its own file changed, which is every commit except the
ones that could break it, and `CLAUDE.md` names that as the reason records carry a `file` at
all.

No `--remeasure` remedy was printed at any commit: `wx_app.rs` held 199 tests before and
after, and the new target is named only by the new record, which carries its count.

## What the gate selected

| File | On the branch |
|---|---|
| `tests/the_page_window_keeps_the_document_focused_when_it_comes_back.rs` | itself, and 33 more targets coupled by `guards/guards.toml` changing on the same commit |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app::`, 199 |
| `nvda-tests/**` | no scoped target, by the README's rule; read and syntax-checked |
| `guards/guards.toml`, `.planning/*.md` | the whole-tree guards on every commit; the document-reading targets on task 3's documents-only commit, 77 in `wired` among them |

`scripts/check.sh all` on the branch at `adeff06f`, its output written to a file and its exit
status read by the same shell, never piped: exit 0, 8,653 passed and none failed, 476 s from
12:58:14Z to 13:06:10Z, the release build included; its last block says five of CI's seven
jobs. Two more than the 8,651 phase 12's README records from 11-12's gate, which is this
plan's two tests. Inside the 275 s to 654 s band on `docs/development/measurements.md`.
`main`'s hook ran `all` again at the merge: 8,653 and none failed, 469 s from 13:06:47Z to
13:14:36Z. The keyring race (ledger 374) did not appear on any run.

## One thing the toolkit says on this desktop

wx logs `'SetFocus' failed with error 0x00000057` from `wxWindowMSW::SetFocus` at
`window.cpp:530` once per activation, twice per run. It is the guarded report at `:539`, which
fires when `::SetFocus` returns false, sets an error and the focus did not arrive at the window
asked for. The end state is right every time and the reading reads the end state. It is not
diagnosed and it is almost certainly this desktop, where no thread is ever active; it is
written here rather than absorbed, because a line nobody wrote down is a line the next reader
meets fresh.

## Deviations from plan

**1. [Rule 3 - Blocking] The reading sends `WM_ACTIVATE` instead of moving the foreground.**
Found during task 1, after the plan's shape was built and read five empty steps. Without it
there is no measurement at all from this machine. The file's header says what it therefore
does not read.

**2. [Rule 1 - Bug] The deactivation takes the keyboard away, and the reading asserts it
went.** Found during task 1 by reading the printed line rather than the verdict. Without it the
reading passed while asserting nothing.

**3. [Decision] A guard record although no handler was added**, above, against the plan's
sentence. Without it the reading runs on every commit except the ones that could break it.

**4. [Rule 3 - Blocking] `show_conversation_as_page` made public.** Found during task 1: the
reading cannot build the real window otherwise, and the scan target's own route needs a whole
running application. Visibility and a doc paragraph only; no behaviour changed.

**5. [Decision] The second return is recorded and not asserted on.** The plan says the case
does the return after each Enter; making the case fail on a return no key follows would hide
the assertion that matters.

**6. [Decision] FOUND-20's evidence line corrected**, above, with the correction left visible.
The plan did not ask for it; a requirement's evidence that misreads the tree is the shape this
project's own file keeps warning about.

**7. [Decision] The plan's `jest --listTests` command corrected** to pass the package's
config. As written it cannot run.

Everything else executed as written. **No scripted edit touched a tracked file: the exception
set for this plan is zero, and it stayed there.** Every tracked file was changed by Read then
Edit or Write, the deliberate break and its undoing included; `cargo fmt` ran before the Rust
commit, which is the project's formatter and not a rewrite; the only `sed`, `awk`, `grep`,
`tr` and `python` in the session read files and logs; `git checkout` was not needed on any
file. Commit messages were written to the scratchpad and passed with `-F`. Carriage returns
measured with `tr -cd '\r' | wc -c` on every changed file before each commit, the `.js` and
`.md` files included: zero on each. No em dash in any file this plan wrote, measured with
`grep -c` for the byte sequence: zero on each; none of the six words. `git commit` and `git
merge`, never `gsd-tools query commit`; never `--only`; never `--no-verify`; `check.sh` never
piped, its exit status written by the shell that ran it. No AI attribution in any commit. The
version stays `1.0.0-alpha.1` and `Cargo.toml` and `Cargo.lock` are untouched. Only the
primary worktree was written to. Nothing pushed.

## Threat register

T-12-01 mitigated: every title crossing into PowerShell goes through `JSON.stringify`, in
`activateWindow` as before and in nothing new, since `foregroundWindow` and `focusedElement`
take no argument at all and `waitForForeground` compares in JavaScript. T-12-02 mitigated: the
wait names the page window by title and pid, the fallback is `activateWindow`, the record says
which returned it, and the failure names the foreground it saw. T-12-03 does not arise: no
activation handler was added, because the reading found the product already right, so nothing
can take focus from a dialog the page window opened. T-12-SC: no crate and no npm package
added; `Cargo.toml`, `Cargo.lock` and `nvda-tests/package-lock.json` untouched. No new surface
outside the register.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash in the description;
`test_both_halves_of_the_ledger_say_the_same_thing` green after, 16 passed in that target. 566
entries before and 567 after; 531 open before and 532 after; 35 fixed, unchanged.

| id | kind | what |
|---|---|---|
| 567 | unrun-verify | the corrected link case at the next push of `main`: which way brought the window back, or neither, and whether the keyboard was in the document when the second K went; run 35520201976 named as the red |

Entry 560 is left as it is: its clause is the tester's ear and it is unchanged by this plan.

## The issue

`gh issue comment 80` after the merge, quoting `e29c514b`. Nothing closed: #80's second half is
12-02's.

## Known stubs

None. The reading builds the shipped window through the shipped function; the case's new
helpers are called by the case; `show_conversation_as_page` keeps its non-test caller,
`open_for_scanning`'s `ScanTarget::Page` arm, which the accessibility workflow and the NVDA
case both reach.

## Not done here, on purpose

The run itself. These cases never run on a machine somebody is using, and the next push of
`main` is Pratik's; that is ledger 567 and FOUND-20's one remaining `[S]` line. Whether
Alt+Tab reaches the task switcher through `SendKeys` is part of the same question, and the
record the case now writes is what answers it either way. `#80` stays open for the separate
window, which is 12-02.

## Self-Check: PASSED

`tests/the_page_window_keeps_the_document_focused_when_it_comes_back.rs` exists on disk and
holds 2 `#[test]` attributes; `grep -c 'pub fn show_conversation_as_page'
src/presentation/wx_app.rs` is 1; `grep -c 'function foregroundWindow\|function
focusedElement\|function waitForForeground' nvda-tests/helpers/launch-app.js` is 3;
`grep -c 'Answers whether Windows agreed' nvda-tests/helpers/launch-app.js` is 0;
`grep -c '35520201976'` is 1 in the case and 1 in `nvda-tests/README.md`;
`guards/guards.toml` holds 1,030 records by the TOML reader and the census line says 232;
`.planning/WINDOWS.md` holds 567 in both halves; `grep -c '^- \[x\] \*\*FOUND-20'
.planning/REQUIREMENTS.md` is 1. Commits `7879e40d`, `7f5ed603`, `adeff06f` and `e29c514b` are
in `git log --oneline` on `main`.

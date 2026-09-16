---
phase: 09-what-the-first-day-of-testing-found
plan: 06
subsystem: settings dialog, accessibility events, developer tooling, nvda-tests, guards
tags: [tab-row, win-events, msaa, uia, comctl32, focus, nvda, guards, scan-target]

requires:
  - phase: 06-how-the-application-speaks
    provides: "06-06: scripts/msaa-names.ps1, the shape of a PowerShell instrument over a running Wixen Mail started with --scan-target on a throwaway profile, and its exit-code contract; nvda-tests and .github/workflows/nvda.yml, a real NVDA driven on CI against the release binary"
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-04: a live-window integration target coupled to wx_settings.rs by a record's suite; 09-05: the rule that when the prescribed instrument cannot answer, a lower-layer one on the same running window says what it can and what it cannot"
provides:
  - "scripts/uia-events.ps1: attaches to a running Wixen Mail or starts --scan-target settings on a throwaway profile, subscribes on both accessibility channels (the managed UI Automation client for focus, selection and name; an out-of-context SetWinEventHook scoped to the process for every EVENT_OBJECT_* but location), posts Right and Left to the tab control's own window and prints one line per event with counts per key and per kind; exit 2 when nothing could be reached"
  - "The capture, before and after, on the channel NVDA reads for a native tab control: per key EVENT_OBJECT_SELECTION once and EVENT_OBJECT_FOCUS twice on the same tab from the control's own arrow handler, and once after the change"
  - "wx_settings::answer_the_arrows_on, Along and the_tab_an_arrow_reaches: the Settings notebook takes Left, Right, Up and Down itself, main keyboard or numpad, and moves the selection through wxWidgets' SetSelection, which goes through TCM_SETCURSEL and raises the focus event once; modified keys and everything else stay with the control; no wrap"
  - "tests/the_settings_tab_row_says_each_tab_once.rs: a real WM_KEYDOWN sent to the built dialog's tab row with an in-context win-event hook counting what the control raises, one selection and one focus per key, plus a companion planting two by hand and two readings of the pure function; one way past wxdragon's no-way-to-raise-an-event limit for a native control"
  - "nvda-tests/tests/settings-tabs-read-once.test.js: six Rights and one Left under a real NVDA, each reached tab heard exactly once and in order, a duplicate and a silence as different failures; waits for the next push of main"
  - "One guard record measured, coupling the new target to wx_settings.rs"
affects: [09-07 onward, which write changelog entries under Unreleased; 09-09, which measures Settings and now finds a key-down handler on the notebook; 09-10, the phase's closing read, which reads FOUND-09 clause by clause and finds its third [D] line waiting on the NVDA run; any dialog whose native tab row is spoken twice, which can take answer_the_arrows_on as written]

actuals:
  tokens: 26500
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "When the question is what a screen reader hears, the instrument goes on the channel that screen reader reads for that control: NVDA reads a native SysTabControl32 through MSAA and win events, and the managed UI Automation client saw one event where the win-event hook saw two"
    - "A native control's own key handling can be replaced from wx without touching the control: take the key in on_key_down, do the work through the wx method that goes the quieter native way, and consume the event so the control's handler never runs"
    - "A wx control in a test can be handed a real key with SendMessageW(WM_KEYDOWN) to its own window handle, carrying the bits a real key carries, and an in-context SetWinEventHook on the test thread counts what it raises before SendMessage returns"
    - "A capture taken on a locked session says so: no window can take the foreground, injected input is refused, keys are posted to the control's window, and what a foreground-only mechanism would do is judged from the in-thread events and the source rather than claimed"

key-files:
  created:
    - scripts/uia-events.ps1
    - tests/the_settings_tab_row_says_each_tab_once.rs
    - nvda-tests/tests/settings-tabs-read-once.test.js
  modified:
    - src/presentation/wx_settings.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/manual-accessibility-pass.md

key-decisions:
  - "The logger logs both channels, not the UI Automation one the plan named: run as written it showed one ElementSelected per key and nothing else, which the plan's third branch would have read as 'no code change, ask for NVDA's debug log', and NVDA does not read that channel for this control"
  - "The win-event hook reads the event's window class, window text, object id and child id and never asks the IAccessible object for anything, because walking one crashes pwsh here (ledger 390) and the child id already says which tab"
  - "Keys are posted as WM_KEYDOWN to the tab control's own window rather than injected, because the session was locked and both SendInput and SendKeys were refused; a posted key goes through the application's own message loop to the control's own handler"
  - "The fix takes the key on the notebook and moves the selection through SetSelection rather than announcing anything or moving focus: the second event is the control's own, raised from its arrow handler and not from its selection setter, so the quiet path is the one the control already has"
  - "Up and Down are taken with Left and Right, and the numpad's four with the main keyboard's, because the native control reads all eight the same way and a fix that took four of them would leave the tester's duplicate on the other four"
  - "The row does not wrap at either end, as the native control's does not; wxdragon's advance_selection wraps and was not used"
  - "The live test shows the dialog and gives the row focus before sending keys, because the control raises its focus events for the focused item and the capture was of a row that had focus"

patterns-established:
  - "Before reading a quiet instrument as a quiet producer, confirm the instrument is on the path between the producer and the consumer who reported the symptom"
  - "A fixture that stands in for input carries every bit the real input carries; when a handler under test does nothing, print what it was handed before touching the handler"

requirements-completed: []

coverage:
  - id: D1
    description: "The event stream during Right and Left on the tab row is captured on this machine by a UI Automation event logger kept in scripts/, and the capture is in the summary before anything changes"
    requirement: FOUND-09
    verification:
      - kind: other
        ref: "scripts/uia-events.ps1 run at 96298371 on 2026-09-16 against the release build on a throwaway profile, six presses, 42 events, quoted below"
        status: pass
      - kind: other
        ref: "pwsh -NoProfile -Command 'Get-Command -Syntax scripts/uia-events.ps1' prints the four parameters; grep -c AddAutomationFocusChangedEventHandler answers 1"
        status: pass
    human_judgment: false
    rationale: "The logger subscribes on UI Automation as the line asks and on win events beside it; the capture that names the second event is on the second channel, and this line is read as met because the stream was captured on this machine before anything changed, by a logger in scripts/"
  - id: D2
    description: "Whatever the capture names as the second event is stopped at its source, and a test holds the handler that stops it"
    requirement: FOUND-09
    verification:
      - kind: integration
        ref: "tests/the_settings_tab_row_says_each_tab_once.rs#test_one_arrow_on_the_settings_tab_row_raises_one_focus_event"
        status: pass
      - kind: integration
        ref: "tests/the_settings_tab_row_says_each_tab_once.rs#test_the_tab_an_arrow_reaches_worked_examples"
        status: pass
      - kind: integration
        ref: "tests/the_settings_tab_row_says_each_tab_once.rs#test_an_arrow_is_read_from_its_key_code"
        status: pass
      - kind: other
        ref: "scripts/uia-events.ps1 run again at bfc9ef4f on the rebuilt release binary: OBJECT_FOCUS x1 on every one of six presses, quoted below"
        status: pass
    human_judgment: false
  - id: D3
    description: "An nvda-tests case arrows through the Settings tabs and holds the transcript to each tab name once; it runs on CI at the next push"
    requirement: FOUND-09
    verification:
      - kind: other
        ref: "node --check nvda-tests/tests/settings-tabs-read-once.test.js; nvda.yml runs npm test over the directory and is unchanged"
        status: pass
      - kind: other
        ref: "the case under a real NVDA on the NVDA workflow"
        status: unrun
    human_judgment: false
    rationale: "Written and syntax-checked; it has not run, cannot run here (nvda-tests/README.md, 'This never runs on your own machine'), and runs at the next push of main, which is Pratik's. Ledger 492"
  - id: D4
    description: "Whether the tab is now heard once is a listening pass on the tester's machine"
    requirement: FOUND-09
    verification: []
    human_judgment: true
    rationale: "FOUND-09's last [S] line, ledger 492. Structure is held by the win-event count; whether one event is one reading is the NVDA case's on CI and the tester's ear after"

duration: 1h25m
completed: 2026-09-16
status: complete
---

# Phase 9 Plan 06: The tab row's events captured, the second focus event stopped, an NVDA case waiting Summary

**The Settings tab control raised `EVENT_OBJECT_FOCUS` twice on the reached tab for every arrow
key, from its own key handler in comctl32, one millisecond apart, and raises it once now that the
dialog takes the arrows itself and moves the selection through wxWidgets' `SetSelection`; captured
before and after with `scripts/uia-events.ps1` on the channel NVDA reads, held by a test that sends
a real key to the built dialog and counts, red first. Whether one event is one reading has been
heard by nobody: the NVDA case is written, cannot run here, and runs at the next push of `main`,
which is Pratik's; #33 is advanced and left open until that run and his ear agree.** Nothing
pushed.

## Performance

- **Duration:** about 1 h 25 min from the first read to the merge, of which about 13 minutes were
  the two whole gates and about 4 minutes the release rebuild and the two captures
- **Started:** 2026-09-16T21:35Z (first commit 21:58:22Z)
- **Merged:** 2026-09-16T22:37:11Z at `de58771a`
- **Tasks:** 3
- **Files modified:** 7 (3 created)

## The capture, before anything changed

Taken 2026-09-16 with `pwsh -NoProfile -File scripts/uia-events.ps1`, pwsh 7.6.6, against
`target/release/wixen-mail.exe` built from `main` at `96298371` (the binary on disk predated only
the docs commit), on a throwaway profile under the temp folder pinned with `WIXEN_MAIL_DATA`, on
this machine (Windows 11 Pro 26200). **Conditions that matter:** the session was locked, so the
focused element was the Lock Screen in process 16028 and nothing in the application could take
the foreground; NVDA was running and was not stopped. The `OBJECT_STATECHANGE` lines on combo
boxes are the pages repainting and are left out here; everything else is verbatim.

```
Tab row: class='_wx_SysTabCtl32' hwnd=6685312 under 'Wixen Mail'
Tabs: General | Compose | Reading | Permissions | Calendar & PIM | Feedback | Advanced
Focused element after SetFocus on the tab row: ControlType.Pane 'Lock Screen' in process 16028
Before the first key:
  122 winevent OBJECT_VALUECHANGE class='Edit' text='' hwnd=8196630 idObject=-4 idChild=0
  158 winevent OBJECT_REORDER class='#32769' text='' hwnd=65548 idObject=-4 idChild=0
--- press 1: Right at 4008 ms, posted=True
  4012 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=71240606 idObject=0 idChild=0
  4016 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=8391590 idObject=0 idChild=0
  4018 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=2
  4018 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=2
  4019 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=2
  4185 uia SelectionItem.ElementSelectedEvent TabItem name='Compose'
--- press 2: Right at 5241 ms, posted=True
  5243 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=8391590 idObject=0 idChild=0
  5245 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=45027002 idObject=0 idChild=0
  5246 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=3
  5247 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=3
  5247 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=3
  5494 uia SelectionItem.ElementSelectedEvent TabItem name='Reading'
--- press 3: Right at 6455 ms, posted=True
  6457 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=45027002 idObject=0 idChild=0
  6460 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=27398256 idObject=0 idChild=0
  6462 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=4
  6462 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=4
  6463 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=4
  6593 uia SelectionItem.ElementSelectedEvent TabItem name='Permissions'
--- press 4: Left at 7663 ms, posted=True
  7666 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=27398256 idObject=0 idChild=0
  7667 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=45027002 idObject=0 idChild=0
  7669 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=3
  7669 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=3
  7670 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=3
  7878 uia SelectionItem.ElementSelectedEvent TabItem name='Reading'
--- press 5: Left at 8882 ms, posted=True
  8884 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=45027002 idObject=0 idChild=0
  8886 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=8391590 idObject=0 idChild=0
  8888 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=2
  8888 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=2
  8889 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=2
  9057 uia SelectionItem.ElementSelectedEvent TabItem name='Compose'
--- press 6: Left at 10095 ms, posted=True
  10098 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=8391590 idObject=0 idChild=0
  10099 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=71240606 idObject=0 idChild=0
  10101 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=1
  10101 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=1
  10102 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=6685312 idObject=-4 idChild=1
  10284 uia SelectionItem.ElementSelectedEvent TabItem name='General'

Counts per key and per event kind:
  press 1 (Right): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x2, winevent OBJECT_STATECHANGE x1, uia SelectionItem.ElementSelectedEvent x1
  press 2 (Right): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x2, winevent OBJECT_STATECHANGE x5, uia SelectionItem.ElementSelectedEvent x1
  press 3 (Right): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x2, uia SelectionItem.ElementSelectedEvent x1
  press 4 (Left): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x2, uia SelectionItem.ElementSelectedEvent x1
  press 5 (Left): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x2, uia SelectionItem.ElementSelectedEvent x1
  press 6 (Left): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x2, uia SelectionItem.ElementSelectedEvent x1
Total events logged over 6 presses: 42
```

**A reading of one Right.** In order, all within seven milliseconds: the old page panel is
hidden and the new one shown (wxWidgets' `UpdateSelection`, run from its `TCN_SELCHANGE`
handler); then, from the tab control itself once that handler returns, `OBJECT_SELECTION` on
child 2, the Compose tab; then `OBJECT_FOCUS` on child 2; then `OBJECT_FOCUS` on child 2 again,
one millisecond later, same window, same child. On the UI Automation channel, one
`ElementSelected` on the Compose tab item about 170 ms later and nothing else: no focus-changed
event (the process never had the foreground) and no `Name` change anywhere. The page panel never
raised a focus event on either channel. So the second reading is the second `EVENT_OBJECT_FOCUS`,
raised by comctl32's own arrow handling on the same tab item, and "frequently" rather than always
is a screen reader that collapses two focus events on one object when they land in one flush of
its queue and speaks both when they do not.

Where the two come from was checked in two ways before anything was written. Reading the
vendored `wxWidgets-3.3.2/src/msw/notebook.cpp`: `UpdateSelection` gives the new page focus only
`if ( !HasFocus() )`, and nothing in wxWidgets or in this code raises a win event on a page
change, so both focus events are the control's. And by driving the control the other way: posting
`TCM_SETCURSEL` to the same window, in a scratch run of the same logger, raised `OBJECT_SELECTION`
once and `OBJECT_FOCUS` once per change, on all six changes. That is what named the fix.

**Two things the capture could not do, said plainly.** The tab row never had foreground focus:
`SetFocus()` on the tab element left the Lock Screen focused, `SendInput` sent one press and then
answered `ERROR_ACCESS_DENIED`, and `System.Windows.Forms.SendKeys.SendWait` threw "The operation
completed successfully", so the keys were posted as `WM_KEYDOWN` and `WM_KEYUP` to the control's
own window, which reach the control's own handler through the application's message loop. And
because of that, whether the page panel takes focus on a foreground machine is judged from the
in-thread events and the source, not from a foreground run: the after capture below caught the
dialog opening and shows the control raising `OBJECT_FOCUS` on child 0 and then child 1 at 389 ms
as the row takes focus, and `UpdateSelection`'s `HasFocus()` branch keeps it there. The NVDA case
on CI runs on an unlocked runner and is the foreground proof.

## The capture after

Taken the same way at `bfc9ef4f` after `cargo build --release`, 1 m 19 s. This run's hook was set
early enough to see the dialog being built; those lines are left out apart from the two that show
the row taking focus at open.

```
Tab row: class='_wx_SysTabCtl32' hwnd=26282788 under 'Wixen Mail'
Tabs: General | Compose | Reading | Permissions | Calendar & PIM | Feedback | Advanced
Focused element after SetFocus on the tab row: ControlType.Pane 'Lock Screen' in process 16028
Before the first key:
  ...
  389 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=0
  389 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=1
  ...
--- press 1: Right at 4639 ms, posted=True
  4642 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=1771480 idObject=0 idChild=0
  4643 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=27987582 idObject=0 idChild=0
  4658 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=2
  4658 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=2
  4822 uia SelectionItem.ElementSelectedEvent TabItem name='Compose'
--- press 2: Right at 5877 ms, posted=True
  5878 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=27987582 idObject=0 idChild=0
  5879 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=13764492 idObject=0 idChild=0
  5902 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=3
  5902 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=3
  6167 uia SelectionItem.ElementSelectedEvent TabItem name='Reading'
--- press 3: Right at 7093 ms, posted=True
  7094 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=13764492 idObject=0 idChild=0
  7096 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=27332072 idObject=0 idChild=0
  7108 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=4
  7108 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=4
  7255 uia SelectionItem.ElementSelectedEvent TabItem name='Permissions'
--- press 4: Left at 8300 ms, posted=True
  8301 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=27332072 idObject=0 idChild=0
  8303 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=13764492 idObject=0 idChild=0
  8314 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=3
  8314 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=3
  8524 uia SelectionItem.ElementSelectedEvent TabItem name='Reading'
--- press 5: Left at 9519 ms, posted=True
  9520 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=13764492 idObject=0 idChild=0
  9521 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=27987582 idObject=0 idChild=0
  9532 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=2
  9532 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=2
  9670 uia SelectionItem.ElementSelectedEvent TabItem name='Compose'
--- press 6: Left at 10733 ms, posted=True
  10734 winevent OBJECT_HIDE class='wxWindowNR' text='panel' hwnd=27987582 idObject=0 idChild=0
  10737 winevent OBJECT_SHOW class='wxWindowNR' text='panel' hwnd=1771480 idObject=0 idChild=0
  10752 winevent OBJECT_SELECTION class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=1
  10752 winevent OBJECT_FOCUS class='_wx_SysTabCtl32' text='' hwnd=26282788 idObject=-4 idChild=1
  10916 uia SelectionItem.ElementSelectedEvent TabItem name='General'

Counts per key and per event kind:
  press 1 (Right): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_STATECHANGE x1, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x1, uia SelectionItem.ElementSelectedEvent x1
  press 2 (Right): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_STATECHANGE x5, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x1, uia SelectionItem.ElementSelectedEvent x1
  press 3 (Right): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x1, uia SelectionItem.ElementSelectedEvent x1
  press 4 (Left): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x1, uia SelectionItem.ElementSelectedEvent x1
  press 5 (Left): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x1, uia SelectionItem.ElementSelectedEvent x1
  press 6 (Left): winevent OBJECT_HIDE x1, winevent OBJECT_SHOW x1, winevent OBJECT_SELECTION x1, winevent OBJECT_FOCUS x1, uia SelectionItem.ElementSelectedEvent x1
Total events logged over 6 presses: 36
```

Six fewer events than before, one focus event per press, the same hide, show, selection and
`ElementSelected` as before.

## What landed

**Task 1, the logger.** `scripts/uia-events.ps1`, on `msaa-names.ps1`'s pattern: a header saying
what it is for, what it reads, what it cannot see, how it was used and the date; the same
`Fail-Capture` shape with exit 2 for a capture that did not happen; the application started with
`--scan-target settings` on a throwaway profile the way `accessibility.yml` starts a target, or
attached to by `-ProcessId`. The handlers are C# compiled with `Add-Type`, because both a UI
Automation client and a win-event hook call back on threads of their own and a PowerShell script
block invoked on a thread PowerShell does not own stops the script with no message; they enqueue a
line each and PowerShell drains the queue after each key. The win-event hook runs on its own
thread with a message loop, because an out-of-context hook delivers through the loop of the thread
that set it. `pwsh -NoProfile -Command "Get-Command -Syntax scripts/uia-events.ps1"` prints
`[[-ProcessId] <int>] [[-Exe] <string>] [[-Presses] <int>] [[-SettleMs] <int>]`.

**The exception, asked for and not cited.** The script is written without a test. `CLAUDE.md`'s
four exceptions (configuration-only files, documentation, glue code wiring already-tested
components, styling) do not cover a script that reads a live accessibility tree, and the same
paragraph says to say so and ask. So this is asked of Pratik here, with `scripts/msaa-names.ps1`
as the precedent: it walks the same tree, has no test, and nothing in `cargo test` can raise a
live event to hold either script to. The testable half the plan names, parsing one event into its
printed fields, was not held by a Pester test: the parsing is one string concatenation in C# per
channel and a synthetic event source is not cheap, and it was not asked for. The script is reached
by no gate target (`scripts/*.ps1` map to none), and its proof is the two captures above.

**Task 2, the fix.** `wx_settings::answer_the_arrows_on(&notebook)`, called straight after the
notebook is built, binds `on_key_down` on it: a key with Control, Shift or Alt held is left alone;
a key that `Along::from_key_code` does not know is left alone; an arrow consumes the event
(`event.skip(false)`, so wxdragon's trampoline leaves it consumed and comctl32's own handler never
runs), reads the selection, asks `the_tab_an_arrow_reaches(current, count, along)` and calls
`set_selection` on the answer. `Along` is `Back` for `WXK_LEFT` 314, `WXK_UP` 315,
`WXK_NUMPAD_LEFT` 376 and `WXK_NUMPAD_UP` 377, `Forward` for `WXK_RIGHT` 316, `WXK_DOWN` 317,
`WXK_NUMPAD_RIGHT` 378 and `WXK_NUMPAD_DOWN` 379, the codes read from the vendored `wx/defs.h` by
counting the enum. `the_tab_an_arrow_reaches` answers `None` at either end. wxWidgets'
`SetSelection` sends the page-changing and page-changed events, runs `UpdateSelection`, and calls
`TabCtrl_SetCurSel`, which is the quiet path; the pages change and focus stays on the row exactly
as before. Nothing is announced and nothing moves focus.

**Task 2, the reading.** `tests/the_settings_tab_row_says_each_tab_once.rs`, `cfg(windows)`
throughout: it builds the real dialog on a frame, shows it and gives the row focus, then for
Right, Left, Right on the numpad and Left on the numpad sends `WM_KEYDOWN` and `WM_KEYUP` to the
notebook's own window handle inside `events_raised_by`, which sets an in-context
`SetWinEventHook` for `EVENT_OBJECT_FOCUS` to `EVENT_OBJECT_SELECTION` on this thread and process
(with `GetModuleHandleW(null)` as the module, which in-context hooks require and a null refused),
runs the action, unhooks and hands back the events on that window. Each key must land on the
expected tab and raise one selection and one focus on it and nothing about another tab; Left at
the first tab must stay and raise nothing. The companion raises a selection and two focus events
by hand with `NotifyWinEvent` and requires the reading to complain `EVENT_OBJECT_FOCUS x2`. Two
more tests hold the pure function and the key codes. `tests/every_event_has_a_control.rs` records
that wxdragon exposes no way to raise a widget event from outside; a message sent to the window's
own handle goes past wxdragon to the same window procedure a real key reaches, and this file's
header says so as one way round that limit for a native control.

**Task 3, the NVDA case.** `nvda-tests/tests/settings-tabs-read-once.test.js`, on the shape of
the four beside it: NVDA started first, `--scan-target settings` on a fresh profile, the dialog
waited for with the same settle. It waits to hear "General" (the row is focused at open, per the
capture), lets the dialog's own opening speech finish, takes the log length as a mark, presses
Right six times waiting to hear each reached tab before the next, waits two seconds for a late
second reading, and reads everything since the mark: each of Compose to Advanced heard exactly
once, in order, and General not again; then a fresh mark, one Left, and Feedback once. A tab
spoken more than once and a tab never heard are reported as two different complaints with the
names. "Calendar & PIM" is matched on "PIM" because NVDA may say the ampersand as "and". The
seven names are quoted in the header with the `add_page` lines they were read from (329, 341,
364, 370, 376, 381, 387 at `bfc9ef4f`). `node --check` accepts it. `nvda.yml` runs `npm test` over
the directory with one worker, so the file is picked up with no workflow change, and the workflow
is unchanged. **It has not run, and cannot run here**: `nvda-tests/README.md`, "This never runs on
your own machine", and this machine is the tester's, with NVDA up. It runs at the next push of
`main`, which is Pratik's. No guard record, because no `cargo test` reaches it, which is the same
sentence the README gives.

## Task commits

| Commit | What |
|---|---|
| `640620b0` | feat(09-06): the logger, and the capture before anything changed |
| `f7789e15` | test(09-06): the red half of task 2, three tests named, red with the capture's own counts and a stub answering None |
| `bfc9ef4f` | feat(09-06): `answer_the_arrows_on`, `Along`, `the_tab_an_arrow_reaches`, the extended bit in the fixture, one record, the changelog entry, the manual-pass line |
| `a7a54743` | test(09-06): the NVDA case, no trailer, unrun by construction |
| `de58771a` | Merge 09-06 into `main` |

Branch `each-settings-tab-said-once` from `main` at `96298371`. Not pushed; 54 commits unpushed
before the commit that lands this summary.

## Honest RED and GREEN

Task 2's red commit names three. `test_one_arrow_on_the_settings_tab_row_raises_one_focus_event`
was red against the dialog as it was for the reason the plan exists, and its message was the
capture reproduced by `cargo test`: `tab 2: EVENT_OBJECT_SELECTION x1, EVENT_OBJECT_FOCUS x2, 0
about another tab` for Right and the same on tab 1 for Left, with the companion and the
end-of-row check passing in the same run. The two readings of the pure function were red against
`Along::from_key_code` and `the_tab_an_arrow_reaches` stubbed to answer `None`, on 09-05's pattern,
so the file compiled and each failed for its own reason. The gate in `red` mode ran exactly the
three named and nothing else was red. The count check did not fire: `wx_settings.rs` gained no
test and the new file was named by no record.

The green stayed red once, and the reason is worth the paragraph. With the handler in, the same
three lines came back. A debug print showed the closure running and seeing key code 378 for
`VK_RIGHT`: `WXK_NUMPAD_RIGHT`, because the fixture sent `lParam = 1` and wxWidgets reads the
extended-key bit in `lParam` to tell a main-keyboard arrow from the numpad's. The handler was
right for the key a person presses and the test had asked about a key nobody had. The fixture now
carries `KF_EXTENDED` for a main-keyboard arrow and sends two presses without it for the numpad,
and the handler takes both sets of codes, because the native control reads all eight the same way
and a fix for four of them would have left the duplicate on the other four. Three passed, and the
gate on the green commit ran the target as changed and as coupled, with `every_event_has_a_control`,
`checkbox_labels`, `the_language_the_screen_shows_is_the_one_used` and
`the_sort_controls_sit_together` green beside it.

Task 3 is red by construction and carries no trailer, as the plan says: nothing here can run it.

## What the gate selected

| File | On the branch |
|---|---|
| `scripts/uia-events.ps1` | no scoped target; formatting, clippy, the four script suites and the whole-tree guards on `640620b0` |
| `tests/the_settings_tab_row_says_each_tab_once.rs`, `src/presentation/wx_settings.rs` | `--test the_settings_tab_row_says_each_tab_once` as the changed target, `--lib presentation::wx_settings` (matching nothing), and the coupled `every_event_has_a_control`, `checkbox_labels`, `the_language_the_screen_shows_is_the_one_used`, `the_sort_controls_sit_together` on the red; the same plus the new target as coupled on the green |
| `guards/guards.toml`, `docs/changelog.md`, `docs/manual-accessibility-pass.md` | no scoped target; rode the green commit |
| `nvda-tests/tests/settings-tabs-read-once.test.js` | no scoped target; formatting, clippy, the script suites and the whole-tree guards on `a7a54743` |

Every commit ran the whole-tree guards. `scripts/check.sh all` on the branch at `a7a54743`, run
once, output to a file and the exit status read directly, never piped: exit 0 in 398 s, 7,829
passed and none failed over 66 result lines, the release build included; three more than 09-05's
7,826, the three in the new target. The keyring race (ledger 374) did not appear. `main`'s hook
ran `all` again on the merge: 7,829 and none failed in 388 s.

`bash scripts/check.sh --suites-for guards/guards.toml src/presentation/wx_settings.rs` answers
`every_event_has_a_control`, `checkbox_labels`, `the_language_the_screen_shows_is_the_one_used`,
`the_sort_controls_sit_together` and `the_settings_tab_row_says_each_tab_once`, the new target
fifth.

## Guard records

846 records by the TOML reader before, 847 after; census 802 + 44 before, 802 + 45 after, the
line at `guards/guards.toml:84` moved in the green commit. One new, measured through
`scripts/guards.sh --remeasure`, `WIXEN_TEST_THREADS` untouched.

| Record | File and suite | Break | Red | Run |
|---|---|---|---|---|
| the Settings tab row answers its own arrow keys, so a tab is raised once | `wx_settings.rs`, `the_settings_tab_row_says_each_tab_once` | the `answer_the_arrows_on(&notebook)` call taken out | 1 | rebuild 20 s, run 2 s |

Counts written: `wx_settings.rs` 0 (11 records name it now, was 10), the new target 3. The count
check fired on no commit.

## Premises the tree contradicted

1. **The plan's instrument was on the wrong channel for the reporter's screen reader.** Premise 2
   named a UI Automation client with three subscriptions and premise 3 said the capture would
   settle it. Built and run as written, the UI Automation channel showed exactly one
   `ElementSelected` per key, no focus-changed event and no name change: the plan's third branch,
   which would have meant no code change and a request for NVDA's debug log. NVDA reads a native
   `SysTabControl32` through `IAccessible` and win events, and the second event is on that
   channel. The logger logs both, each named as what it is. Ledger 493.
2. **Neither candidate in premise 3 was the cause.** The page panel raised no focus event on
   either channel (wxWidgets' `UpdateSelection` gives the page focus only when the notebook has
   none, and the row had it), and no tab item's name changed. The second event is a second
   `EVENT_OBJECT_FOCUS` on the same tab from the control's own arrow handler, which task 2's
   behaviour list did not have a branch for; the fix follows `wx_reader.rs:329`'s shape in spirit,
   a handler on the notebook, but on `on_key_down` rather than `on_page_changed`, because by the
   time the page has changed the control has already raised both.
3. **Input injection is refused on this machine as it was.** The session was locked, and the
   observation log already records synthetic keystrokes refused here. Posting to the control's
   window is what worked, and the header says why.
4. **`tests/every_event_has_a_control.rs`'s limit has a way round for a native control.** It says
   wxdragon exposes no way to raise a widget event from outside; `SendMessageW(WM_KEYDOWN)` to the
   window's own handle does, for anything that is a real window, and the new target's header says
   so.
5. **The count of records naming `wx_settings.rs` was 10 at `96298371`, not the README's 8** at
   `524ff24f`; 09-02 and 09-04 added one each. 11 now.

Premises 1 and 4 held: seven pages, no announce on a page change, one `on_page_changed` in the
tree, `scripts/*.ps1` and `nvda-tests/` reached by no scoped target, the workflow running the
directory.

## Deviations from plan

**1. [Decision] The second channel.** Premise 1 above; the win-event hook was not in the plan and
is the half of the logger that found the cause. It reads no `IAccessible`, so ledger 390's crash
is not in its path, and it did not crash on any of the four runs here. Ledger 493.

**2. [Decision] Keys posted, not injected.** Premise 3 above. Ledger 493.

**3. [Decision] Up, Down and the numpad taken with Left and Right.** The plan's behaviour named
Left and Right; the native control reads eight keys the same way, and the test's own first green
run is how the numpad half was found. Said in the changelog.

**4. [Rule 1] The fixture's extended bit.** The first green run stayed red because the sent key
was not the key a person presses. Corrected in the green commit, with the reason in the fixture's
doc comment.

**5. [Process] Two scripted edits on a tracked file.** The two debug prints that found deviation
4 were put into `src/presentation/wx_settings.rs` through a Python one-liner, twice, because it
was quick; Python's text mode rewrote the file's 3,127 line endings to CRLF, measured with
`tr -cd '\r' | wc -c` before the green commit. Repaired with `git checkout -- src/presentation/wx_settings.rs`
and the green edits re-applied by hand through the Edit tool, then checked byte-identical to the
CRLF copy apart from line endings before the commit. **The exception set for scripted edits on a
tracked file is therefore two, not zero**, both scratch, both reverted, and the memory note that
warns of exactly this was in the context at the time. Every other edit to a tracked file went
through Read then Edit or Write; the scratch scripts that found `SendKeys` refused and
`TCM_SETCURSEL` quiet lived in the scratchpad. Commit messages were written to the scratchpad and
passed with `-F`; `cargo fmt` ran before each commit; carriage returns measured on every changed
file before each commit, zero on each at commit time; no em-dash in any file this plan wrote,
measured with `grep -c` for the byte sequence. `Cargo.toml` and `nvda-tests/package.json`
untouched; no package added. Every run of the release binary used a throwaway profile under the
temp folder pinned with `WIXEN_MAIL_DATA`, ended by the script; the tester's profile was not
touched, NVDA was not stopped, and no key was sent to any window but the tab control of the
process the script started.

**6. [Decision] No Pester test for the parsing.** Above, under the exception.

Everything else executed as written.

## Threat register

T-09-19: the capture before is quoted whole above and the capture after beside it, and the fix
is the one the `TCM_SETCURSEL` run named. T-09-20: the handler acts on an unmodified arrow only,
consumes nothing else, and moves no focus; a person tabbing into a page never reaches it because
key events go to the focused window and the pages are not the notebook; the manual pass line asks
for Tab into a page and back. T-09-21: both runs on a throwaway profile with `--scan-target
settings`; the logger prints control names, classes and window text of a fresh profile's own
dialog, and one such line (the phishing note) is in the after transcript above, which is the
program's own wording. T-09-SC: no package added; the C# is compiled from source in the script by
`Add-Type` against assemblies pwsh ships. No new surface outside the register: the logger is a
developer tool run by hand, the handler adds no path anything outside the dialog can reach, and
the test binds nothing a non-test path uses.

## Ledger

`.planning/WINDOWS.md` 492 to 494 written through `gsd-tools windows append`, both halves at 494,
no backslash in any description (the nine in the file predate this plan);
`the_planning_files_agree_with_themselves` green after, 16 passed. 491 before, 494 after; 463
open before, 466 after.

| id | kind | what |
|---|---|---|
| 492 | unrun-verify | whether each tab is heard once: the NVDA case on CI at the next push of `main`, the tester's ear after; criterion 6's transcript clause and FOUND-09's third `[D]` line open until the run |
| 493 | deviation | the second channel, the locked session, the posted keys, NVDA running, and the numpad |
| 494 | todo | `nvda-tests/README.md`'s table lists two of the five test files and its prose says two tests |

## Known stubs

None. `answer_the_arrows_on` is called once, at the notebook's construction in
`build_settings_dialog`, which every Settings open and the `settings` scan target go through; the
after capture on the release binary is the non-test path reaching it. `Along::from_key_code` and
`the_tab_an_arrow_reaches` have one caller each, the handler. The new target is reached by the
gate through the record's `suite`. The NVDA case is reached by `nvda.yml`'s `npm test` and by
nothing here, which is its design.

## Not done here, on purpose

The listening. Nobody has heard the row since the change, and the summary's own measurements are
of events, not of speech; ledger 492 is the line. The NVDA case's first run is the next push of
`main`, Pratik's. The nvda-tests README table, ledger 494, is a document three plans have walked
past and is not this plan's file. No `FOUND` requirement is ticked, on the phase's rule that the
last plan reads each clause by clause; the closing read should count FOUND-09's first two `[D]`
lines as met by the capture and the test, its third as waiting on the run, and its `[S]` line as
the tester's. Nothing pushed.

## Self-Check: PASSED

`scripts/uia-events.ps1`, `tests/the_settings_tab_row_says_each_tab_once.rs` and
`nvda-tests/tests/settings-tabs-read-once.test.js` exist; `grep -c answer_the_arrows_on
src/presentation/wx_settings.rs` is 2 (the definition and the call); `guards/guards.toml` holds
847 records by the TOML reader; `.github/workflows/nvda.yml` is unchanged. Commits `640620b0`,
`f7789e15`, `bfc9ef4f`, `a7a54743` and `de58771a` are in `git log --oneline` on `main`.

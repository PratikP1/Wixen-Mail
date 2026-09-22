# NVDA tests

These tests drive a real copy of NVDA against the real Wixen Mail binary and check what it
actually says out loud. They exist because the Rust test suite cannot answer that question.
It can read the source text of an announcement and confirm the code would call it. It cannot
confirm a screen reader hears it, and more than one Wixen Mail bug has hidden in exactly that
gap: a call that looked like accessibility and was not one.

## How this differs from the other accessibility checks

Wixen Mail already has two other layers, and this package is a third, not a replacement for
either:

- `cargo test` reads announcement text as strings and checks the code says the right sentence.
- `.github/workflows/accessibility.yml` walks the running application's UI Automation and MSAA
  trees with Axe.Windows and a custom script, and catches missing names, wrong roles, and
  similar structural defects, on every window the scan can reach.

Neither one presses a key and listens. A structural scan can report a control as fully named,
correctly typed, and reachable, and still be silent, or say something different from what it
shows, because nothing walked up to it and asked NVDA what it heard. This package does that.
It began on 2026-08-16 with two places a previous round of accessibility work found and could
not close any other way:

1. Whether NVDA announces "Signing in failed" when signing in to an account fails, in the
   Account Manager.
2. Whether the dialog that asks which days you mean, when you change or delete one day of a
   repeating event, focuses and ticks the same answer, so what NVDA announces on open names one
   answer as chosen rather than naming one and ticking another.

Four cases have been added since, and the table below is the inventory: six files under
`tests/`, five that run and one that is skipped. The skipped one is the second above, written
and correct. Read `tests/which-days-focus-and-tick.test.js` for why: reaching that dialog from
a clean, disposable profile needs a Rust change this package's own scope does not cover, and
the file explains what that change is and why it is out of scope here rather than made silently.

## This never runs on your own machine

Read this section before you do anything else with this package.

Running these tests starts a real copy of NVDA and a real copy of Wixen Mail, with a real,
visible, focus-taking window. If you are using a screen reader on this computer right now,
running these tests interrupts it: NVDA gets driven by a script instead of by you, and whatever
you were doing loses focus to a window you did not ask for.

These tests run in exactly one place: the `NVDA` GitHub Actions workflow
(`.github/workflows/nvda.yml`), on a disposable, Windows GitHub-hosted runner that nobody is
using for anything else. Trigger it from the Actions tab, or let it run on a push or pull
request the way the workflow file already describes. Do not run `npm test` here, and do not run
Jest directly against these files, on a computer you or anyone else is using.

## What is in here

| File | Purpose |
|---|---|
| `package.json` | Declares the two real dependencies: `@guidepup/guidepup` drives NVDA; `@guidepup/setup` downloads the disposable, portable copy of NVDA it drives. `jest` runs the tests. |
| `jest.config.js` | Raises Jest's test timeout. Starting the application, starting NVDA, and waiting for a debounced announcement all take longer than Jest's five-second default expects. |
| `helpers/launch-app.js` | Starts and stops the built `wixen-mail.exe`, and waits for its window the way `accessibility.yml` already does in PowerShell: poll for the main window rather than guess at a fixed delay, then settle briefly for a dialog opened on top of it. Since 2026-09-20 it also lists the titles of the windows the process owns, and of the ones it does not, and puts a window back in front by its title, for the case that watches a link leave. |
| `helpers/nvda-navigation.js` | Finds a control by tabbing until NVDA says its name, the way a screen reader user finds it, rather than by counting how many Tab presses come first. Also polls the spoken-phrase log rather than reading it once immediately after an action, since an announcement takes a moment to arrive and Guidepup's own capture debounces it. |
| `helpers/results.js` | Writes everything NVDA said during a test to `results/`, so the CI workflow can upload it as an artifact. |
| `tests/account-manager-sign-in-failure.test.js` | The first case described above: tabs to Sign In Again on a synthesised account no provider recognises, presses it, and waits to hear "Signing in failed" with its reason. Runs. |
| `tests/which-days-focus-and-tick.test.js` | The second case described above. Written against the real, current wording; skipped until a scan target for this dialog exists. |
| `tests/calendar-immediate-actions.test.js` | Presses Edit Event, Delete Event and Sync on an empty calendar and waits to hear each button's own answer, because those three once ran their answer while the window was hidden and NVDA heard nothing. Runs. |
| `tests/filter-manager-delete.test.js` | Presses Delete in the Filter Manager with nothing selected and waits to hear the same sentence Delete shows, the same class of bug as the calendar's. Runs. |
| `tests/settings-tabs-read-once.test.js` | Opens Settings on a fresh profile and presses Right six times and Left once, holding the log to each reached tab heard once (#33). Counts from a mark taken after the dialog has settled, not from an opening announcement; see below. Runs. |
| `tests/a-link-opens-where-the-setting-says.test.js` | Opens the formatted message window on the `page` scan target, moves to each of its two links with `K` and presses Enter on each, the way NVDA activates a link (#80). Holds the page window to still holding the message afterwards, by finding the second link where the message has it, and writes what NVDA said and which windows appeared beside the log. Added 2026-09-20. Runs. |

## Why only these two dependencies

`@guidepup/guidepup` is the only package that drives NVDA's own Controller Client API
correctly. Writing that ourselves means reverse-engineering NVDA's remote-control protocol,
which is exactly the kind of narrow, high-cost, easy-to-get-wrong problem a dependency exists
to solve. `@guidepup/setup` is the companion tool that downloads the disposable, portable copy
of NVDA the driver expects to find; without it there is no NVDA for the driver to start.

Both are pinned to exact versions rather than a range. The driver finds NVDA by reading a
manifest bundled inside `@guidepup/guidepup` itself and looking for exactly the version that
manifest names; letting the two packages drift to different versions independently is a way
for "NVDA got installed" and "the driver went looking for it" to disagree on what got
installed.

`jest` runs the tests. Nothing else was added. There is no TypeScript compiler, no Babel, and
no separate assertion or matcher library: `@guidepup/guidepup` ships as CommonJS with no
`"type": "module"` in its own `package.json`, so a plain `require()` and Jest's own built-in
`expect()` are enough, and adding a build step to convert syntax nothing here uses would be
weight with no job to do.

## Running this in CI

The `NVDA` workflow:

1. Builds the release binary (`cargo build --release`).
2. Installs Node and this package's dependencies (`npm ci`).
3. Downloads the disposable, portable copy of NVDA `@guidepup/guidepup` expects.
4. Runs every case in `tests/` that is not skipped: five on 2026-09-20.
5. Uploads whatever NVDA said, as a plain text artifact, whether the run passed or failed.

Since 2026-09-18 a case that fails fails the run. Until then the job carried
`continue-on-error: true`, so its findings would be a work queue rather than a gate, and the
queue went unread: run 35336142908, on `main` at `744d05ef`, reported success over a job that
failed, and the sign-in case had failed at every run since 2026-09-15 with nobody told. The
badge now says what the job said. Still read the summary and the uploaded transcript: a run
that fails to start NVDA at all and a run where NVDA started and heard the wrong sentence are
different problems, and the summary says which one happened.

## What the log holds, and what it does not

Each case ends by writing everything NVDA said during it to `results/`, read back from
Guidepup's spoken-phrase log. Run 35336142908 showed the shape of that log, and a case has to
be written to it. The transcript of every case that passed begins with what the case's first
key made NVDA say: the account row after Down, a button after Tab. None holds what NVDA said
as the dialog opened, though the dialog does speak: the tester heard Settings speak by hand on
the same build the same day. The settings case had been written to wait for that opening
announcement before pressing any key, and it timed out with an empty log. So a case waits for
what a key makes NVDA say, and never for an opening; it takes its mark after the dialog has
settled and counts from the first key.

An activation call's answer is not evidence that a window is in front. `AppActivate`, which
`activateWindow` in `helpers/launch-app.js` uses, answers that it found a window and asked for
it. Windows lets a process that is not itself in front find a window and flash its taskbar
button instead of raising it, and the answer is the same either way. Run 35520201976 on `main`
at `0ad66e48` is what that costs. The link case there pressed Enter on a link, the browser
opened the page, the case called `AppActivate` on the page window, kept its `true` in the
record and pressed its next key, and NVDA said one empty phrase. Two causes fitted and the
record could choose neither: the window never came back, so the key went to the browser; or it
came back and the keyboard was not in the document, so the key was not a browse-mode key.

So a case that hands the front to another process and takes it back reads the front rather
than trusting the call: `waitForForeground` polls `foregroundWindow` until Windows names the
window it asked for, and throws with what it saw in front instead. Before the next key it
writes down `foregroundWindow` and `focusedElement`, which say where that key went and what
had the keyboard when it got there. A failure after that names its own cause instead of
leaving it to be inferred from silence.

A clean result here means the specific keystrokes and the specific sentences these cases
check were really spoken. That is stronger than the structural scan, which never presses a key.
It is still not a full manual walkthrough, and it says nothing about any control, any dialog,
or any sentence these cases do not touch.

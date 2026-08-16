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
shows, because nothing walked up to it and asked NVDA what it heard. This package does that,
for two places a previous round of accessibility work found and could not close any other way:

1. Whether NVDA announces "Signing in failed" when signing in to an account fails, in the
   Account Manager.
2. Whether the dialog that asks which days you mean, when you change or delete one day of a
   repeating event, focuses and ticks the same answer, so what NVDA announces on open names one
   answer as chosen rather than naming one and ticking another.

The second test is written and correct, but skipped. Read `tests/which-days-focus-and-tick.test.js`
for why: reaching that dialog from a clean, disposable profile needs a Rust change this package's
own scope does not cover, and the file explains what that change is and why it is out of scope
here rather than made silently.

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
| `helpers/launch-app.js` | Starts and stops the built `wixen-mail.exe`, and waits for its window the way `accessibility.yml` already does in PowerShell: poll for the main window rather than guess at a fixed delay, then settle briefly for a dialog opened on top of it. |
| `helpers/nvda-navigation.js` | Finds a control by tabbing until NVDA says its name, the way a screen reader user finds it, rather than by counting how many Tab presses come first. Also polls the spoken-phrase log rather than reading it once immediately after an action, since an announcement takes a moment to arrive and Guidepup's own capture debounces it. |
| `helpers/results.js` | Writes everything NVDA said during a test to `results/`, so the CI workflow can upload it as an artifact. |
| `tests/account-manager-sign-in-failure.test.js` | The first test described above. Runs. |
| `tests/which-days-focus-and-tick.test.js` | The second test described above. Written against the real, current wording; skipped until a scan target for this dialog exists. |

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
4. Runs both tests.
5. Uploads whatever NVDA said, as a plain text artifact, whether the run passed or failed.

The job does not block a pull request. Read its own summary and the uploaded transcript rather
than the pass or fail badge: a run that fails to start NVDA at all and a run where NVDA started
and heard the wrong sentence are different problems, and the summary says which one happened.

A clean result here means the specific keystrokes and the specific sentences these two tests
check were really spoken. That is stronger than the structural scan, which never presses a key.
It is still not a full manual walkthrough, and it says nothing about any control, any dialog,
or any sentence these two tests do not touch.

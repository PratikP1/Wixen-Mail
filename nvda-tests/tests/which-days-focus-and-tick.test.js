// Meant to prove that what NVDA announces when the "which days do you mean"
// dialog opens, the focused control's name, role and state, names the
// preselected answer as both focused and checked, with no mismatch between
// the two: see `wx_which_days.rs`'s own file comment on why that mismatch
// was a real bug once, and what it cost somebody who heard one answer and
// got another.
//
// This test is written against the real, current wording and is otherwise
// ready to run. It is skipped because opening this dialog needs an event
// that repeats, selected, with an edit or delete already under way, and
// every one of the three places that call `which_days_are_meant`
// (`managers.rs:1621`, `wx_calendar.rs:302`, `wx_calendar.rs:378`) is
// reached only from deep inside the calendar's own edit and delete flow,
// gated on a real, selected, repeating event. None of them is reachable
// from a fresh profile the way `--scan-target accounts` reaches the Account
// Manager: `src/presentation/scan_target.rs` has no target for this dialog.
//
// Adding one is possible and has a real precedent in this codebase,
// `ScanTarget::Reader` opens on a made-up message the same way this dialog
// would need to open on a made-up repeating event, but it is not a change
// this package's own scope covers on its own, for a reason specific to this
// module: `scan_target.rs` carries a test,
// `test_the_workflow_asks_for_every_target`, that requires every scan
// target to be named in `.github/workflows/accessibility.yml`'s own target
// list. Adding a target here would mean editing that workflow too, and
// would fold this dialog, opened on synthesised data, into the structural
// scan that workflow already runs on every push and pull request, which is
// a decision belonging to whoever owns that scan's scope, not a side effect
// of building this package.
//
// Once a target exists, this test needs only its launch call filled in and
// `.skip` removed.

"use strict";

const { nvda } = require("@guidepup/guidepup");
const {
  freshProfileDir,
  launchForScanning,
  waitForWindow,
  killApp,
} = require("../helpers/launch-app");
const { writeSpokenLog } = require("../helpers/results");

const RESULT_NAME = "which-days-focus-and-tick";

// Read from src/presentation/wx_which_days.rs and
// src/application/calendar.rs rather than guessed. The dialog title is
// `Dialog::builder(parent, "This event repeats")`
// (wx_which_days.rs:62). The preselected answer is
// `EditMeans::PRESELECTED`, which is `EditMeans::WholeSeries`
// (calendar.rs:1694), spoken as `EVERY_DAY_IN_THE_SERIES` with its
// keyboard letter removed (calendar.rs:1679, :1708-1710):
const PRESELECTED_NAME = "Every day in the series";

// `set_accessible_name_and_description(&button, &means.spoken(), &will_do)`
// (wx_which_days.rs:108) puts this on the button too, read on focus by a
// screen reader working through MSAA. Computed by
// `what_it_will_do(WhatIsBeingDone::Changing, EditMeans::WholeSeries,
// &WhatTheCalendarAllows::just(WhereAChangeGoes::KeptHere))`
// (calendar.rs:2112, matched at :2146-2148), which is the call shape a
// synthesised "changing a kept-here event" scan target would use:
const PRESELECTED_DESCRIPTION =
  "Changes every day this event falls on, and leaves the day it starts on where it is. " +
  "Nothing is sent anywhere, because no account holds this event.";

let app;

beforeAll(async () => {
  const dataDir = freshProfileDir("which-days");
  // No scan target exists for this dialog yet; see the file comment above.
  // Left as the name this dialog would use if one were added, following
  // `scan_target.rs`'s existing kebab-case naming
  // (`as_name()`: "first-run", "add-calendar").
  app = launchForScanning("which-days", dataDir);
  await waitForWindow(app, { extraSettleMs: 3000 });
  await nvda.start();
});

afterAll(async () => {
  // Best-effort: this test's own assertions already say whether it passed.
  // A problem capturing the log must not hide or replace that result.
  try {
    const log = await nvda.spokenPhraseLog();
    writeSpokenLog(RESULT_NAME, log);
  } catch {
    // Nothing to do: there is no log worth having if NVDA never started.
  }
  await nvda.stop();
  killApp(app);
});

test.skip("NVDA names the preselected answer as both focused and checked on open", async () => {
  // Focus lands on the ticked radio button as soon as the dialog opens
  // (wx_which_days.rs:154-156: `ticked.set_focus()`, found by reading the
  // buttons back rather than assumed), so the first thing NVDA says without
  // any key pressed at all is that button's name, role and state.
  const onOpen = await nvda.lastSpokenPhrase();

  // The name: the preselected answer, not the group's own question, and not
  // the other answer.
  expect(onOpen).toContain(PRESELECTED_NAME);

  // The role and the state: a screen reader working through MSAA reads a
  // radio button's role as "radio button", and reads whether it is ticked
  // as "checked" or "not checked". Both are platform-supplied from the
  // native control, not this application's own words, so the exact phrase
  // is confirmed on the first real run rather than guessed here.
  //
  // "checked" alone is a weak check: the word "checked" is also a substring
  // of "not checked", so it would pass either way and only really rules out
  // NVDA saying nothing about state at all. The second assertion is the one
  // that does the real work, ruling out the specific mismatch this dialog
  // had before: the ticked answer named, but reported as not ticked.
  const state = onOpen.toLowerCase();
  expect(state).toContain("radio button");
  expect(state).toContain("checked");
  expect(state).not.toContain("not checked");

  // The description, read as part of the same focus announcement.
  expect(onOpen).toContain(PRESELECTED_DESCRIPTION);
});

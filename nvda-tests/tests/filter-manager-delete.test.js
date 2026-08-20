// Proves that a real, running copy of NVDA hears Delete's own answer when
// pressed, not just that the source code contains a `said_and_shown` call
// that would.
//
// The same class of bug `account-manager-sign-in-failure.test.js` found in
// Sign In Again: Delete used to end the Filter/Tag/Signature managers'
// shared modal session, run its answer in the gap while the window was
// hidden, and reopen it before Windows was ever handed back control, so
// NVDA heard nothing and reported the app unavailable instead. See
// `wx_managers.rs`'s `delete_selected`, shared by all three managers, and
// the commit that pulled it out of `run_manager_loop`'s own match arm.
//
// The Filter Manager opens on whatever the account being scanned already
// holds; a fresh profile has no filter rules, so this exercises
// `delete_selected`'s own "nothing selected" branch. That is not a smaller
// test of the fix: the bug was in Delete's own click handler running the
// announcement in the wrong place, and both branches of `delete_selected`
// run from the same handler this fix changed. `--scan-target filters` also
// exercises the fix landed alongside this test: `open_for_scanning` used to
// hand this window a message store of `None`, which made it refuse before
// ever opening, so this test would not have been able to reach the dialog
// at all before that was corrected.

"use strict";

const { nvda } = require("@guidepup/guidepup");
const {
  freshProfileDir,
  launchForScanning,
  waitForWindow,
  killApp,
} = require("../helpers/launch-app");
const { tabUntilHeard, waitToHearAll } = require("../helpers/nvda-navigation");
const { writeSpokenLog } = require("../helpers/results");

const RESULT_NAME = "filter-manager-delete";

let app;

beforeAll(async () => {
  await nvda.start();

  const dataDir = freshProfileDir("filters");
  app = launchForScanning("filters", dataDir);
  // The Filter Manager is a modal dialog opened on top of the main frame,
  // the same shape accessibility.yml already scans, so it gets the same
  // extra settle time on top of the main-window poll.
  await waitForWindow(app, { extraSettleMs: 3000 });
});

afterAll(async () => {
  try {
    const log = await nvda.spokenPhraseLog();
    writeSpokenLog(RESULT_NAME, log);
  } catch {
    // Nothing to do: there is no log worth having if NVDA never started.
  }
  await nvda.stop();
  killApp(app);
});

test("NVDA announces the same sentence Delete would show when nothing is selected", async () => {
  // Found by name rather than by counting Tab presses, so this does not
  // depend on guessing the dialog's exact control order.
  await tabUntilHeard(nvda, "Delete");
  await nvda.press("Enter");

  const heard = await waitToHearAll(nvda, ["Select a filter to delete"]);
  expect(heard).toContain("Select a filter to delete");
});

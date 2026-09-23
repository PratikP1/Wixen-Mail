// Proves that a real, running copy of NVDA hears what Edit Event, Delete
// Event and Sync actually say when pressed, not just that the source code
// contains a `said_and_shown` call that would.
//
// This is the same class of bug `account-manager-sign-in-failure.test.js`
// found in Sign In Again: each of these three used to end the dialog's own
// modal session, run its answer in the gap while the window was hidden, and
// reopen it before Windows was ever handed back control, so NVDA heard
// nothing and reported the app unavailable instead. See `wx_calendar.rs`'s
// `request_sync`, `edit_selected_event` and `delete_selected_event`, and the
// commit that pulled them out of `show_calendar_dialog`'s own match arms.
//
// A fresh profile's calendar holds no events, so this exercises Edit's and
// Delete's own "nothing selected" branch rather than a fixture event. That
// is not a smaller test of the fix: the bug was in the button's own click
// handler running the announcement in the wrong place, and both branches of
// `edit_selected_event`/`delete_selected_event` run from the same handler
// this fix changed. New Event is left alone, since pressing it opens a
// second, nested modal dialog this test has no need to drive.

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

const RESULT_NAME = "calendar-immediate-actions";

let app;

beforeAll(async () => {
  // NVDA starts before the application does, and stays running while it
  // opens, so nothing the dialog announces as it appears is missed.
  await nvda.start();

  const dataDir = freshProfileDir("calendar");
  app = launchForScanning("calendar", dataDir);
  // The Calendar window is a modal dialog opened on top of the main frame,
  // the same shape accessibility.yml already scans, so it gets the same
  // extra settle time on top of the main-window poll.
  await waitForWindow(app, { extraSettleMs: 3000 });
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

test("NVDA hears Edit Event's, Delete Event's and Sync's own answers when pressed", async () => {
  // Found by name rather than by counting Tab presses, the same reason
  // `account-manager-sign-in-failure.test.js` does it this way: that is how
  // somebody using NVDA would find each button too, and it does not depend
  // on guessing the dialog's exact control order. Pressed in construction
  // order (Edit Event, Delete Event, Sync all sit after New Event and
  // before Close in `build_calendar_dialog`), so each search only ever
  // moves forward through the dialog's own Tab order.
  //
  // The three sentences moved on 2026-09-23. 12-03 gave every "nothing
  // chosen" refusal one wording, built by `status_sentences::nothing_chosen`,
  // so Edit Event and Delete Event both say "Choose an event first.", and
  // Sync's step says what it is happening to, "Syncing the calendar...". NVDA
  // runs 35839692317 and 35839954840 heard the new sentences and this case
  // went on waiting for the old ones. `tests/the_nvda_cases_wait_for_words_the_program_says.rs`
  // now holds every text here to the place in `src` that says it.
  //
  // Each wait counts only what NVDA said after its own key, because Edit
  // Event and Delete Event say the same sentence and the second wait would
  // otherwise be answered by the first key's.
  await tabUntilHeard(nvda, "Edit Event");
  const beforeEdit = (await nvda.spokenPhraseLog()).length;
  await nvda.press("Enter");
  const editHeard = await waitToHearAll(nvda, ["Choose an event first."], { after: beforeEdit });
  expect(editHeard).toContain("Choose an event first.");

  await tabUntilHeard(nvda, "Delete Event");
  const beforeDelete = (await nvda.spokenPhraseLog()).length;
  await nvda.press("Enter");
  const deleteHeard = await waitToHearAll(nvda, ["Choose an event first."], {
    after: beforeDelete,
  });
  expect(deleteHeard).toContain("Choose an event first.");

  await tabUntilHeard(nvda, "Sync");
  const beforeSync = (await nvda.spokenPhraseLog()).length;
  await nvda.press("Enter");
  const syncHeard = await waitToHearAll(nvda, ["Syncing the calendar..."], { after: beforeSync });
  expect(syncHeard).toContain("Syncing the calendar...");
});

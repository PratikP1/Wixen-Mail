// Proves that a real, running copy of NVDA says each Settings tab once as
// Right and Left move along the tab row, not just that the control raises
// one focus event per key.
//
// The tester's words on 2026-09-15 (#33): "Arrowing left/right on a tab list
// in the settings dialog frequently reads the focused tab twice." Nothing in
// `src/presentation/wx_settings.rs` announces a page change, so what was
// heard came from what the native tab control raises. `scripts/uia-events.ps1`
// showed on 2026-09-16 that the control's own arrow handler raised
// EVENT_OBJECT_FOCUS twice on the reached tab per key, one millisecond
// apart, and that moving the selection through TCM_SETCURSEL raised it once;
// the dialog now answers its arrows itself and goes that way. The Rust side
// holds that with a win-event hook in
// `tests/the_settings_tab_row_says_each_tab_once.rs`, which counts one focus
// event where there were two. What it cannot say is what NVDA does with one
// event, and whether one event is one reading. This is the test that can.
//
// The seven tab names, read from `src/presentation/wx_settings.rs` rather
// than guessed, at the `notebook.add_page` calls on lines 329, 341, 364, 370,
// 376, 381 and 387 on 2026-09-16: "General", "Compose", "Reading",
// "Permissions", "Calendar && PIM" (a doubled ampersand is wxWidgets' way of
// writing one, so the tab reads "Calendar & PIM"), "Feedback" and "Advanced".
// NVDA may say the ampersand as "and", so that tab is matched on the word
// no other tab has.
//
// A fresh profile opens Settings on the General tab with the tab row itself
// focused: the win-event capture of 2026-09-16 shows the control raising
// EVENT_OBJECT_FOCUS on tab 1 as the dialog opens, before any key. So the
// first Right reaches Compose, and six of them reach Advanced.

"use strict";

const { nvda } = require("@guidepup/guidepup");
const {
  freshProfileDir,
  launchForScanning,
  waitForWindow,
  killApp,
  sleep,
} = require("../helpers/launch-app");
const { waitToHearAll } = require("../helpers/nvda-navigation");
const { writeSpokenLog } = require("../helpers/results");

const RESULT_NAME = "settings-tabs-read-once";

// Each tab, and the text a reading of it must contain. Substrings rather than
// whole phrases: NVDA says "Compose tab 2 of 7", or splits it, and the name
// is the part that a duplicate would repeat.
const TABS = [
  { name: "General", heardAs: "General" },
  { name: "Compose", heardAs: "Compose" },
  { name: "Reading", heardAs: "Reading" },
  { name: "Permissions", heardAs: "Permissions" },
  { name: "Calendar & PIM", heardAs: "PIM" },
  { name: "Feedback", heardAs: "Feedback" },
  { name: "Advanced", heardAs: "Advanced" },
];

// How long to leave after the last key for a late second reading to arrive
// before the log is read. Guidepup's own capture debounces speech for about
// a second; two seconds is that with room.
const SETTLE_AFTER_KEYS_MS = 2000;

/**
 * How many times `heardAs` occurs in everything NVDA said in `phrases`,
 * counted over the phrases joined together, so a reading split across two
 * log entries and two readings merged into one entry both count right.
 */
function timesHeard(phrases, heardAs) {
  const whole = phrases.join(" ");
  return whole.split(heardAs).length - 1;
}

/**
 * Everything NVDA said after the first `alreadySaid` entries of its log.
 *
 * The log is whole-session, and the dialog opening is itself spoken: the
 * title, the focused tab, and whatever static text NVDA reads from a new
 * dialog. Counting from a mark taken after that has settled keeps a tab
 * name spoken as part of the dialog's own text from being counted as a
 * reading of the tab.
 */
async function saidSince(alreadySaid) {
  const log = await nvda.spokenPhraseLog();
  return log.slice(alreadySaid);
}

/**
 * The tabs among `expected` that were not heard exactly once in `phrases`,
 * said as two different complaints, because a duplicate and a silence are
 * different bugs with different fixes.
 */
function whatWentWrong(phrases, expected) {
  const twice = [];
  const never = [];
  for (const tab of expected) {
    const times = timesHeard(phrases, tab.heardAs);
    if (times === 0) {
      never.push(tab.name);
    } else if (times > 1) {
      twice.push(`${tab.name} (${times} times)`);
    }
  }
  const complaints = [];
  if (twice.length > 0) {
    complaints.push(`spoken more than once: ${twice.join(", ")}`);
  }
  if (never.length > 0) {
    complaints.push(`never heard: ${never.join(", ")}`);
  }
  return complaints;
}

/**
 * Whether the first reading of each tab comes after the first reading of
 * the one before it, which is the order the keys reached them in.
 */
function inOrder(phrases, expected) {
  const whole = phrases.join(" ");
  let last = -1;
  for (const tab of expected) {
    const at = whole.indexOf(tab.heardAs);
    if (at < last) {
      return false;
    }
    last = at;
  }
  return true;
}

let app;

beforeAll(async () => {
  // NVDA starts before the application does, and stays running while it
  // opens, so the first tab's own reading, at open, is in the log and the
  // mark taken below comes after it.
  await nvda.start();

  const dataDir = freshProfileDir("settings");
  app = launchForScanning("settings", dataDir);
  // Settings is a modal dialog opened on top of the main frame, the same
  // shape accessibility.yml already scans, so it gets the same extra settle
  // time on top of the main-window poll.
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

test("NVDA says each Settings tab once as Right and Left move along the tab row", async () => {
  // The dialog opens on General with the row focused, and NVDA says so;
  // wait for that, then let it finish before taking the mark everything
  // below is counted from.
  await waitToHearAll(nvda, [TABS[0].heardAs]);
  await sleep(SETTLE_AFTER_KEYS_MS);
  const opened = (await nvda.spokenPhraseLog()).length;

  // Six Rights, Compose to Advanced, each waited for so the next key is
  // not pressed into a reading still being spoken.
  const reachedGoingRight = TABS.slice(1);
  for (const tab of reachedGoingRight) {
    await nvda.press("Right");
    await waitToHearAll(nvda, [tab.heardAs]);
  }
  await sleep(SETTLE_AFTER_KEYS_MS);

  const heardGoingRight = await saidSince(opened);
  const wrongGoingRight = whatWentWrong(heardGoingRight, reachedGoingRight);
  expect(wrongGoingRight).toEqual([]);
  expect(inOrder(heardGoingRight, reachedGoingRight)).toBe(true);
  // General was heard once at open and is not reached again going right.
  expect(timesHeard(heardGoingRight, TABS[0].heardAs)).toBe(0);

  // One Left, back from Advanced to Feedback, counted from a fresh mark.
  const beforeLeft = (await nvda.spokenPhraseLog()).length;
  await nvda.press("Left");
  await waitToHearAll(nvda, [TABS[5].heardAs]);
  await sleep(SETTLE_AFTER_KEYS_MS);

  const heardGoingLeft = await saidSince(beforeLeft);
  expect(whatWentWrong(heardGoingLeft, [TABS[5]])).toEqual([]);
});

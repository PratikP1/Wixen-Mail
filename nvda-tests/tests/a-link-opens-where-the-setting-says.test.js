// A link in the formatted message window, activated the way NVDA's Enter
// activates it, goes where the Reading tab's "Open links" setting says and
// nowhere else. On a fresh profile that is the default browser.
//
// #80, 11-11.1. The tester's words on 2026-09-18, under NVDA on
// `1.0.0-alpha.1+149.g744d05ef`: "Enter on a message and Enter on a link
// both open in the same window; the link does not go to the default
// browser." That was against a handler in `src/presentation/wx_app.rs` that
// vetoed every navigation and handed the address to the browser. The probe
// in `tests/a_link_opens_where_the_setting_says.rs` found why by reading and
// then measured it: wxdragon 0.9.17 hands a navigating event an empty
// string where the address should be, so the veto's guard against the
// control's own document load skipped every link, and the browser followed
// each one inside the window. Since 2026-09-20 the page's own script catches
// the anchor's activation before the browser navigates and posts the
// address to the window, which routes it by the setting. This is the case
// that presses the key a person presses.
//
// NVDA's Enter on a link in browse mode performs the link's default action
// through the accessibility API rather than a mouse click; Chromium delivers
// that as a click on the anchor, which is what the page's listener catches.
// So this case is the one place the route is shown to hold under the
// activation the tester used, and not only under a script's click.
//
// The `page` scan target opens the window on a made-up conversation of two
// messages: one written as a page with the link "Where it went" to
// https://example.com/where-it-went, and one written as text with
// https://example.org/written-out on a line of its own, which the renderer
// makes a link (#89). `K` is NVDA's next-link key in browse mode; it reaches
// the sender's link first and the made one second.
//
// What the case can see, and what it cannot. Under the default setting the
// page goes to the browser, which is some other process's window: so after
// Enter the case lists the windows this process does not own and records
// which appeared, and lists this process's own and requires the page
// window's title unchanged. It does not require the browser's window to
// have appeared, because which browser the runner has, what it does on its
// first start and what it titles a page are the runner's, not this
// program's; the transcript and the two window lists are written to the
// results so a person reads what happened. What it does require: the page
// window still holds the message after Enter, shown by putting it back in
// front and finding the second link with `K` where the message page has it.
// A page that had navigated inside the window has no second link of that
// name, and that is the tester's report, reproduced as a failure.
//
// **Coming back, and why this part was rewritten.** Run 35520201976 on
// `main` at `0ad66e48` failed here, on the second `K`: NVDA said "main
// landmark, Where it went, link", then "document", then one empty phrase,
// and never `example.org/written-out`. The record that run wrote settles
// the first half and not the second. It settles the route: the browser's
// window ("Example Domain - Profile 1 - Microsoft Edge") appeared among the
// windows this process does not own after the first Enter, and the page
// window kept its title and its message, so the `expect` below passed. It
// cannot settle the return, because the only evidence it kept about the
// return was `AppActivate`'s own answer, `true`, which means a window was
// found and asked for and not that it came to the front. Two causes fitted
// and nothing could choose between them: the window never came back, so
// `K` went to the browser's address bar, where a typed letter is spoken
// only if NVDA is set to speak typed characters; or it came back and the
// document did not take the keyboard, so `K` was not a next-link key.
//
// The second of those is answered:
// `tests/the_page_window_keeps_the_document_focused_when_it_comes_back.rs`
// activates the real page window on the same fixture and reads what Windows
// says has the keyboard. Activated, deactivated and activated again, the
// keyboard lands on the browser's own window inside it every time. So the
// product gives the document back and the return is this case's to get
// right.
//
// **Corrected 2026-09-23: that paragraph was true of 12-01's reading and
// false on the runner.** Runs 35839692317 (at `26beb051`) and 35839954840 (at
// `6cb8f17c`) both failed here on the second `K`, and both records say the
// page window came back, by `activateWindow`, with UI Automation's focused
// element on the window's own frame (`wxWindowNR`, named by the window's
// title) rather than in the page. Three paths fit and that one read cannot
// choose: wx gave the keyboard back to the frame because the frame had been
// saved as its own last focused child; wx gave nothing back because the
// activation carried the minimised flag; or the browser got it and let it go
// again. 12-03.1 added a handler that gives the keyboard to the page from the
// frame or from nothing, and this case now writes down what the window's own
// thread says has the keyboard from the moment it is in front, sampled over
// three seconds, and any line the handler logged when it moved it, so the
// next run says which path the runner took whether it is green or red.
//
// So the case comes back the way a person does, with Alt+Tab, and then
// waits for Windows to say the page window is in front rather than for a
// call to say it asked. Guidepup presses a chord on Windows through
// `WScript.Shell.SendKeys`, which is not certain to reach the task
// switcher, so `activateWindow` stays as a fallback and the record says
// which of the two brought the window back. Whatever brought it back, the
// foreground and the focused element are written down before any key is
// pressed, so the next run that fails here says whose failure it is instead
// of leaving it to be inferred from silence.
//
// What the runs of 2026-09-23 showed about Alt+Tab: when its ten seconds
// ran out, the window in front was an untitled one of another process, pid
// 2036, NVDA had called the Edge window "unavailable", and `activateWindow` then
// brought the page window back. The likeliest reading is a first-run window
// of Edge's disabling its main window; `foregroundWindow` now records the
// class and the process's name as well, which will confirm that or not.

"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { nvda } = require("@guidepup/guidepup");
const {
  freshProfileDir,
  launchForScanning,
  waitForWindow,
  windowTitles,
  activateWindow,
  foregroundWindow,
  focusedElement,
  waitForForeground,
  watchTheKeyboardOfTheWindow,
  killApp,
  sleep,
} = require("../helpers/launch-app");
const { waitToHearAll } = require("../helpers/nvda-navigation");
const { writeSpokenLog } = require("../helpers/results");

const RESULT_NAME = "a-link-opens-where-the-setting-says";

// The page window's title, as `show_conversation_as_page` builds it from the
// target's subject: "<subject> - headings - Wixen Mail".
const THE_PAGE_WINDOW = "Scan target - headings - Wixen Mail";

// The two links on the page, by the text NVDA reads for each: the sender's
// link by its words, the made link by the end of its address.
//
// The end and not the whole, since 2026-09-23. The made link's name is its
// address, and NVDA reads the punctuation in an address as words: the pull
// request run 35872797349, the first in which the second K reached the link,
// heard "https: slash slash example dot org slash written-out, link" while
// this case waited for "example.org/written-out" and timed out. The last
// part of the path has no punctuation NVDA speaks, so it reads the same in
// the source and in the speech.
const THE_SENDERS_LINK = "Where it went";
const THE_END_OF_THE_ADDRESS_WRITTEN_OUT = "written-out";

// How long to leave after Enter for the browser, or the page, to arrive and
// be spoken before anything is read.
const SETTLE_AFTER_ENTER_MS = 4000;

// How long to wait for Windows to say the page window is in front after
// being asked. Ten seconds is long for a window switch and short beside the
// fifteen `waitToHearAll` gives a phrase, which is the point: a case that
// waited the same either way could not say which of the two it was waiting
// for.
const WAIT_FOR_THE_FRONT_MS = 10000;

// How long to leave after the window is in front before reading what has
// the keyboard. The activation arrives first and the focus follows it.
const SETTLE_AFTER_COMING_BACK_MS = 1000;

// How long the keyboard watcher waits for the window to come to the front.
// It is started before Alt+Tab and has to outlast both ways back, Alt+Tab's
// wait and then `activateWindow`'s, so it is more than twice one of them.
const WATCH_FOR_THE_FRONT_MS = 2 * WAIT_FOR_THE_FRONT_MS + 5000;

// What `presentation::page_focus` logs when it gives the keyboard to the
// page, in `src/presentation/page_focus.rs`, after the surface's name and
// where it found the keyboard. A rewording there leaves this list empty
// rather than failing anything, which is why the record keeps every file's
// matching lines and not a yes or no.
const THE_LINE_THE_WINDOW_WRITES = "when the window came back, so it was given to the page";

let app;
let pid;
let dataDir;
// Watchers still running, stopped by `afterAll` if a return threw first.
const watchers = new Set();

/** Everything the case saw, written beside the spoken log. */
const record = {
  spokenAfterTheFirstEnter: [],
  spokenAfterTheSecondEnter: [],
  otherWindowsBefore: [],
  otherWindowsAfterTheFirstEnter: [],
  ownWindowsAfterTheFirstEnter: [],
  // Which of the two ways of coming back worked, what Windows said was in
  // front once it had, and what had the keyboard there. Written for each of
  // the two returns, before the key that follows it.
  howTheFirstReturnWorked: null,
  foregroundAfterTheFirstReturn: null,
  focusAfterTheFirstReturn: null,
  howTheSecondReturnWorked: null,
  foregroundAfterTheSecondReturn: null,
  focusAfterTheSecondReturn: null,
  // What the page window's own thread said had the keyboard from the moment
  // the window was in front, each sample with the offset it was planned for
  // and the one it measured, and how far the watcher's sighting was from
  // `waitForForeground`'s, both on the system clock.
  keyboardAfterTheFirstReturn: null,
  keyboardAfterTheSecondReturn: null,
  // Every line under the data folder's `logs` holding what the window writes
  // when it moves the keyboard: the only record of where the keyboard was
  // before the fix acted, on a run the fix makes green. Empty when it never
  // had to act.
  whatTheWindowSaidWhenItCameBack: null,
};

beforeAll(async () => {
  await nvda.start();
  dataDir = freshProfileDir("page");
  app = launchForScanning("page", dataDir);
  pid = app.pid;
  // The page window is a second frame over the main one, and the browser
  // inside it takes a moment after the frame exists: the same settle the
  // accessibility workflow gives a target that opens on top of the frame.
  await waitForWindow(app, { extraSettleMs: 3000 });
});

afterAll(async () => {
  watchers.forEach((watcher) => watcher.stop());
  if (record.whatTheWindowSaidWhenItCameBack === null) {
    record.whatTheWindowSaidWhenItCameBack = whatTheWindowSaid();
  }
  // Best-effort: the assertions above already say whether it passed, and a
  // problem writing the record must not replace that result.
  try {
    const log = await nvda.spokenPhraseLog();
    writeSpokenLog(RESULT_NAME, { record, log });
  } catch {
    // Nothing to do: there is no log worth having if NVDA never started.
  }
  await nvda.stop();
  killApp(app);
});

/** Everything NVDA said after the first `alreadySaid` entries of its log. */
async function saidSince(alreadySaid) {
  const log = await nvda.spokenPhraseLog();
  return log.slice(alreadySaid);
}

/**
 * Come back to the page window and answer how it really came back.
 *
 * Alt+Tab first, which is what a person who heard the browser open does,
 * and the page window is where Alt+Tab goes because it was in front until
 * the browser took it. Then `activateWindow`, which asks Windows directly.
 * Either way the answer is not the call's: it is `waitForForeground`, which
 * polls until Windows says the window is in front and throws with what it
 * saw instead when it never does.
 *
 * Throws only when neither worked, and then says what each one left in
 * front, because a key pressed into the wrong window is the failure this
 * whole function exists to stop.
 */
async function comeBackToThePageWindow() {
  await nvda.press("Alt+Tab");
  try {
    await waitForForeground(pid, THE_PAGE_WINDOW, { timeoutMs: WAIT_FOR_THE_FRONT_MS });
    return "Alt+Tab";
  } catch (afterAltTab) {
    const asked = await activateWindow(THE_PAGE_WINDOW);
    try {
      await waitForForeground(pid, THE_PAGE_WINDOW, { timeoutMs: WAIT_FOR_THE_FRONT_MS });
      return `activateWindow, which AppActivate answered ${asked} to; Alt+Tab did not: ${afterAltTab.message}`;
    } catch (afterActivateWindow) {
      throw new Error(
        "the page window never came back to the front, so no key after this could " +
          `reach it.\nAlt+Tab: ${afterAltTab.message}\n` +
          `activateWindow, which AppActivate answered ${asked} to: ${afterActivateWindow.message}`,
      );
    }
  }
}

/**
 * Come back to the page window with the keyboard watcher running, and
 * answer how it came back and what the window's own thread said had the
 * keyboard from the moment it was in front.
 *
 * The watcher is started and ready before Alt+Tab, so its sighting is the
 * return and not something later. Its samples are awaited before anything
 * else, so the last of them describes the return and not the key after it.
 * If neither way back worked, the watcher is stopped before the error goes
 * on.
 */
async function comeBackWatchingTheKeyboard() {
  const watcher = watchTheKeyboardOfTheWindow(pid, THE_PAGE_WINDOW, {
    waitForTheFrontMs: WATCH_FOR_THE_FRONT_MS,
  });
  watchers.add(watcher);
  await watcher.ready;
  let how;
  try {
    how = await comeBackToThePageWindow();
  } catch (why) {
    watcher.stop();
    watchers.delete(watcher);
    throw why;
  }
  const seenInFrontAt = Date.now();
  const keyboard = await watcher.samples;
  watchers.delete(watcher);
  const fromTheWatchersSighting =
    keyboard.sightedAt === null ? null : seenInFrontAt - keyboard.sightedAt;
  return { how, keyboard: { ...keyboard, waitForForegroundSawItMsAfterTheWatcher: fromTheWatchersSighting } };
}

/**
 * Every line in the data folder's logs that the window writes when it gives
 * the keyboard to the page, or an empty list.
 */
function whatTheWindowSaid() {
  if (!dataDir) {
    return [];
  }
  const logs = path.join(dataDir, "logs");
  if (!fs.existsSync(logs)) {
    return [];
  }
  return fs
    .readdirSync(logs)
    .flatMap((name) => fs.readFileSync(path.join(logs, name), "utf8").split(/\r?\n/))
    .filter((line) => line.includes(THE_LINE_THE_WINDOW_WRITES));
}

/**
 * What Windows says is in front and what has the keyboard, once the window
 * is back and the focus has followed the activation.
 */
async function whereTheNextKeyWillGo() {
  await sleep(SETTLE_AFTER_COMING_BACK_MS);
  return {
    foreground: await foregroundWindow(),
    focus: await focusedElement(),
  };
}

test("Enter on a link in the formatted message window leaves the message where it is and goes where the setting says", async () => {
  // No wait for the window's opening speech: the harness captures none
  // (nvda-tests/README.md, "What the log holds"). The page window is the
  // frame in front once it opens, and browse mode is in its document.
  record.otherWindowsBefore = await windowTitles(pid, { owned: false });
  const beforeTheFirstKey = (await nvda.spokenPhraseLog()).length;

  // K, the next link: the sender's.
  await nvda.press("k");
  await waitToHearAll(nvda, [THE_SENDERS_LINK]);

  // Enter, the way a person follows it. Under the default setting the page
  // goes to the browser and the message stays.
  await nvda.press("Enter");
  await sleep(SETTLE_AFTER_ENTER_MS);
  record.spokenAfterTheFirstEnter = await saidSince(beforeTheFirstKey);
  record.otherWindowsAfterTheFirstEnter = await windowTitles(pid, { owned: false });
  record.ownWindowsAfterTheFirstEnter = await windowTitles(pid, { owned: true });

  // The page window is still there under its own title. A page loaded in
  // its place under the message-view route would have retitled it, and the
  // default route touches no title.
  expect(record.ownWindowsAfterTheFirstEnter).toContain(THE_PAGE_WINDOW);

  // Back in front, because the browser took the front, and not one key
  // until Windows says so and the record says where that key will go. Then
  // the second link is where the message page has it. This is the assertion
  // the tester's report fails: a window whose document had become the
  // linked page has no link by this name.
  const theFirstReturn = await comeBackWatchingTheKeyboard();
  record.howTheFirstReturnWorked = theFirstReturn.how;
  record.keyboardAfterTheFirstReturn = theFirstReturn.keyboard;
  const afterTheFirstReturn = await whereTheNextKeyWillGo();
  record.foregroundAfterTheFirstReturn = afterTheFirstReturn.foreground;
  record.focusAfterTheFirstReturn = afterTheFirstReturn.focus;
  const beforeTheSecondKey = (await nvda.spokenPhraseLog()).length;
  await nvda.press("k");
  await waitToHearAll(nvda, [THE_END_OF_THE_ADDRESS_WRITTEN_OUT]);

  // Enter on the made link, the same way, so #89's link is shown to take
  // the same route as the sender's.
  await nvda.press("Enter");
  await sleep(SETTLE_AFTER_ENTER_MS);
  record.spokenAfterTheSecondEnter = await saidSince(beforeTheSecondKey);
  expect(await windowTitles(pid, { owned: true })).toContain(THE_PAGE_WINDOW);

  // The second return is written down and never asserted on. No key follows
  // it, so a window that did not come back here says nothing about the
  // product; what it does say is how well the way back works on this
  // runner, which is worth having the next time the first return fails.
  try {
    const theSecondReturn = await comeBackWatchingTheKeyboard();
    record.howTheSecondReturnWorked = theSecondReturn.how;
    record.keyboardAfterTheSecondReturn = theSecondReturn.keyboard;
    const afterTheSecondReturn = await whereTheNextKeyWillGo();
    record.foregroundAfterTheSecondReturn = afterTheSecondReturn.foreground;
    record.focusAfterTheSecondReturn = afterTheSecondReturn.focus;
  } catch (why) {
    record.howTheSecondReturnWorked = `neither way worked: ${why.message}`;
  }
  record.whatTheWindowSaidWhenItCameBack = whatTheWindowSaid();
});

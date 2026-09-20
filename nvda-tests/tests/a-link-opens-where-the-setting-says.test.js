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

"use strict";

const { nvda } = require("@guidepup/guidepup");
const {
  freshProfileDir,
  launchForScanning,
  waitForWindow,
  windowTitles,
  activateWindow,
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
// link by its words, the made link by its address.
const THE_SENDERS_LINK = "Where it went";
const THE_ADDRESS_WRITTEN_OUT = "example.org/written-out";

// How long to leave after Enter for the browser, or the page, to arrive and
// be spoken before anything is read.
const SETTLE_AFTER_ENTER_MS = 4000;

let app;
let pid;

/** Everything the case saw, written beside the spoken log. */
const record = {
  spokenAfterTheFirstEnter: [],
  spokenAfterTheSecondEnter: [],
  otherWindowsBefore: [],
  otherWindowsAfterTheFirstEnter: [],
  ownWindowsAfterTheFirstEnter: [],
  pageWindowPutBackInFront: null,
};

beforeAll(async () => {
  await nvda.start();
  const dataDir = freshProfileDir("page");
  app = launchForScanning("page", dataDir);
  pid = app.pid;
  // The page window is a second frame over the main one, and the browser
  // inside it takes a moment after the frame exists: the same settle the
  // accessibility workflow gives a target that opens on top of the frame.
  await waitForWindow(app, { extraSettleMs: 3000 });
});

afterAll(async () => {
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

  // Back in front, in case the browser took the front, and the second link
  // is where the message page has it. This is the assertion the tester's
  // report fails: a window whose document had become the linked page has no
  // link by this name.
  record.pageWindowPutBackInFront = await activateWindow(THE_PAGE_WINDOW);
  await sleep(1000);
  const beforeTheSecondKey = (await nvda.spokenPhraseLog()).length;
  await nvda.press("k");
  await waitToHearAll(nvda, [THE_ADDRESS_WRITTEN_OUT]);

  // Enter on the made link, the same way, so #89's link is shown to take
  // the same route as the sender's.
  await nvda.press("Enter");
  await sleep(SETTLE_AFTER_ENTER_MS);
  record.spokenAfterTheSecondEnter = await saidSince(beforeTheSecondKey);
  expect(await windowTitles(pid, { owned: true })).toContain(THE_PAGE_WINDOW);
});

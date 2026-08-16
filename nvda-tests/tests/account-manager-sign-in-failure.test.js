// Proves that a real, running copy of NVDA hears the Account Manager say
// "Signing in failed" when signing in fails, not just that the source code
// contains a `format!` call that would.
//
// Wixen Mail's own Rust test suite already reads this sentence as text (see
// `src/presentation/status_line.rs`'s own comment on the point: reading it
// as text "cannot tell whether the announcement reaches a screen reader").
// This is the test that can.
//
// The account this drives exists only for this scan: a fresh profile has no
// accounts of its own, so `--scan-target accounts` opens the dialog on one
// synthesised account, turned on for OAuth and addressed at a domain
// nothing in `service::oauth` recognises. That is what makes "Sign In
// Again" fail on the spot, with no network and no real credentials: see
// `scan_only_account` beside `open_for_scanning` in
// `src/presentation/wx_app.rs`.

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

const RESULT_NAME = "account-manager-sign-in-failure";

// The exact sentence, read from src/presentation/wx_account_manager.rs
// rather than guessed:
//   said_and_shown(&status, a11y, &format!("Signing in failed: {msg}"), ...)
// where `msg` is the reason `run_oauth_flow` gives up. The fixture account's
// address is not one `OAuthService::detect_provider` recognises, so `msg` is
// that function's first, network-free refusal.
const SIGN_IN_FAILED = "Signing in failed";
const WHY_IT_FAILED = "not one Wixen Mail can sign in to through a browser";

// The fixture account's own name, set in `scan_only_account`. Hearing it
// confirms the row NVDA is looking at really is the synthesised one, not an
// empty list somehow read as something else.
const FIXTURE_ACCOUNT_NAME = "Scan target";

let app;

beforeAll(async () => {
  // NVDA starts before the application does, and stays running while it
  // opens, so nothing the dialog announces as it appears is missed. This
  // test's own assertions do not depend on that first announcement, since
  // pressing Down makes a fresh one either way, but the other test in this
  // package does, and starting NVDA first is correct for both.
  await nvda.start();

  const dataDir = freshProfileDir("accounts");
  app = launchForScanning("accounts", dataDir);
  // Accounts is a modal dialog opened on top of the main frame, the same
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

test("NVDA announces Signing in failed when Sign In Again cannot reach a provider", async () => {
  // The list is the first control the dialog creates, and nothing in
  // `show_account_manager_dialog` moves focus off it, so it is what wxWidgets'
  // own default tab order gives focus to when the dialog opens. Pressing
  // Down is the first real interaction either way: on a fresh, single-
  // selection list with focus and nothing chosen yet, it selects the first
  // row, which both moves onto the fixture account and gives NVDA a state
  // change worth announcing.
  await nvda.press("Down");
  const onTheRow = await nvda.lastSpokenPhrase();
  expect(onTheRow).toContain(FIXTURE_ACCOUNT_NAME);

  // Found by name rather than by counting Tab presses: that is how somebody
  // using NVDA would find it too, and it does not depend on guessing the
  // dialog's exact control order.
  await tabUntilHeard(nvda, "Sign In Again");
  await nvda.press("Enter");

  const heard = await waitToHearAll(nvda, [SIGN_IN_FAILED, WHY_IT_FAILED]);
  expect(heard).toContain(SIGN_IN_FAILED);
  expect(heard).toContain(WHY_IT_FAILED);
});

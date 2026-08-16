// A screen reader user finds a control by listening for its name while
// tabbing, not by counting how many controls come before it. Both tests in
// this package find their way around a dialog the same way, so the pattern
// lives here once instead of twice.

"use strict";

/**
 * Press Tab (or Shift+Tab) until NVDA announces something containing
 * `substring`, and return everything it said on the press that found it.
 *
 * Throws after `maxAttempts` presses rather than looping forever, because a
 * dialog whose layout has changed should fail the test with a clear reason,
 * not hang until Jest's own timeout gives a far vaguer one.
 */
async function tabUntilHeard(nvda, substring, { maxAttempts = 15, backwards = false } = {}) {
  const key = backwards ? "Shift+Tab" : "Tab";
  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    await nvda.press(key);
    const heard = await nvda.lastSpokenPhrase();
    if (heard.includes(substring)) {
      return heard;
    }
  }
  throw new Error(
    `never heard anything containing ${JSON.stringify(substring)} in ${maxAttempts} presses of ${key}`,
  );
}

/**
 * Poll `nvda.spokenPhraseLog()` until every string in `substrings` has been
 * heard somewhere in it, or give up after `timeoutMs`.
 *
 * Checked against the whole log joined together, not one entry at a time.
 * One sentence handed to a single announcement call is not guaranteed to
 * arrive as a single entry in the log: Guidepup's own capture debounces
 * speech for a second before treating it as one phrase, and a long enough
 * sentence can still land split across two. Requiring one entry to contain
 * everything would make the test sensitive to that split rather than to
 * what was actually said.
 *
 * An announcement made through `UiaRaiseNotificationEvent` also reaches NVDA
 * a moment after the call that raises it returns, which is why this polls
 * instead of reading the log once right after the action that should have
 * caused it.
 */
async function waitToHearAll(nvda, substrings, { timeoutMs = 15000, intervalMs = 500 } = {}) {
  const deadline = Date.now() + timeoutMs;
  let log = [];
  while (Date.now() < deadline) {
    log = await nvda.spokenPhraseLog();
    const whole = log.join(" ");
    if (substrings.every((s) => whole.includes(s))) {
      return whole;
    }
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
  throw new Error(
    `never heard all of ${JSON.stringify(substrings)} within ${timeoutMs}ms. ` +
      `Everything NVDA said: ${JSON.stringify(log)}`,
  );
}

module.exports = { tabUntilHeard, waitToHearAll };

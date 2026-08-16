// Where captured NVDA speech goes, so the CI workflow can upload it as an
// artifact and a person can read what NVDA actually said without re-running
// anything.

"use strict";

const fs = require("node:fs");
const path = require("node:path");

const RESULTS_DIR = path.join(__dirname, "..", "results");

/**
 * Write everything NVDA said during a test to its own file, as plain JSON.
 *
 * Best-effort by design: called from `afterAll`, after a test has already
 * passed or failed on its own assertions, so a problem writing this file
 * must never become the error that gets reported. The caller catches
 * whatever this throws.
 */
function writeSpokenLog(name, log) {
  fs.mkdirSync(RESULTS_DIR, { recursive: true });
  const file = path.join(RESULTS_DIR, `${name}.json`);
  fs.writeFileSync(file, JSON.stringify(log, null, 2));
  return file;
}

module.exports = { RESULTS_DIR, writeSpokenLog };

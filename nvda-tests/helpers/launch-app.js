// Starting and stopping the real Wixen Mail binary, the way
// .github/workflows/accessibility.yml does it in PowerShell, written instead
// for a Jest test that needs the process to still be running while NVDA
// reads whatever it opened.
//
// Nothing in this file is safe to run on a developer's own machine: it opens
// a real, visible, focus-taking window. See the repository root README for
// why this package exists only for a disposable CI runner.

"use strict";

const { spawn, execFile } = require("node:child_process");
const { promisify } = require("node:util");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const execFileAsync = promisify(execFile);

const REPO_ROOT = path.resolve(__dirname, "..", "..");
const APP_PATH = path.join(REPO_ROOT, "target", "release", "wixen-mail.exe");

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * A fresh, empty folder for WIXEN_MAIL_DATA, so a launch never touches a
 * real profile and always starts from first-run state. Named after the
 * window it is for, so two tests running in the same job never share one.
 */
function freshProfileDir(name) {
  const base = process.env.RUNNER_TEMP || os.tmpdir();
  const dir = path.join(base, `wixen-mail-nvda-${name}-profile`);
  fs.rmSync(dir, { recursive: true, force: true });
  fs.mkdirSync(dir, { recursive: true });
  return dir;
}

/**
 * Start the built release binary on one scan target, with its data folder
 * set to somewhere disposable. Returns the child process; the caller owns
 * waiting for the window and killing it afterwards.
 *
 * `--scan-target` is the flag `src/presentation/scan_target.rs` reads.
 * Something other than a name it recognises is refused rather than quietly
 * starting the ordinary main window, which is the whole reason that flag
 * exists: this launcher inherits that safety rather than duplicating it.
 */
function launchForScanning(scanTarget, dataDir) {
  if (!fs.existsSync(APP_PATH)) {
    throw new Error(
      `${APP_PATH} does not exist. Build the release binary first ` +
        "(cargo build --release), which is a step this package's CI " +
        "workflow runs before these tests.",
    );
  }
  return spawn(APP_PATH, ["--scan-target", scanTarget], {
    env: { ...process.env, WIXEN_MAIL_DATA: dataDir },
    stdio: "ignore",
    windowsHide: false,
  });
}

/**
 * The window handle a running process owns, or 0 before it has one.
 *
 * Node has no direct way to ask Windows this; PowerShell does, and
 * accessibility.yml already trusts `Get-Process ... .MainWindowHandle` for
 * exactly this question, so this asks it the same way rather than adding a
 * native dependency for one property read a handful of times per test run.
 */
async function mainWindowHandle(pid) {
  const { stdout } = await execFileAsync("powershell", [
    "-NoProfile",
    "-Command",
    `(Get-Process -Id ${pid} -ErrorAction SilentlyContinue).MainWindowHandle`,
  ]);
  return Number.parseInt(stdout.trim(), 10) || 0;
}

/**
 * Wait for the process to publish a main window, the way
 * accessibility.yml polls rather than guessing at a fixed sleep: wxWidgets
 * needs a moment to realise the frame, and that moment is not the same on
 * every runner.
 *
 * `extraSettleMs` is the flat delay accessibility.yml adds on top, only for
 * a target that opens a dialog on top of the main frame: the dialog appears
 * after the frame does, so the window handle existing is not yet the dialog
 * existing.
 */
async function waitForWindow(child, { timeoutMs = 60000, extraSettleMs = 0 } = {}) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(
        `wixen-mail.exe exited (code ${child.exitCode}) before showing a window`,
      );
    }
    const handle = await mainWindowHandle(child.pid);
    if (handle !== 0) {
      if (extraSettleMs > 0) {
        await sleep(extraSettleMs);
      }
      return;
    }
    await sleep(500);
  }
  throw new Error(`no window appeared within ${timeoutMs}ms`);
}

/**
 * Stop the process, the way accessibility.yml's `finally` block does:
 * unconditionally, and without caring whether it had already exited.
 */
function killApp(child) {
  if (child && child.exitCode === null && !child.killed) {
    child.kill();
  }
}

module.exports = {
  REPO_ROOT,
  APP_PATH,
  freshProfileDir,
  launchForScanning,
  waitForWindow,
  killApp,
  sleep,
};

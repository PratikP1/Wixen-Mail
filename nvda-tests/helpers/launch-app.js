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
 * The titles of every visible top-level window, either the ones the process
 * owns (`owned: true`) or everybody else's (`owned: false`), in the order
 * Windows enumerates them.
 *
 * `MainWindowTitle` names one window, and a message opened into the page
 * window is a second frame of the same process, so a case that wants to
 * know what that window is called after a key has to list them all; and a
 * link that went to the default browser is a window some other process
 * owns, which is the one way to see from outside that it left. Asked of
 * Windows through PowerShell for the same reason `mainWindowHandle` is: a
 * few reads per run are not worth a native dependency.
 */
async function windowTitles(pid, { owned = true } = {}) {
  const test = owned ? `$owner -eq ${pid}` : `$owner -ne ${pid}`;
  const { stdout } = await execFileAsync("powershell", [
    "-NoProfile",
    "-Command",
    [
      "Add-Type -Namespace WixenNvda -Name Windows -MemberDefinition @'",
      "[DllImport(\"user32.dll\")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lp);",
      "[DllImport(\"user32.dll\")] public static extern int GetWindowText(IntPtr h, System.Text.StringBuilder s, int n);",
      "[DllImport(\"user32.dll\")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);",
      "[DllImport(\"user32.dll\")] public static extern bool IsWindowVisible(IntPtr h);",
      "public delegate bool EnumWindowsProc(IntPtr h, IntPtr lp);",
      "'@",
      "$titles = New-Object System.Collections.Generic.List[string]",
      "[WixenNvda.Windows]::EnumWindows({ param($h, $lp)",
      "  $owner = 0; [void][WixenNvda.Windows]::GetWindowThreadProcessId($h, [ref]$owner)",
      `  if (${test} -and [WixenNvda.Windows]::IsWindowVisible($h)) {`,
      "    $sb = New-Object System.Text.StringBuilder 512",
      "    [void][WixenNvda.Windows]::GetWindowText($h, $sb, 512)",
      "    if ($sb.Length -gt 0) { $titles.Add($sb.ToString()) }",
      "  }",
      "  $true }, [IntPtr]::Zero) | Out-Null",
      "$titles -join \"`n\"",
    ].join("\n"),
  ]);
  return stdout
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

/**
 * Ask Windows to bring the window whose title starts with `title` to the
 * front, so keys sent next reach it. A link that went to the default browser
 * leaves the browser in front; a case that then wants to read the page
 * window has to put it back.
 *
 * Answers whether `AppActivate` found a window and asked for it. That is not
 * whether the window came to the front, and reading it as though it were
 * cost run 35520201976 its diagnosis: the record kept this `true` and
 * nothing else, so the failure could not say whether the window had come
 * back. Windows lets a process that is not in front find a window and flash
 * its taskbar button rather than raise it, and answers the same either way.
 * What is in front is `foregroundWindow` below, which reads it.
 */
async function activateWindow(title) {
  const { stdout } = await execFileAsync("powershell", [
    "-NoProfile",
    "-Command",
    `(New-Object -ComObject WScript.Shell).AppActivate(${JSON.stringify(title)})`,
  ]);
  return stdout.trim().toLowerCase() === "true";
}

/**
 * The window Windows says is in front, as `{ title, pid }`, or
 * `{ title: "", pid: 0 }` when there is none.
 *
 * Asked through PowerShell for the same reason `windowTitles` is, and built
 * the same way, so the two read alike. This is the read an activation call
 * cannot stand in for: a key goes to whatever is in front, so a case about
 * to press one needs to know what that is, not what it asked for.
 */
async function foregroundWindow() {
  const { stdout } = await execFileAsync("powershell", [
    "-NoProfile",
    "-Command",
    [
      "Add-Type -Namespace WixenNvda -Name Front -MemberDefinition @'",
      '[DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();',
      '[DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, System.Text.StringBuilder s, int n);',
      '[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);',
      "'@",
      "$h = [WixenNvda.Front]::GetForegroundWindow()",
      "$owner = 0",
      "$sb = New-Object System.Text.StringBuilder 512",
      "if ($h -ne [IntPtr]::Zero) {",
      "  [void][WixenNvda.Front]::GetWindowThreadProcessId($h, [ref]$owner)",
      "  [void][WixenNvda.Front]::GetWindowText($h, $sb, 512)",
      "}",
      '$sb.ToString() + "`n" + $owner',
    ].join("\n"),
  ]);
  const [title = "", owner = "0"] = stdout.split(/\r?\n/);
  return { title: title.trim(), pid: Number.parseInt(owner.trim(), 10) || 0 };
}

/**
 * What Windows says has the keyboard, as
 * `{ className, controlType, pid, name }`, or `null` with `why` set when UI
 * Automation answers nothing.
 *
 * The second half of the read above, and the half that tells a window which
 * came back with the keyboard on its document from one which came back with
 * the keyboard somewhere else. A screen reader's browse mode needs a
 * document; a case that presses a browse-mode key without knowing where the
 * keyboard is cannot say, when it hears nothing, whether the key was wrong
 * or the place was.
 *
 * Read-only: `FocusedElement` asks, it does not move anything.
 */
async function focusedElement() {
  try {
    const { stdout } = await execFileAsync("powershell", [
      "-NoProfile",
      "-Command",
      [
        "Add-Type -AssemblyName UIAutomationClient",
        "Add-Type -AssemblyName UIAutomationTypes",
        "$e = [System.Windows.Automation.AutomationElement]::FocusedElement",
        "if ($e -eq $null) { 'none' } else {",
        '  $e.Current.ClassName + "`n" + $e.Current.ControlType.ProgrammaticName + "`n" +',
        '    $e.Current.ProcessId + "`n" + $e.Current.Name',
        "}",
      ].join("\n"),
    ]);
    const lines = stdout.split(/\r?\n/);
    if (lines[0].trim() === "none") {
      return null;
    }
    const [className = "", controlType = "", owner = "0", name = ""] = lines;
    return {
      className: className.trim(),
      controlType: controlType.trim(),
      pid: Number.parseInt(owner.trim(), 10) || 0,
      name: name.trim(),
    };
  } catch (why) {
    return { why: why.message };
  }
}

/**
 * Wait until the window in front belongs to `pid` and its title starts with
 * `title`, and answer what was in front when it did.
 *
 * Throws with the last thing it saw rather than with "timed out", because a
 * case that gave up here has to say what it was looking at instead: the
 * browser still in front and the window never raised is one story, and some
 * third window is another.
 */
async function waitForForeground(pid, title, { timeoutMs = 10000, intervalMs = 250 } = {}) {
  const deadline = Date.now() + timeoutMs;
  let seen = { title: "", pid: 0 };
  while (Date.now() < deadline) {
    seen = await foregroundWindow();
    if (seen.pid === pid && seen.title.startsWith(title)) {
      return seen;
    }
    await sleep(intervalMs);
  }
  throw new Error(
    `${JSON.stringify(title)} (pid ${pid}) was not in front within ${timeoutMs}ms. ` +
      `In front instead: ${JSON.stringify(seen)}`,
  );
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
  windowTitles,
  activateWindow,
  foregroundWindow,
  focusedElement,
  waitForForeground,
  killApp,
  sleep,
};

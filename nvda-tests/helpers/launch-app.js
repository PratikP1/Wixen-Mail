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
 * The window Windows says is in front, as
 * `{ title, pid, className, processName }`, or empty strings and pid 0 when
 * there is none.
 *
 * Asked through PowerShell for the same reason `windowTitles` is, and built
 * the same way, so the two read alike. This is the read an activation call
 * cannot stand in for: a key goes to whatever is in front, so a case about
 * to press one needs to know what that is, not what it asked for.
 *
 * The class and the process's name since 2026-09-23. Runs 35839692317 and
 * 35839954840 found an untitled window of another process in front after
 * Alt+Tab, pid 2036, and a title and a pid could not say whose it was.
 */
async function foregroundWindow() {
  const { stdout } = await execFileAsync("powershell", [
    "-NoProfile",
    "-Command",
    [
      "Add-Type -Namespace WixenNvda -Name Front -MemberDefinition @'",
      '[DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();',
      '[DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, System.Text.StringBuilder s, int n);',
      '[DllImport("user32.dll")] public static extern int GetClassName(IntPtr h, System.Text.StringBuilder s, int n);',
      '[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);',
      "'@",
      "$h = [WixenNvda.Front]::GetForegroundWindow()",
      "$owner = 0",
      "$sb = New-Object System.Text.StringBuilder 512",
      "$cls = New-Object System.Text.StringBuilder 256",
      "$name = ''",
      "if ($h -ne [IntPtr]::Zero) {",
      "  [void][WixenNvda.Front]::GetWindowThreadProcessId($h, [ref]$owner)",
      "  [void][WixenNvda.Front]::GetWindowText($h, $sb, 512)",
      "  [void][WixenNvda.Front]::GetClassName($h, $cls, 256)",
      "  $name = (Get-Process -Id $owner -ErrorAction SilentlyContinue).ProcessName",
      "}",
      '$sb.ToString() + "`n" + $owner + "`n" + $cls.ToString() + "`n" + $name',
    ].join("\n"),
  ]);
  const [title = "", owner = "0", className = "", processName = ""] = stdout.split(/\r?\n/);
  return {
    title: title.trim(),
    pid: Number.parseInt(owner.trim(), 10) || 0,
    className: className.trim(),
    processName: processName.trim(),
  };
}

/**
 * The PowerShell body `watchTheKeyboardOfTheWindow` runs, one process for
 * every sample. Kept apart from the function so the body can be run by hand
 * and read as it is.
 *
 * The process id is compared before any title is read, so a window of some
 * other process, on a machine somebody is using, is never read beyond whose
 * it is. What it writes, a line at a time:
 *   `ready`, once the `Add-Type` block has compiled;
 *   `not-in-front <pid of whatever was in front>`, when the window never came;
 *   `sighted <system time in ms>`, when it did;
 *   `sample <planned ms> <measured ms> <focus handle> <focus is the window>
 *     <focus is inside it> <the window is its thread's active one> <focus class>`;
 *   `unanswered <planned ms> <measured ms>`, when `GetGUIThreadInfo` failed;
 *   `done`.
 */
function theWatchersScript(pid, title, atMs, waitForTheFrontMs) {
  return [
    `$pidWanted = ${Number(pid)}`,
    // A single-quoted literal with its quotes doubled, in which PowerShell
    // expands nothing. `JSON.stringify` makes a double-quoted one, in which
    // `$(...)` in a title would run.
    `$title = '${String(title).replace(/'/g, "''")}'`,
    `$atMs = @(${atMs.map((at) => Number(at)).join(",")})`,
    `$waitMs = ${Number(waitForTheFrontMs)}`,
    "Add-Type -Namespace WixenNvda -Name Keyboard -MemberDefinition @'",
    "[StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }",
    "[StructLayout(LayoutKind.Sequential)] public struct GUITHREADINFO {",
    "  public int cbSize; public int flags; public IntPtr hwndActive; public IntPtr hwndFocus;",
    "  public IntPtr hwndCapture; public IntPtr hwndMenuOwner; public IntPtr hwndMoveSize;",
    "  public IntPtr hwndCaret; public RECT rcCaret; }",
    '[DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();',
    '[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);',
    '[DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, System.Text.StringBuilder s, int n);',
    '[DllImport("user32.dll")] public static extern int GetClassName(IntPtr h, System.Text.StringBuilder s, int n);',
    '[DllImport("user32.dll")] public static extern bool IsChild(IntPtr parent, IntPtr child);',
    '[DllImport("user32.dll")] public static extern bool GetGUIThreadInfo(uint thread, ref GUITHREADINFO info);',
    "'@",
    "[Console]::WriteLine('ready')",
    "$waiting = [System.Diagnostics.Stopwatch]::StartNew()",
    "$h = [IntPtr]::Zero; $thread = 0; $owner = 0; $found = $false",
    "while ($waiting.ElapsedMilliseconds -lt $waitMs) {",
    "  $h = [WixenNvda.Keyboard]::GetForegroundWindow()",
    "  $owner = 0",
    "  $thread = [WixenNvda.Keyboard]::GetWindowThreadProcessId($h, [ref]$owner)",
    "  if ($h -ne [IntPtr]::Zero -and $owner -eq $pidWanted) {",
    "    $sb = New-Object System.Text.StringBuilder 512",
    "    [void][WixenNvda.Keyboard]::GetWindowText($h, $sb, 512)",
    "    if ($sb.ToString().StartsWith($title)) { $found = $true; break }",
    "  }",
    "  Start-Sleep -Milliseconds 20",
    "}",
    "if (-not $found) { [Console]::WriteLine('not-in-front ' + $owner); exit 0 }",
    "[Console]::WriteLine('sighted ' + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())",
    "$since = [System.Diagnostics.Stopwatch]::StartNew()",
    "foreach ($at in $atMs) {",
    "  while ($since.ElapsedMilliseconds -lt $at) { Start-Sleep -Milliseconds 5 }",
    "  $info = New-Object WixenNvda.Keyboard+GUITHREADINFO",
    "  $info.cbSize = [System.Runtime.InteropServices.Marshal]::SizeOf($info)",
    "  $answered = [WixenNvda.Keyboard]::GetGUIThreadInfo($thread, [ref]$info)",
    "  $measured = $since.ElapsedMilliseconds",
    "  if (-not $answered) { [Console]::WriteLine('unanswered ' + $at + ' ' + $measured); continue }",
    "  $focus = $info.hwndFocus",
    "  $cls = New-Object System.Text.StringBuilder 256",
    "  if ($focus -ne [IntPtr]::Zero) { [void][WixenNvda.Keyboard]::GetClassName($focus, $cls, 256) }",
    "  $isTheWindow = $focus -eq $h",
    "  $isInside = ($focus -ne [IntPtr]::Zero) -and [WixenNvda.Keyboard]::IsChild($h, $focus)",
    "  $activeIsTheWindow = $info.hwndActive -eq $h",
    "  [Console]::WriteLine('sample ' + $at + ' ' + $measured + ' ' + $focus.ToInt64() + ' ' + $isTheWindow + ' ' + $isInside + ' ' + $activeIsTheWindow + ' ' + $cls.ToString())",
    "}",
    "[Console]::WriteLine('done')",
  ].join("\n");
}

/** One `sample` line of the watcher's, as the record keeps it. */
function aSampleFrom(words) {
  const [, planned, measured, focus, isTheWindow, isInside, activeIsTheWindow, ...cls] = words;
  return {
    plannedMs: Number(planned),
    measuredMs: Number(measured),
    focus: Number(focus),
    focusClass: cls.join(" "),
    focusIsTheWindow: isTheWindow === "True",
    focusIsInsideIt: isInside === "True",
    activeIsTheWindow: activeIsTheWindow === "True",
  };
}

/**
 * Watch what the window's own thread says has the keyboard, from the moment
 * the window comes to the front, at each offset in `atMs`.
 *
 * Answers `{ ready, samples, pid, stop }`. `ready` resolves `true` once the
 * watcher can see the window arrive, so a case starts it and awaits `ready`
 * before it does anything that could bring the window back. `samples`
 * resolves `{ sightedAt, samples }`, `sightedAt` in system milliseconds and
 * every sample with the offset it was planned for and the one its own
 * stopwatch measured, or `{ why, samples }` when the window never came to
 * the front, a sample went unanswered, or the process ended without its last
 * line.
 *
 * **Why the thread and not UI Automation.** `focusedElement` reads UI
 * Automation's focused element, which cannot tell a frame holding the
 * keyboard from nothing holding it, and runs 35839692317 and 35839954840
 * could not say which of three things had happened to the page window's
 * keyboard. `GetGUIThreadInfo` answers for the window's own thread.
 *
 * **Why one process and not a call per sample.** Every other helper here
 * starts its own PowerShell and compiles its own `Add-Type` block. Measured
 * on 2026-09-23 on the machine 12-03.1 was planned on, a start with one
 * compile took 478, 361 and 343 ms, and a cold runner is slower, so calls
 * made one after another cannot put a sample at 0 ms and another at 250 ms,
 * and those two are what tell a keyboard that was in the page and then left
 * it from one that was never there. One process started before the return
 * pays the compile before the window comes back and records the offset it
 * measured beside the one it planned, so a late sample says it was late.
 *
 * Read-only: nothing is moved, nothing is pressed, and only the window's
 * own process has its title read.
 */
function watchTheKeyboardOfTheWindow(
  pid,
  title,
  { atMs = [0, 250, 1000, 3000], waitForTheFrontMs = 25000 } = {},
) {
  const child = spawn(
    "powershell",
    ["-NoProfile", "-Command", theWatchersScript(pid, title, atMs, waitForTheFrontMs)],
    { stdio: ["ignore", "pipe", "pipe"], windowsHide: true },
  );
  let sayReady;
  let saySamples;
  const ready = new Promise((resolve) => {
    sayReady = resolve;
  });
  const samples = new Promise((resolve) => {
    saySamples = resolve;
  });
  const seen = { sightedAt: null, samples: [] };
  let stderr = "";
  let pending = "";

  const readLine = (line) => {
    const words = line.trim().split(" ");
    switch (words[0]) {
      case "ready":
        sayReady(true);
        break;
      case "not-in-front":
        saySamples({ why: `the window never came to the front; process ${words[1]} was in front`, ...seen });
        break;
      case "sighted":
        seen.sightedAt = Number(words[1]);
        break;
      case "sample":
        seen.samples.push(aSampleFrom(words));
        break;
      case "unanswered":
        saySamples({ why: `GetGUIThreadInfo answered nothing at ${words[1]} ms (measured ${words[2]})`, ...seen });
        break;
      case "done":
        saySamples(seen);
        break;
      default:
        break;
    }
  };

  child.stdout.on("data", (chunk) => {
    pending += chunk.toString();
    const lines = pending.split(/\r?\n/);
    pending = lines.pop();
    lines.filter((line) => line.trim().length > 0).forEach(readLine);
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });
  child.on("close", (code) => {
    const why = `the watcher ended (code ${code}) without its last line${stderr ? `: ${stderr.trim()}` : ""}`;
    sayReady({ why });
    saySamples({ why, ...seen });
  });

  return {
    ready,
    samples,
    pid: child.pid,
    stop: () => {
      if (child.exitCode === null && !child.killed) {
        child.kill();
      }
    },
  };
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
  watchTheKeyboardOfTheWindow,
  killApp,
  sleep,
};

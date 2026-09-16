<#
.SYNOPSIS
Log the accessibility events a running Wixen Mail raises while an arrow key
moves along the Settings tab row.

.DESCRIPTION
Written for #33, "arrowing left/right on a tab list in the settings dialog
frequently reads the focused tab twice". Nothing in `wx_settings.rs` announces
a page change, so both readings come from what the control raises and what the
screen reader does with it. This script records the first half. It cannot see
the second: what NVDA says about an event is NVDA's, and only its own log at
debug level, or the NVDA case under `nvda-tests/`, can show that.

Two channels are logged, because Windows has two and NVDA reads the older one
for a native tab control.

UI Automation, the newer channel: a focus-changed handler over the whole
desktop filtered to the process, `ElementSelected` and `SelectionInvalidated`
over the process's windows, and a `Name` property-changed handler over the
same. Narrator reads this channel, and the managed client in
`UIAutomationClient.dll` is what subscribes here.

Win events, the older channel: `SetWinEventHook`, out of context, scoped to the
process, for `EVENT_SYSTEM_FOREGROUND` and every `EVENT_OBJECT_*` except
location changes. `EVENT_OBJECT_FOCUS`, `EVENT_OBJECT_SELECTION` and
`EVENT_OBJECT_NAMECHANGE` are what NVDA acts on for a `SysTabControl32`, which
it reads through `IAccessible`. This script never asks the `IAccessible` object
for anything: walking one crashes PowerShell on this machine (ledger 390,
`STATUS_STACK_BUFFER_OVERRUN`), and the event's window class, window text,
object id and child id are enough to say which control raised it and which tab
it was about. A tab item's child id is its index plus one.

The keys are posted to the tab control's own window as `WM_KEYDOWN` and
`WM_KEYUP`, not sent through `SendInput` or `SendKeys`. Both of those need the
window to be in the foreground, and on a locked session neither can put it
there: `SendInput` answered `ERROR_ACCESS_DENIED` after the first press and
`SendKeys.SendWait` threw "The operation completed successfully" on
2026-09-16. A posted key goes through the application's own message loop and
reaches the control's own key handler, so the events logged are the ones a
real key raises, with one difference the header must say: the tab control is
asked to move without ever having taken foreground focus. The script tries
`SetFocus()` on the tab row first and prints which element was focused after,
so a run can say whether it had the foreground or not.

How it was used: on 2026-09-16, against `target/release/wixen-mail.exe` at
`96298371`, on a throwaway profile, with NVDA running and the session locked.
The capture is quoted in `.planning/phases/09-what-the-first-day-of-testing-found/09-06-SUMMARY.md`.

.PARAMETER ProcessId
A running Wixen Mail to attach to. When absent, the script starts `-Exe` with
`--scan-target settings` on a throwaway profile under the temp folder and ends
it afterwards.

.PARAMETER Exe
The binary to start when no process id is given. The release build, by default.

.PARAMETER Presses
How many times to press Right, then Left. Three each, by default.

.PARAMETER SettleMs
How long to wait after each key for events to arrive before printing them.

.OUTPUTS
One line per event: milliseconds since the hook was set, the channel, the
event kind, the control (a control type and name on the UI Automation channel;
a window class, window text, object id and child id on the win-event channel).
After the keys, a table of counts per key and per event kind, and a total. Exit
code 0 when at least one event was logged; 2 when the process, the dialog, the
tab row or the events could not be reached, which is not a clean result and
has to stay distinguishable from one.
#>
[CmdletBinding()]
param(
  [int]$ProcessId,
  [string]$Exe = (Join-Path $PSScriptRoot '..' 'target' 'release' 'wixen-mail.exe'),
  [int]$Presses = 3,
  [int]$SettleMs = 1200
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$uiaClient = [System.Windows.Automation.AutomationElement].Assembly.Location
$uiaTypes = [System.Windows.Automation.ControlType].Assembly.Location

# The handlers live in C#. A UI Automation client and a win-event hook both
# call back on threads of their own, and a PowerShell script block invoked on a
# thread PowerShell does not own stops the script with no message. C# code
# enqueues a line per event and PowerShell drains the queue between keys.
Add-Type -ReferencedAssemblies $uiaClient, $uiaTypes, 'WindowsBase', 'System.Collections.Concurrent', 'System.Runtime.InteropServices', 'System.Threading.Thread' -TypeDefinition @"
using System;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Windows.Automation;

public static class UiaEvents {
  public static ConcurrentQueue<string> Lines = new ConcurrentQueue<string>();
  public static Stopwatch Clock = Stopwatch.StartNew();
  public static int Pid;

  static string Describe(AutomationElement el) {
    if (el == null) return "?";
    try {
      var c = el.Current;
      return c.ControlType.ProgrammaticName.Replace("ControlType.", "") + " name='" + c.Name + "'"
        + (c.AutomationId.Length > 0 ? " id='" + c.AutomationId + "'" : "");
    } catch (Exception ex) { return "(unreadable: " + ex.GetType().Name + ")"; }
  }
  static bool Ours(AutomationElement el) {
    try { return el != null && el.Current.ProcessId == Pid; } catch { return false; }
  }
  static void Log(string channel, string kind, string what) {
    Lines.Enqueue(Clock.ElapsedMilliseconds + " " + channel + " " + kind + " " + what);
  }

  public static void OnFocus(object sender, AutomationFocusChangedEventArgs e) {
    var el = sender as AutomationElement;
    if (!Ours(el)) return;
    Log("uia", "FocusChanged", Describe(el));
  }
  public static void OnEvent(object sender, AutomationEventArgs e) {
    Log("uia", e.EventId.ProgrammaticName.Replace("PatternIdentifiers.", "."), Describe(sender as AutomationElement));
  }
  public static void OnProperty(object sender, AutomationPropertyChangedEventArgs e) {
    Log("uia", "PropertyChanged " + e.Property.ProgrammaticName, "-> '" + e.NewValue + "' " + Describe(sender as AutomationElement));
  }

  // The win-event hook. Out of context, so it needs a message loop on the
  // thread that set it; that thread is started below and never joined.
  public delegate void WinEventProc(IntPtr hook, uint ev, IntPtr hwnd, int idObject, int idChild, uint thread, uint time);
  [DllImport("user32.dll")] static extern IntPtr SetWinEventHook(uint min, uint max, IntPtr mod, WinEventProc proc, uint pid, uint tid, uint flags);
  [DllImport("user32.dll")] static extern bool GetMessage(out MSG msg, IntPtr hwnd, uint min, uint max);
  [DllImport("user32.dll")] static extern bool TranslateMessage(ref MSG msg);
  [DllImport("user32.dll")] static extern IntPtr DispatchMessage(ref MSG msg);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassName(IntPtr hwnd, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr hwnd, StringBuilder s, int n);
  [StructLayout(LayoutKind.Sequential)] struct MSG { public IntPtr hwnd; public uint message; public IntPtr wParam; public IntPtr lParam; public uint time; public int x; public int y; }
  static WinEventProc keepAlive;

  static string EventName(uint ev) {
    switch (ev) {
      case 0x0003: return "SYSTEM_FOREGROUND";
      case 0x8000: return "OBJECT_CREATE"; case 0x8001: return "OBJECT_DESTROY";
      case 0x8002: return "OBJECT_SHOW"; case 0x8003: return "OBJECT_HIDE"; case 0x8004: return "OBJECT_REORDER";
      case 0x8005: return "OBJECT_FOCUS"; case 0x8006: return "OBJECT_SELECTION"; case 0x8007: return "OBJECT_SELECTIONADD";
      case 0x8008: return "OBJECT_SELECTIONREMOVE"; case 0x8009: return "OBJECT_SELECTIONWITHIN"; case 0x800A: return "OBJECT_STATECHANGE";
      case 0x800C: return "OBJECT_NAMECHANGE"; case 0x800D: return "OBJECT_DESCRIPTIONCHANGE"; case 0x800E: return "OBJECT_VALUECHANGE";
      case 0x800F: return "OBJECT_PARENTCHANGE"; case 0x8013: return "OBJECT_INVOKED"; case 0x8014: return "OBJECT_TEXTSELECTIONCHANGED";
      default: return "0x" + ev.ToString("X4");
    }
  }
  static void HookThread(object pidObj) {
    uint pid = (uint)(int)pidObj;
    keepAlive = (hook, ev, hwnd, idObject, idChild, thread, time) => {
      if (ev == 0x800B) return; // EVENT_OBJECT_LOCATIONCHANGE: every repaint, and never spoken
      var cls = new StringBuilder(256); GetClassName(hwnd, cls, 256);
      var txt = new StringBuilder(256); GetWindowText(hwnd, txt, 256);
      Log("winevent", EventName(ev), "class='" + cls + "' text='" + txt + "' hwnd=" + hwnd + " idObject=" + idObject + " idChild=" + idChild);
    };
    SetWinEventHook(0x0003, 0x0003, IntPtr.Zero, keepAlive, pid, 0, 0);
    SetWinEventHook(0x8000, 0x80FF, IntPtr.Zero, keepAlive, pid, 0, 0);
    MSG m;
    while (GetMessage(out m, IntPtr.Zero, 0, 0)) { TranslateMessage(ref m); DispatchMessage(ref m); }
  }
  public static void StartHook(int pid) {
    var t = new Thread(HookThread); t.IsBackground = true; t.Start(pid);
  }

  // A key posted to the control's own window: down, then up, with the
  // repeat count of one and the transition bit set on the up.
  [DllImport("user32.dll", SetLastError = true)] static extern bool PostMessage(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);
  public static bool PostKey(IntPtr hwnd, int vk) {
    bool down = PostMessage(hwnd, 0x0100, (IntPtr)vk, (IntPtr)1);
    bool up = PostMessage(hwnd, 0x0101, (IntPtr)vk, unchecked((IntPtr)(long)0xC0000001));
    return down && up;
  }
}
"@

# Said on stderr and left with the code that means so, on
# `msaa-names.ps1`'s pattern and for its reason: under the Stop preference,
# Write-Error would end the script before the exit after it.
function Fail-Capture($why) {
  [Console]::Error.WriteLine($why)
  exit 2
}

$started = $null
if (-not $ProcessId) {
  if (-not (Test-Path $Exe)) {
    Fail-Capture "No binary at $Exe. Build the release first, or pass -ProcessId."
  }
  # A throwaway profile, never the real one: the application reads its data
  # folder from WIXEN_MAIL_DATA before the known-folder API.
  $profile = Join-Path ([System.IO.Path]::GetTempPath()) 'wixen-mail-uia-events-profile'
  Remove-Item -Recurse -Force $profile -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force $profile | Out-Null
  $env:WIXEN_MAIL_DATA = $profile
  $started = Start-Process -FilePath $Exe -ArgumentList @('--scan-target', 'settings') -PassThru
  $ProcessId = $started.Id
  $deadline = (Get-Date).AddSeconds(60)
  while ((Get-Date) -lt $deadline -and $started.MainWindowHandle -eq 0 -and -not $started.HasExited) {
    Start-Sleep -Milliseconds 500
    $started.Refresh()
  }
  if ($started.HasExited) { Fail-Capture "The application left with code $($started.ExitCode) before showing a window." }
  # The dialog opens after the frame is shown; the same settle the
  # accessibility workflow gives it.
  Start-Sleep -Seconds 3
}

try {
  $process = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
  if (-not $process) { Fail-Capture "No process $ProcessId. Nothing was captured." }
  [UiaEvents]::Pid = $ProcessId
  [UiaEvents]::StartHook($ProcessId)

  $root = [System.Windows.Automation.AutomationElement]::RootElement
  $byPid = New-Object System.Windows.Automation.PropertyCondition(
    [System.Windows.Automation.AutomationElement]::ProcessIdProperty, $ProcessId)
  $windows = $root.FindAll([System.Windows.Automation.TreeScope]::Children, $byPid)
  if ($windows.Count -eq 0) { Fail-Capture "Process $ProcessId has no top-level window. Nothing was captured." }
  $isTab = New-Object System.Windows.Automation.PropertyCondition(
    [System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Tab)
  $tab = $null
  $scope = $null
  foreach ($window in $windows) {
    $found = $window.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $isTab)
    if ($found) { $tab = $found; $scope = $window; break }
  }
  if (-not $tab) { Fail-Capture "No tab control under any window of process $ProcessId. Nothing was captured." }
  $tabHwnd = [IntPtr]$tab.Current.NativeWindowHandle
  Write-Host ("Tab row: class='{0}' hwnd={1} under '{2}'" -f $tab.Current.ClassName, $tabHwnd, $scope.Current.Name)
  $isTabItem = New-Object System.Windows.Automation.PropertyCondition(
    [System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::TabItem)
  $items = @($tab.FindAll([System.Windows.Automation.TreeScope]::Children, $isTabItem) | ForEach-Object { $_.Current.Name })
  Write-Host ("Tabs: {0}" -f ($items -join ' | '))

  [System.Windows.Automation.Automation]::AddAutomationFocusChangedEventHandler(
    [System.Windows.Automation.AutomationFocusChangedEventHandler][UiaEvents]::OnFocus)
  foreach ($window in $windows) {
    [System.Windows.Automation.Automation]::AddAutomationEventHandler(
      [System.Windows.Automation.SelectionItemPattern]::ElementSelectedEvent, $window,
      [System.Windows.Automation.TreeScope]::Subtree, [System.Windows.Automation.AutomationEventHandler][UiaEvents]::OnEvent)
    [System.Windows.Automation.Automation]::AddAutomationEventHandler(
      [System.Windows.Automation.SelectionPattern]::InvalidatedEvent, $window,
      [System.Windows.Automation.TreeScope]::Subtree, [System.Windows.Automation.AutomationEventHandler][UiaEvents]::OnEvent)
    [System.Windows.Automation.Automation]::AddAutomationPropertyChangedEventHandler(
      $window, [System.Windows.Automation.TreeScope]::Subtree,
      [System.Windows.Automation.AutomationPropertyChangedEventHandler][UiaEvents]::OnProperty,
      @([System.Windows.Automation.AutomationElement]::NameProperty))
  }

  # Asked for, and then reported rather than assumed: on a locked session no
  # window can take the foreground, and the focused element says so.
  $tab.SetFocus()
  Start-Sleep -Milliseconds 1500
  $focused = [System.Windows.Automation.AutomationElement]::FocusedElement
  Write-Host ("Focused element after SetFocus on the tab row: {0} '{1}' in process {2}" -f `
    $focused.Current.ControlType.ProgrammaticName, $focused.Current.Name, $focused.Current.ProcessId)
  $line = $null
  Write-Host 'Before the first key:'
  while ([UiaEvents]::Lines.TryDequeue([ref]$line)) { Write-Host "  $line" }

  $counts = [ordered]@{}
  $total = 0
  $keys = @(1..$Presses | ForEach-Object { 'Right' }) + @(1..$Presses | ForEach-Object { 'Left' })
  $press = 0
  foreach ($key in $keys) {
    $press++
    $vk = if ($key -eq 'Right') { 0x27 } else { 0x25 }
    $at = [UiaEvents]::Clock.ElapsedMilliseconds
    $posted = [UiaEvents]::PostKey($tabHwnd, $vk)
    Start-Sleep -Milliseconds $SettleMs
    Write-Host ("--- press {0}: {1} at {2} ms, posted={3}" -f $press, $key, $at, $posted)
    $perKey = [ordered]@{}
    while ([UiaEvents]::Lines.TryDequeue([ref]$line)) {
      Write-Host "  $line"
      $total++
      $parts = $line -split ' ', 4
      $kind = $parts[1] + ' ' + $parts[2]
      if (-not $perKey.Contains($kind)) { $perKey[$kind] = 0 }
      $perKey[$kind]++
    }
    $counts["press $press ($key)"] = $perKey
  }

  [System.Windows.Automation.Automation]::RemoveAllEventHandlers()

  Write-Host ''
  Write-Host 'Counts per key and per event kind:'
  foreach ($entry in $counts.GetEnumerator()) {
    $said = @($entry.Value.GetEnumerator() | ForEach-Object { "{0} x{1}" -f $_.Key, $_.Value }) -join ', '
    if (-not $said) { $said = 'nothing' }
    Write-Host ("  {0}: {1}" -f $entry.Key, $said)
  }
  Write-Host ("Total events logged over {0} presses: {1}" -f $press, $total)
  if ($total -eq 0) {
    Fail-Capture 'No event arrived on either channel. A capture with nothing in it is a broken capture, not a quiet control.'
  }
} finally {
  if ($started -and -not $started.HasExited) { $started.Kill() }
}
exit 0

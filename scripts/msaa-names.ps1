<#
.SYNOPSIS
Walk a running Wixen Mail's MSAA tree and report the accessible names on it.

.DESCRIPTION
The UI Automation scan next to this one has never measured a name this codebase
sets, and could not have.

On Windows there are two accessibility channels. UI Automation is the newer one,
and Axe.Windows reads it. MSAA, through IAccessible, is the older one, and it is
what `wxAccessible` implements: `set_accessible_name` puts a name there and
nowhere else. For a native control such as an edit box or a button, Windows
supplies its own UI Automation provider, and that provider shadows the MSAA
object underneath it. So UI Automation reports the system's name for those
controls, which is usually empty, and never the one the code set.

That makes the UI Automation scan wrong in both directions on native controls:
it reports a missing name where the name is in fact present and spoken, and it
would report nothing amiss if every `set_accessible_name` call in the tree were
deleted.

NVDA reads IAccessible for these controls. This script reads the same thing, so
what it reports is what a screen reader user gets. It does not replace the UI
Automation scan: Narrator reads UI Automation, and both have to be right.

Two things to know when reading a clean result.

A control with a visible label beside it is named by that label even when
nothing set one, because Windows falls back to the nearest static text. That is
correct behaviour and the name really is spoken, so a pass here does not mean
every name came from this codebase. It means every operated control has one.

And this only sees the windows that exist while it runs. A dialog that is not
open is not a dialog with nothing wrong with it, which is why the workflow
starts the application once per window with `--scan-target`. Every visible
top-level window the process owns is walked, the dialog and the frame behind
it, and each finding's path starts with the title of the window it is in.
Until 2026-09-14 only the window .NET calls the main one was walked, which is
never a dialog, so no dialog had been read on this channel by anything.

Proved by removing one `set_accessible_name` call and watching this report the
control it belonged to. Do that again before believing a clean run.

.PARAMETER ProcessId
The running process to walk.

.PARAMETER Json
Write the whole tree as JSON to this path, for the CI artifact.

.OUTPUTS
A line per unnamed interactive control, and a count. Exit code 0 when every
interactive control has a name, 1 when any does not, 2 when the walk itself
failed, which is a different thing from a clean result and has to stay
distinguishable from one.
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  [string]$Json
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

public static class Msaa {
  [DllImport("oleacc.dll")]
  public static extern int AccessibleObjectFromWindow(
    IntPtr hwnd, uint id, ref Guid iid,
    [MarshalAs(UnmanagedType.IUnknown)] out object ppvObject);

  [DllImport("oleacc.dll")]
  public static extern int AccessibleChildren(
    [MarshalAs(UnmanagedType.IUnknown)] object paccContainer,
    int iChildStart, int cChildren,
    [Out, MarshalAs(UnmanagedType.LPArray, ArraySubType = UnmanagedType.Struct)] object[] rgvarChildren,
    out int pcObtained);

  // IAccessible is a dual interface, so once the object is in hand every
  // property can be reached through IDispatch. Declaring the whole vtable would
  // add sixty lines and one more place to get an ordering wrong.
  public static readonly Guid IID_IAccessible =
    new Guid("618736E0-3C3D-11CF-810C-00AA00389B71");

  public const uint OBJID_CLIENT = 0xFFFFFFFC;

  // For finding every top-level window the process owns, rather than the
  // one .NET calls its main window. See the comment where they are used.
  public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);

  [DllImport("user32.dll")]
  public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);

  [DllImport("user32.dll")]
  public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);

  [DllImport("user32.dll")]
  public static extern bool IsWindowVisible(IntPtr hwnd);

  [DllImport("user32.dll", CharSet = CharSet.Unicode)]
  public static extern int GetWindowText(IntPtr hwnd, System.Text.StringBuilder text, int max);
}
"@

# The roles somebody operates. An unnamed grouping box is untidy; an unnamed
# edit box is a control a screen reader user cannot identify, which is the thing
# worth failing over. Numbers from oleacc.h, because that is the form accRole
# returns.
$OPERATED = @{
  42 = 'editable text'
  43 = 'push button'
  44 = 'check box'
  45 = 'radio button'
  46 = 'combo box'
  33 = 'list'
  34 = 'list item'
  35 = 'outline'
  36 = 'outline item'
  37 = 'page tab'
  38 = 'property page'
  50 = 'slider'
  51 = 'spin button'
  57 = 'hotkey field'
  58 = 'slider'
  61 = 'progress bar'
}

# How many elements would not answer at all. Counted rather than swallowed: the
# first version of this returned a default on failure, every call failed, and it
# reported "0 without a name" over a tree it had read nothing from. A check that
# cannot tell "everything is named" from "nothing answered" is worse than none.
$script:unreadable = 0

# accName and accRole take a child argument, so they are parameterized COM
# properties. `$acc.accName(0)` does not bind to one; InvokeMember does.
function Get-Property($acc, $name, $child) {
  try {
    return $acc.GetType().InvokeMember(
      $name, [System.Reflection.BindingFlags]::GetProperty,
      $null, $acc, @([object]$child))
  } catch {
    $script:unreadable++
    return $null
  }
}

function Get-Name($acc, $child) {
  $value = Get-Property $acc 'accName' $child
  if ($null -eq $value) { return '' }
  return [string]$value
}

function Get-Role($acc, $child) {
  $value = Get-Property $acc 'accRole' $child
  if ($null -eq $value) { return -1 }
  return [int]$value
}

function Get-Children($acc) {
  $count = 0
  try { $count = [int]$acc.accChildCount } catch { return @() }
  if ($count -le 0) { return @() }
  $buffer = New-Object object[] $count
  $got = 0
  $hr = [Msaa]::AccessibleChildren($acc, 0, $count, $buffer, [ref]$got)
  if ($hr -ne 0 -and $got -eq 0) { return @() }
  return $buffer[0..([Math]::Max($got - 1, 0))]
}

$found = New-Object System.Collections.ArrayList

function Walk($acc, $depth, $path) {
  if ($depth -gt 20) { return }
  foreach ($child in Get-Children $acc) {
    if ($null -eq $child) { continue }
    if ($child -is [int]) {
      # A child with no object of its own: ask the parent about it by id.
      $name = Get-Name $acc $child
      $role = Get-Role $acc $child
      $null = $found.Add([pscustomobject]@{
        path = $path; role = $role
        roleName = if ($OPERATED.ContainsKey($role)) { $OPERATED[$role] } else { "role $role" }
        name = $name
        operated = $OPERATED.ContainsKey($role)
      })
      continue
    }
    $name = Get-Name $child 0
    $role = Get-Role $child 0
    $null = $found.Add([pscustomobject]@{
      path = $path; role = $role
      roleName = if ($OPERATED.ContainsKey($role)) { $OPERATED[$role] } else { "role $role" }
      name = $name
      operated = $OPERATED.ContainsKey($role)
    })
    Walk $child ($depth + 1) ("$path/" + $(if ($name) { $name } else { "role $role" }))
  }
}

# A walk that did not happen, said on stderr and left with the code that
# means so. Not Write-Error: under the Stop preference set above, Write-Error
# terminates the script before the `exit 2` after it runs, and PowerShell
# leaves with 1, which is the code for "an operated control has no name". So
# every failed walk this script has ever had was reported to the workflow as
# an unnamed control, and never as a walk that failed. Measured 2026-09-14 by
# asking for a process that does not exist: exit 1, where the header says 2.
function Fail-Walk($why) {
  [Console]::Error.WriteLine($why)
  exit 2
}

# Every visible top-level window the process owns, in the order Windows
# enumerates them, which is front to back.
#
# Not $process.MainWindowHandle. .NET's main window is the first visible
# top-level window that has no owner, and a wxWidgets dialog is owned by the
# frame it opened from, so for every dialog the workflow opens this walked
# the frame behind it and never the dialog. The run of 2026-09-10 shows it:
# 1797 elements, 1058 operated, for accounts, compose and filters alike,
# which is the main window three times over. The channel NVDA reads had
# never been asked about a single dialog.
#
# The callback reads and writes script-scoped variables on purpose. Windows
# calls it from native code, where a function's own parameters and locals are
# out of reach, and under strict mode an unreachable variable is an error the
# native caller has nowhere to report: the script simply stops, with no
# message and no exit code of its own.
$script:windowsOf = 0
$script:windowsFound = New-Object System.Collections.ArrayList
$script:collectWindow = [Msaa+EnumWindowsProc]{
  param($hwnd, $lParam)
  [uint32]$owner = 0
  [void][Msaa]::GetWindowThreadProcessId($hwnd, [ref]$owner)
  if ($owner -eq $script:windowsOf -and [Msaa]::IsWindowVisible($hwnd)) {
    $text = New-Object System.Text.StringBuilder 512
    [void][Msaa]::GetWindowText($hwnd, $text, 512)
    $null = $script:windowsFound.Add([pscustomobject]@{ hwnd = $hwnd; title = $text.ToString() })
  }
  return $true
}
function Get-TopLevelWindows([int]$owningProcess) {
  $script:windowsOf = $owningProcess
  $script:windowsFound.Clear()
  [void][Msaa]::EnumWindows($script:collectWindow, [IntPtr]::Zero)
  return $script:windowsFound
}

$process = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
if (-not $process) {
  Fail-Walk "No process $ProcessId. Nothing was walked, which is not the same as nothing being wrong."
}
# Wrapped in @(): a function's return unrolls a list, so one window would
# come back as a bare object with no Count, and strict mode stops there.
$windows = @(Get-TopLevelWindows $ProcessId)
if ($windows.Count -eq 0) {
  Fail-Walk "Process $ProcessId has no window yet. Nothing was walked."
}

foreach ($window in $windows) {
  $iid = [Msaa]::IID_IAccessible
  $root = $null
  $hr = [Msaa]::AccessibleObjectFromWindow($window.hwnd, [Msaa]::OBJID_CLIENT, [ref]$iid, [ref]$root)
  if ($hr -ne 0 -or $null -eq $root) {
    Fail-Walk ("Could not get an IAccessible for the window '{0}' (hr=0x{1:X}). Nothing was walked." -f $window.title, $hr)
  }
  # The window's title leads the path, so a finding says which window it is
  # in now that there can be more than one.
  Walk $root 0 ("[" + $window.title + "] " + (Get-Name $root 0))
}
Write-Host ("Walked {0} window(s): {1}" -f $windows.Count, (($windows | ForEach-Object { "'" + $_.title + "'" }) -join ', '))

if ($found.Count -eq 0) {
  Fail-Walk "The walk reached no controls at all. A tree with nothing in it is a broken walk, not a clean one."
}

# Two properties are asked of each element, so this is the ceiling.
$asked = $found.Count * 2
if ($script:unreadable -ge $asked) {
  Fail-Walk ("Not one of {0} elements answered. The walk ran and read nothing, which is a broken walk and not a clean result." -f $found.Count)
}
if ($script:unreadable -gt 0) {
  # Said out loud rather than folded into the total. An element that could not
  # be read is not an element with nothing wrong with it.
  Write-Host ("{0} of {1} property reads failed; those elements are not counted either way." -f $script:unreadable, $asked)
}

if ($Json) {
  $found | ConvertTo-Json -Depth 4 | Set-Content -Path $Json -Encoding UTF8
}

$operated = @($found | Where-Object { $_.operated })
$unnamed = @($operated | Where-Object { -not $_.name -or -not $_.name.Trim() })

Write-Host ("MSAA walk: {0} elements, {1} of them operated, {2} without a name." -f `
  $found.Count, $operated.Count, $unnamed.Count)

foreach ($one in $unnamed) {
  Write-Host ("  no name: {0} at {1}" -f $one.roleName, $one.path)
}

if ($unnamed.Count -gt 0) { exit 1 }
exit 0

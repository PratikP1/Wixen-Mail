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
starts the application once per window with `--scan-target`.

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

$process = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
if (-not $process) {
  Write-Error "No process $ProcessId. Nothing was walked, which is not the same as nothing being wrong."
  exit 2
}
$hwnd = $process.MainWindowHandle
if ($hwnd -eq [IntPtr]::Zero) {
  Write-Error "Process $ProcessId has no window yet. Nothing was walked."
  exit 2
}

$iid = [Msaa]::IID_IAccessible
$root = $null
$hr = [Msaa]::AccessibleObjectFromWindow($hwnd, [Msaa]::OBJID_CLIENT, [ref]$iid, [ref]$root)
if ($hr -ne 0 -or $null -eq $root) {
  Write-Error ("Could not get an IAccessible for the window (hr=0x{0:X}). Nothing was walked." -f $hr)
  exit 2
}

Walk $root 0 (Get-Name $root 0)

if ($found.Count -eq 0) {
  Write-Error "The walk reached no controls at all. A tree with nothing in it is a broken walk, not a clean one."
  exit 2
}

# Two properties are asked of each element, so this is the ceiling.
$asked = $found.Count * 2
if ($script:unreadable -ge $asked) {
  Write-Error ("Not one of {0} elements answered. The walk ran and read nothing, which is a broken walk and not a clean result." -f $found.Count)
  exit 2
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

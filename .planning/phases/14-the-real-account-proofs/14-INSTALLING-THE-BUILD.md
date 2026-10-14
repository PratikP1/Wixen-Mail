# Installing the build for the four sittings

**Replaced from 14 October.** The sittings now wait for a new build, made after the fixes
from the audit of 14 October and for the four problems you reported that day (phase 13.2), and
this page is written again for it before Sitting 2. Until then, what follows is the page for
the build of 6 October, which you have installed.

Every sitting of phase 14 runs on one build, made on this computer on
2026-10-06 from the merge of plan 14-07. This page installs it over the
build you have now. It takes about five to ten minutes. Nothing else runs
on this computer while you install.

## What you need

- The setup file, `Wixen-Mail-Setup-1.0.0-alpha.1+1118.gd3455792.exe`, in
  `C:\Users\prati\Documents\projects\Wixen-Mail\dist\`.
- About ten minutes, and your own consent at Windows' prompt.
- Your Google testing key only if it is ready. The install does not need it.

## Install it

1. If Wixen Mail is open, press Ctrl+Q to quit it. Setup stops and asks you to close it if it is still running.
2. Open File Explorer at `C:\Users\prati\Documents\projects\Wixen-Mail\dist\` and press Enter on `Wixen-Mail-Setup-1.0.0-alpha.1+1118.gd3455792.exe`. The folder also holds the older setup file, `Wixen-Mail-Setup-1.0.0-alpha.1+351.g5f363255.exe`; leave that one alone.
3. Windows asks whether to let the app make changes to your device. Choose Yes. It asks because your current copy is installed for all users, and setup keeps that choice without asking you again.
4. On the page "License Agreement", choose "I accept the agreement" (Alt+A), then Next (Alt+N).
5. On the page "Select Additional Tasks", leave the boxes as setup shows them, which is how you set them last time. Leave "Let Windows Search find my mail" unticked. Then Next.
6. On the page "Ready to Install", choose Install (Alt+I).
7. If setup says another copy of Wixen Mail is installed in a different place and asks whether to remove it, choose No and tell the agent. That question should not come, and if it does, the agent needs to look before anything is removed.
8. Wait while the page "Installing" runs.
9. On the last page, "Completing the Wixen Mail Setup Wizard", untick "Start Wixen Mail" with Space.
10. Choose Finish (Alt+F).

**Why Start Wixen Mail is unticked.** The agent reads your profile before
the new build's first start, at the beginning of the first sitting. The
first start also sends the two deletes that have waited since 20 September,
and you said to let them go; the first sitting is where they are read. Since
2026-10-06 the first sitting is Sitting 2, because your Google key comes
later; until then this paragraph said Sitting 1.

## Check what is installed

The build has two numbers, and they show in different places. Check them as
two separate steps.

1. Open Settings, then Apps, then Installed apps (Apps and Features on earlier Windows), and find Wixen Mail. Its version should read `1.0.0-alpha.1+1118.gd3455792` exactly, which is the same string Help, About shows once the program runs, and the first line of its log.
2. The other number, `1.0.0.14999`, belongs to the setup file alone. You see it only in that file's Properties, on the Details tab, under File version. It stops at 14999 because the build counter, 1118, is past the 999 that part of a Windows file version can hold; the version string above carries the true count. The installed `wixen-mail.exe` shows `1.0.0.0` in its own Properties, which is a known limit the changelog already states.

You do not need to check the second step. It is here so that a number in
Properties that differs from Installed apps does not look like a wrong
install.

## Your Google key, when it is ready

The steps for making your Google testing key and placing `oauth.toml` are in
Help, Setting up a provider, in the section "Calendars, contacts and tasks for
a Gmail account". So that you do not have to start Wixen Mail to read them,
the same page is installed as a file:
`C:\Program Files\Wixen Mail\docs\PROVIDER_SETUP.md`.

The key can wait. Sittings 2 and 3 do not need it; Sittings 1 and 4 do. So
the sittings run in the order 2, 3, 1, 4, and Sitting 1 starts once you say
the key is placed.

## When you are done

Send the agent one line:

- "installed, key placed", or
- "installed, key later", or
- what went wrong, in your words.

The agent then reads the installed version from the registry, without
starting anything, and checks that it is the string above.

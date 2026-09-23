# Installing Wixen Mail

Wixen Mail is a Windows application. Download `Wixen-Mail-Setup-<version>.exe` from the
[releases page](https://github.com/PratikP1/Wixen-Mail/releases) and run it.

## The warning you will see first

Wixen Mail is not yet code signed, so Windows does not recognise it. On a copy
you downloaded, SmartScreen shows a blue box saying **"Windows protected your
PC"**.

The Run button is not on that box. It is hidden behind a link, and the button
you can see cancels the install. To get past it:

1. Activate the **More info** link. It sits just below the message text, above
   the buttons.
2. A **Run anyway** button appears. Activate that.

With a screen reader, tab to **More info** and press Enter, then tab to **Run
anyway** and press Enter. Do not press the button you land on first: that is
**Don't run**.

This is not a fault in the download and it is not a virus warning. It means
Windows has not seen this particular file downloaded enough times yet to have
formed an opinion about it.

**Signing the setup file will not make this box go away**, and an earlier
version of this page said it would. Microsoft's own guidance is plain about it:
a signed application still gets the warning until enough people have downloaded
it, and Extended Validation certificates stopped being an exception to that in
2024. What signing does change is the name in the box. Instead of an unknown
publisher, you see who actually made it, which is the thing worth checking
before you press anything.

What makes the box stop appearing is time and downloads. Windows watches how
often a file is fetched and whether it behaves, and the warning fades once
enough clean installs have accumulated. For a program at this stage, expect to
keep seeing it.

## Choosing how to install

The first page of setup asks who the installation is for.

| Choice | Where it goes | Administrator rights |
|---|---|---|
| For me only | `%LOCALAPPDATA%\Programs\Wixen Mail` | Not needed |
| For all users | `C:\Program Files\Wixen Mail` | Needed, so Windows shows an elevation prompt |

Install for yourself unless you share the computer and want everyone to have it. That choice
needs no administrator rights, which means no elevation prompt and no switch to the secure
desktop part way through setup. It also works on a computer where you are not an
administrator.

The rest of setup is a licence page, a folder page, a checkbox for a desktop shortcut, and a
checkbox to start the application when setup finishes.

## Installing without the wizard

Setup accepts the standard Inno Setup switches.

```bash
Wixen-Mail-Setup-<version>.exe /SILENT /CURRENTUSER
```

`/SILENT` shows a progress window and nothing else. `/VERYSILENT` shows nothing at all.
`/CURRENTUSER` and `/ALLUSERS` answer the first page, so setup does not stop to ask.
`/DIR="D:\Wixen Mail"` chooses the folder. `/LOG="setup.log"` records what happened.

## Where your things are kept

Everything Wixen Mail stores about you is in one folder:

```text
%LOCALAPPDATA%\wixen-mail\
    config\           your settings, one file per account, and oauth.toml
    cache\            the mail that has been downloaded
    sound_schemes\    sound packs you have imported, if any
    logs\             the running log and crash.log
    logs\feedback\    a copy of each feedback report you send, if any
    updates\          an installer being downloaded, while one is
    security.key      only on a machine upgraded from an older version
```

Paste `%LOCALAPPDATA%\wixen-mail` into File Explorer's address bar to open it.

`updates\` exists only while Wixen Mail is fetching a new version, and it holds one file. Wixen
Mail empties it when the update is installed, when the installer is refused, and again the next
time it starts, so a download cut short does not leave an installer behind.

`oauth.toml` holds the sign-in keys this build was made with. It says nothing about you, and a
build made without one cannot offer the browser sign-in at all.

`security.key` is there only if this computer ran an older version of Wixen Mail. Nothing
creates it now. Older versions locked saved passwords in a file with it, and it is read once
so those passwords can be moved into the Windows credential store. A fresh install never has
one.

Your passwords and sign-in tokens are not in that folder. They are in the Windows credential
store, which is the same place Windows keeps its own saved sign-ins, protected per user by
Windows itself.

**The downloaded mail is not encrypted.** Windows stops other people who use the computer
from reading the folder, but anything running as you can read it, and so can anyone who takes
the drive out unless the disk itself is encrypted. Turn on BitLocker if that matters to you.
This is the same position as Outlook's offline folders and Thunderbird's local store, and it
is stated here rather than left to be discovered.

**To back up:** copy the whole `wixen-mail` folder. `config` is the part worth keeping.
`cache` is a copy of what is on the mail server and comes back on its own.

**To move to another computer:** copy the folder across, then add your password or sign in
again on the new machine. Credentials do not travel with the folder, by design.

### Keeping the folder somewhere else

Set the `WIXEN_MAIL_DATA` environment variable to a folder of your choosing and Wixen Mail
uses that instead. This is how to run from a memory stick, or to keep a large mail cache off
a small system drive.

```bash
setx WIXEN_MAIL_DATA "D:\Wixen Mail Data"
```

Sign out and back in, or restart the application, for the change to take effect. Move the
existing folder to the new place first if you want to keep what is in it.

## Updating

Run the new setup file. It installs over the old one and keeps your accounts, settings and
downloaded mail. There is no need to uninstall first.

### Letting Wixen Mail do it

Wixen Mail can fetch the new setup file for you. Under Settings, then General, then "New
versions", the setting "Tell me about new versions" has three answers, and it starts on "Do
not look for new versions". Choose "Released versions" or "Released versions and test
versions" and two things follow.

**The fetch is automatic. The install is not.** When a newer version is published, Wixen Mail
downloads the installer for it without asking you at that moment, which is what you agreed to
by choosing. Then it asks you once, and only about running it. Answering no leaves the version
you are running exactly as it was and deletes the file. Answering yes closes Wixen Mail and
opens the installer; you are told that is about to happen before it does.

Help, then Check for Updates, does the same thing on demand whatever that setting says, so you
never have to change a setting to look deliberately.

**Anything this project did not sign is refused, not warned about.** Before Wixen Mail will
offer to run a downloaded installer it checks two things: that the signature on the file is
valid, and that the name on that signature is this project's own. The second is the one that
matters, because a file can be perfectly validly signed and still be somebody else's program.
A file failing either check is deleted and you are told which check it failed. You are never
asked whether to run it anyway.

**Today that refuses everything.** Nothing this project publishes is signed yet, so an update
will download, be refused, and send you to the
[releases page](https://github.com/PratikP1/Wixen-Mail/releases) to fetch it by hand. That is
the intended behaviour rather than a fault: the refusal was built before the signing, so that
it could never be added afterwards to something already running installers.

Signing, when it arrives, changes the name in the SmartScreen box. It does not remove the box.
The section at the top of this page still applies.

## Uninstalling

Uninstall from Settings, then Apps, then Installed apps. Find Wixen Mail and choose
Uninstall.

**Uninstalling removes everything.** The program, your accounts, your settings, the
downloaded mail, and your saved passwords and sign-in tokens. It writes a note in your
temporary folder every time, `wixen-mail-uninstall.log`, saying what went and naming
anything it could not remove, so a leftover is something you are told about rather than
something you find. Two cases are the exception, and Checking what an uninstall did, below,
says which.

Your mail itself is not affected. It is still on your mail service's server, at Google or
wherever your account lives, and Wixen Mail only ever held a copy. Signing in from a new installation, or from any other mail application, brings
it all back.

If you are moving Wixen Mail to another drive rather than getting rid of it, copy
`%LOCALAPPDATA%\wixen-mail` somewhere safe before you uninstall.

### Checking what an uninstall did

Wixen Mail writes `wixen-mail-uninstall.log` to your temporary folder every time, whether
or not anything was left behind. Paste `%TEMP%` into File Explorer to find it, and read the
first line: it either says everything was removed, or names what was not. The file being
there is not itself a sign of trouble.

Two cases leave no note at all, because the note is written by the program and in both of
these the program never ran:

- The program was already gone from its folder before you uninstalled, so the step that
  clears your data was skipped. The uninstaller tells you this on screen when it happens.
- You closed the uninstaller before it finished.

Either way, check `%LOCALAPPDATA%\wixen-mail` yourself. If that folder is still there, the
downloaded mail in it is still there too, and it is not encrypted. Delete the folder to be
rid of it. Your saved passwords and sign-in tokens live in the Windows credential store,
not in that folder: open Credential Manager, choose Windows Credentials, and remove any
entry whose name begins with `wixen-mail`.

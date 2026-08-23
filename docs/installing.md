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
nobody has paid a certificate authority to vouch for the publisher yet, which
is being sorted out during testing. Once the setup file is signed, this box
stops appearing.

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
    config\           your settings and one file per account
    cache\            the mail that has been downloaded
    sound_schemes\    sound packs you have imported, if any
    logs\             the running log and crash.log
```

Paste `%LOCALAPPDATA%\wixen-mail` into File Explorer's address bar to open it.

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

## Uninstalling

Uninstall from Settings, then Apps, then Installed apps. Find Wixen Mail and choose
Uninstall.

**Uninstalling removes everything.** The program, your accounts, your settings, the
downloaded mail, and your saved passwords and sign-in tokens. It writes a note in your
temporary folder every time, `wixen-mail-uninstall.log`, saying what went and naming
anything it could not remove, so a leftover is something you are told about rather than
something you find.

Your mail itself is not affected. It is on your provider's server, and Wixen Mail only ever
held a copy. Signing in from a new installation, or from any other mail application, brings
it all back.

If you are moving Wixen Mail to another drive rather than getting rid of it, copy
`%LOCALAPPDATA%\wixen-mail` somewhere safe before you uninstall.

If something could not be removed, Wixen Mail writes the reason to
`wixen-mail-uninstall.log` in your temporary folder. Paste `%TEMP%` into File Explorer to
find it. That file existing at all means something was left behind, and it says what.

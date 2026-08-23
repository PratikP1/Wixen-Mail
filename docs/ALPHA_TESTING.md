# Testing Wixen Mail

Thank you for trying this. It is an alpha, which here means something specific:
large parts of it have never been run against a real mail account by anybody.
This page says which parts, so you can decide what to point it at.

## The short version

**Reading your mail is the part that has been used.** Signing in, listing
folders, fetching messages, reading them, searching, threading.

**Everything that writes is experimental.** Sending a message, deleting one,
moving or copying one to another folder, marking one read on the server, filing
a copy of what you send in Sent, changing which folders you are subscribed to,
and sending changes to your tasks, contacts or calendar back to Google or
Microsoft. None of that has run against a real account. Expect bugs.

Wixen Mail splits that answer in two, under a setting called Allow Changes.
Mail starts switched off: a message that has been sent cannot be recalled.
Changing your tasks, contacts and calendar starts switched on: those changes
go to your provider, and a task in the wrong place can be moved back. The
next section says what each answer covers and how to change it, but read
this first.

## What Wixen Mail is allowed to change

There are two separate permissions, because getting them wrong costs different
amounts.

| | What it covers | Default |
|---|---|---|
| Mail | Sending, deleting, moving, copying, marking read on the server, filing a copy in Sent, sending read receipts, changing subscriptions | **Off** |
| Tasks, contacts and calendar | Sending your changes back to your provider | On |

A message that has been sent cannot be recalled, and a message deleted from a
server may have been the only copy. A task in the wrong place can be moved
back. That is the whole reason they are separate.

Change them in Settings, under Allow Changes. The answer covers every account
you have signed in, so there is no way to leave one account read only and
allow everything on another. To use a real account with nothing at risk, start
Wixen Mail with `--read-only`, which is next.

### Signing in with a browser, and a limit worth knowing before you hit it

Choosing browser sign-in (OAuth) for Gmail during testing carries two limits
that are Google's, not this project's. Only people added by hand to the
project's tester list can sign in at all, and that list holds at most a
hundred people. Once you are on it, Google expires your sign-in after seven
days, so you sign in again through the browser about once a week. Neither
limit applies to an app password, which is why an app password is the steadier
choice if you would rather not repeat that every week. [Choosing a sign-in
method](PROVIDER_SETUP.md#choosing-a-sign-in-method) has the full detail, and
what to do if you signed in before this limit applied to you.

### Turning it off for one run

    wixen-mail --read-only

Changes nothing at any server for that run, whatever the settings say. Useful
if you want to look at something without any risk at all.

    wixen-mail --allow tasks

Allows tasks, contacts and calendar but not mail, for that run.

Neither of these can permit anything the settings forbid. They only ever take
permissions away, so leaving one in a shortcut is safe.

## What would help most to hear about

In rough order of how useful it is to know.

1. **Anything a screen reader gets wrong.** Something unlabelled, read in the
   wrong order, a focus that goes somewhere unexpected, an announcement that
   says something untrue. This is the whole reason the application exists, so
   these matter more than crashes.
2. **The message list and reading a message.** This is the most used path and
   the one most likely to meet mail shaped in a way nobody anticipated.
3. **Signing in.** Especially if your provider is not Gmail or Outlook.
4. **Anything that claims to have worked and did not.** A status line saying a
   message was sent when it was not, a count that does not match what you can
   see, a setting that does not take effect.
5. **Keyboard traps.** Anywhere Tab or Shift+Tab cannot get you out of.
6. **Whether the ticks are announced in Folders to Keep Up to Date.** Windows
   draws those check boxes itself instead of using a control that has them, so
   the state does not reach a screen reader on its own. Each row now reports
   itself as a check box with its state, which is the same fix NVDA makes in its
   own settings. Whether that works is a thing only a screen reader can answer,
   so it is worth a specific listen: arrow down the list and say whether you
   hear "ticked" and "not ticked" as well as the folder name, and whether it
   changes when you press Space.
7. **Whether the folder list matches what your provider shows you.** New in
   this version, and the part most likely to differ between providers. On Gmail
   in particular: whether your labels are all there, whether anything appears
   twice, and whether the count beside a folder matches the web interface.
8. **Deleting, moving and copying a message.** All three are new and all three
   change what is on the server. Say what you heard and what you then found on
   another device, because the two disagreeing is the failure worth catching.
9. **Whether a copy of what you send turns up in Sent.** Nothing was saving one
   before this version. Every account now files its own, Gmail included. On
   Gmail, Google also files one, and it should be the same copy rather than a
   second: if you see a message in Sent twice there, say so.
10. **Read receipts, if a sender asks for one.** Wixen Mail tells you a message
   asked and, by default, sends nothing. Worth reporting: whether you were told,
   whether anything was sent when it should not have been, and whether the
   setting under Reading does what it says.

## What is already known to be missing or unproven

Written down so you do not spend time reporting things already on the list.

- **Nothing that writes has run against a real account.** Sending, deleting,
  moving, copying, filing a copy in Sent, sending a read receipt, changing
  which folders you are subscribed to, and the three syncs that push changes.
- **A task cannot be moved between lists.** It goes into your provider's
  default list when you make it. Moving and copying work for mail only so far.
- **Importing a file can join two people who share a name.** Two cards in one
  imported file are read as one person when nothing but their addresses tells
  them apart. Two people with the same name, and nothing else on their cards,
  come out as one contact holding both addresses, and that joined contact is
  then sent to your real address book. Turning Allow Changes off before the
  next sync holds it back.
- **Conversations on Gmail are worked out from the message headers**, so a
  conversation here may be split differently from the same one in Gmail's web
  interface. Gmail publishes its own grouping and the library this is built on
  gives no way to read it.
- **A folder cannot be created, renamed or deleted**, and a whole folder cannot
  be marked read or emptied.
- **A POP account has never been run against a real POP server.** Everything
  about it is new in this version: the client, the local folders, the sync, and
  the policy that removes mail from the server. Mail is left on the server
  unless you turn that off, which is the setting to be careful with.
- **Notes do not sync anywhere.** They stay on this computer.
- **The cached mail on this computer is not encrypted.** Anybody who can read
  your user folder can read your mail. Passwords and tokens are not in there,
  they are in the Windows credential store.
- **The installer is not signed**, so Windows will warn about it.

## How to report something

Include what you did, what you expected, and what happened. If a screen reader
was involved, say which one and what it said.

Log files are in your Wixen Mail data folder, under `logs`. They do not contain
your passwords or the text of your messages. They may contain folder names and
email addresses, so read one before attaching it if that matters to you.

## Where your data is

Everything is in one folder, `%LOCALAPPDATA%\wixen-mail`: the cached mail, your
settings, and the logs. Paste that into File Explorer's address bar to open it.
Passwords and tokens are not in there, they are in the Windows credential store.

`--erase-all-data` removes all of it, including the saved passwords. The
uninstaller runs it for you. [Installing and uninstalling](installing.md) has the detail,
including what to copy if you want to keep your mail before uninstalling.

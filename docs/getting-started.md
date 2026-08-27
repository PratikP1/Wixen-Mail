# Getting started

This page is for somebody who has just installed Wixen Mail and wants to read
their mail. If you want to build it from source, see
[How to contribute](contributing.md).

## What works, and what does not

Reading your mail is the part that has been used. Everything that writes is
experimental: sending, moving, deleting, filing a copy in Sent, and sending your
changes to tasks, contacts and the calendar back to your provider. None of that
has been run against a real account yet, so expect bugs. A message that has been
sent cannot be recalled, and a message deleted from a server may have been the
only copy.

The first screen you see asks what Wixen Mail may change at your provider. The
answer it starts on allows nothing to your mail. You can change it later in the
Tools menu, under Settings, and the answer covers every account you have
signed in.

[What is worth testing, and what is known to be broken](ALPHA_TESTING.md) has
the fuller list.

## Add your account

Press `Ctrl+Shift+A`, or open the Tools menu and choose Account Manager. Fill in your
address and password, and the incoming and outgoing server details your provider
publishes.

If you use Gmail, Outlook.com, Yahoo or iCloud, the settings you need and the
app password each of them expects are in
[Setting up your provider](PROVIDER_SETUP.md). Most of them will not accept your
ordinary password.

Your password is kept in the credential store Windows already uses for saved
sign-ins, not in Wixen Mail's own files. The mail it downloads is not encrypted.
[What is stored, and what leaves this computer](privacy.md) says exactly what is
kept where.

## The six areas, and the key that reaches each

Wixen Mail holds six kinds of thing. Each has its own area, and each area has a
key.

| Area | Key |
| --- | --- |
| Mail | `Ctrl+Shift+1` |
| Contacts | `Ctrl+Shift+2` |
| Calendar | `Ctrl+Shift+3` |
| Reminders | `Ctrl+Shift+4` |
| Tasks | `Ctrl+Shift+5` |
| Notes | `Ctrl+Shift+6` |

`Ctrl+N` makes a new one of whatever the area you are in is for: a message in
Mail, a contact in Contacts, an appointment in Calendar, and so on.

## Finding your way around a mailbox

Mail is two lists: your folders, and the messages in the folder you are on.
`F6` moves to the next one and says which it arrived at, and `Shift+F6` moves
back. Arrow keys move within a list.

`Enter` on a message opens it in a window of its own, with the sender's headings
as headings and their links as links. `Space` on the message list reads the
message where you are instead, without opening anything: once for the summary,
again for the rest.

`Ctrl+F` searches.

## Getting help while you are working

`F1` opens the page for the area you are in. The Help menu lists every page,
including a full list of the keys.

The keys are worth reading early: this application is built to be worked by
keyboard, and there is a key for almost everything.
[Keyboard shortcuts](KEYBOARD_SHORTCUTS.md) has all of them, grouped by what
they do.

## When something goes wrong

[When something goes wrong](TROUBLESHOOTING.md) covers what to try first and how
to send a report that can be acted on.

## Where to go next

- [Using Wixen Mail](USER_GUIDE.md), for reading, writing, searching and
  attachments in detail
- [What is built for screen readers, and what is not done yet](accessibility.md)
- [What is built and what is not](IMPLEMENTATION_STATUS.md)
- [What changed, newest first](changelog.md)

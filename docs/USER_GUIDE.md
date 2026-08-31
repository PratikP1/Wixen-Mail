# Wixen Mail User Guide

## Table of Contents
1. [Getting Started](#getting-started)
2. [Account Setup](#account-setup)
3. [Email provider setup guides](PROVIDER_SETUP.md)
4. [Reading and Managing Email](#reading-and-managing-email)
5. [Composing Email](#composing-email)
6. [Search Functionality](#search-functionality)
7. [Thread View](#thread-view)
8. [Attachments](#attachments)
9. [Other modules: contacts, calendar, reminders, tasks, notes](#other-modules)
10. [Keyboard Shortcuts](#keyboard-shortcuts)
11. [Accessibility Features](#accessibility-features)
12. [Troubleshooting](#troubleshooting)

## Getting Started

Wixen Mail is designed to work with screen readers (NVDA, JAWS, Windows Narrator), and targets WCAG 2.2 Level AA. [Accessibility](accessibility.md) has the full detail, including what has and has not been confirmed with a real screen reader.

### System Requirements
- Windows 10 or later
- Internet connection for email access
- Optional: Screen reader (NVDA, JAWS, or Narrator) for accessibility features

### First Launch
When you first launch Wixen Mail, you'll need to configure an email account to get started.

## Account Setup

### Adding an account

1. Press `Ctrl+A`, or open the Tools menu and choose Account Manager.
2. Choose **Add Account**.
3. Type your email address. Wixen Mail recognises the domain of the
   popular providers and fills in the server settings, and turns the
   browser sign-in checkbox on or off, whichever usually works for that
   address. You can change either.
4. Depending on the provider and the checkbox, either sign in through the
   browser Wixen Mail opens, or enter your password or app password in the
   password box.
5. Type the name you want people to see when your mail arrives.
6. Choose **OK**.

[Setting up your provider](PROVIDER_SETUP.md) has the exact settings and
app-password steps for Gmail, Outlook.com and Office 365, Yahoo, iCloud, and
ProtonMail Bridge, and what to do for a provider not listed there.

### Managing accounts

The Account Manager (`Ctrl+A`) is also where you manage the accounts you
have already added:

- **Edit** changes an account's settings.
- **Delete** removes an account and its stored credentials.
- **Set Active** switches which account's mail you are looking at.
- **Sign In Again** re-authorises an account using browser sign-in, for
  when a token has been revoked or Google's weekly expiry has caught up
  with you.
- **Set as Default** chooses which account a new contact, event, task, or
  note is filed under when you make one from outside that account's own
  module.

`Ctrl+1` through `Ctrl+3` switch directly to your first, second, and third
enabled accounts.

### Other tools

The Tools menu also opens:

- **Message Filters**, rules that sort, mark, or move messages as they
  arrive. Each rule matches on a field such as subject, sender, or date,
  and can mark a message read, star it, move it, or tag it.
- **Contact Manager**, a dialog for the contacts stored for the account you
  are looking at. The [Contacts module](#other-modules) reached with
  `Ctrl+Shift+2` is the fuller way to work with contacts; this dialog
  overlaps it.
- **Signatures**, the text added to the end of messages you send.
- **Tags**, the labels you can put on a message.
- **Sync Contacts**, **Sync Calendar**, and **Sync Tasks**, to sync with
  your provider immediately rather than waiting for the next automatic
  sync.

### Offline Mode

**View → Offline Mode** switches to offline-first behaviour. While it is
on, sending a composed message queues it in a local outbox instead of
sending it immediately, one queue per account. **View → Flush Outbox**
attempts every queued send once you are back online.

## Reading and Managing Email

### Three-Pane Layout

Wixen Mail uses a classic three-pane layout:

```
┌─────────────┬─────────────────┬─────────────────┐
│  FOLDERS    │  MESSAGE LIST   │  PREVIEW PANE   │
│             │                 │                 │
│  Inbox      │  Subject        │  Message body   │
│  Sent       │  From           │  appears here   │
│  Drafts     │  Date           │                 │
│  Trash      │                 │  Attachments    │
│             │                 │  listed below   │
└─────────────┴─────────────────┴─────────────────┘
```

### Navigating Between Panes

- **Keyboard:** Press `F6` to cycle through panes
- **Mouse:** Click on the desired pane

### Message Status

Wixen Mail deliberately avoids icons for status that matters, since an icon
is something a screen reader user has to be taught to decode. Whether a
message is read or unread, starred, or has an attachment shows as a real
column in the message list, and reads as a word: "unread", "starred", "has
attachment". `Space` on a message reads its full status along with the rest
of the item, once for a short summary and again for everything.

### Message Actions

**Using Context Menu (Right-Click):**
1. Right-click on a message in the message list
2. Select an action:
   - **Reply** - Reply to the sender
   - **Forward** - Forward the message to someone else
   - **Delete** - Move to trash
   - **Toggle Star** - Add or remove star/flag
   - **Mark as Unread** - Mark message as unread

**Using Keyboard Shortcuts:**
- `Ctrl+R` - Reply
- `Ctrl+Shift+R` - Reply all
- `Ctrl+L` - Forward
- `Delete` - Delete message
- `S` - Star/flag message
- `Space` - Toggle read/unread

## Composing Email

### Creating a New Message

1. Click **File → New Message** or press `Ctrl+N`
2. Enter recipient(s) in the **To:** field
3. Optionally add CC and BCC recipients
4. Enter a subject
5. Type your message in the body field
6. Click **Send** or press `Ctrl+Enter`

### Saving Drafts

- Click **Save Draft** button or press `Ctrl+S`
- The draft will be saved to your Drafts folder
- You can return to edit it later
- Files you attached and any formatting are kept with it. If a file has been
  moved or deleted by the time you reopen the draft, it says which one

### Replying to Messages

1. Select a message in the message list
2. Press `Ctrl+R` or right-click and select **Reply**
3. The composition window opens with:
   - Recipient pre-filled
   - Subject pre-filled with "Re: [original subject]"
4. Type your reply and send

### Forwarding Messages

1. Select a message
2. Press `Ctrl+L` or right-click and select **Forward**
3. Enter recipient(s)
4. Add any additional comments
5. Send the message

## Search Functionality

### Opening Search

- Click **Edit → Search** or press `Ctrl+F`
- The search dialog will open

### Searching for Messages

1. Enter your search terms in the search field
2. Click **Search** button or press `Enter`
3. Results appear below the search field
4. Click on a result to view the message

### Search Tips

- Search matches against message subjects, senders, and the message preview
- Search is case-insensitive
- Search queries the mail already on this computer, so it works offline and
  needs no connection
- Use specific terms for better results

## Thread View

Related messages are grouped into a conversation using the `References` and
`In-Reply-To` headers rather than subject matching, so "Re: lunch" from two
strangers years apart is not folded into one conversation by mistake.

**Thread View**, `Ctrl+T` on the View menu, collapses the message list to one
row per conversation. Each row says what the conversation is about, how many
messages it holds and how many you have not read, and every other column
answers about the whole conversation rather than about its newest message.
Press `Ctrl+T` again to go back to one row per message.

A conversation row does not open out where it sits. The list stays flat, which
is what lets it tell your screen reader how many rows there really are, and a
list that grew branches when you pressed a key could not say that. Press
`Enter` on a conversation row to open the conversation window instead.

The Thread column, the one that says how many messages and how many unread,
appears in folders that hold a conversation of more than one message and stays
out of folders where every message stands alone. If you show or hide it
yourself in View, Columns, your choice wins from then on in that folder.

Each folder remembers its own view, and a folder you have never set is flat.
**Apply View To Other Folders**, on the same menu, gives your choice to the
folders under this one, to every folder in this account, or to every folder in
every account. It tells you which folders it will change and how many that is,
and asks first.

Switching the view keeps your selection and your sort. Switching to
conversations selects the conversations holding the messages you had selected;
switching back selects those messages again, not everything in their
conversations.

**Delete on a conversation row** asks first and names the number: "Delete 5
messages in Quarterly report?". `Enter` answers no. How far it reaches is a
setting on the Reading page: only the messages in the folder you are reading,
which is what it does unless you change it, or every message in the
conversation wherever it is filed.

**To read one conversation**, press `Enter` on it, or on any message that
belongs to one. That opens the conversation as a tree; `Enter` on its first row
opens the whole conversation as one document, with every message a real heading
you can move between with `H`, and `Enter` on any other row opens that one
message alone. `Esc` from the tree goes back to the message list, on the row
you came from. [Keyboard shortcuts](KEYBOARD_SHORTCUTS.md#conversations) has
the full detail.

## Attachments

### Viewing Attachments

A message with attachments is announced as having them, and Wixen Mail does
not use an icon for this: your screen reader hears it in words rather than
having to identify a glyph. Select the message to see the attachments listed
below the message body in the preview pane, or press `F8` from inside the
reader window to jump straight to the list.

### Attachment Information

Each attachment reads as its name, what kind of file it is in plain words,
and its size in a readable unit, for example "Report.pdf, PDF document,
240 KB", rather than as an icon. If an attachment is a program, something
Windows would run on opening it such as `.exe`, `.msi`, or `.ps1`, it is
named as a program rather than whatever the message claims it is, since the
type a message gives its own attachment is written by whoever sent it.

### Saving Attachments

1. Find the attachment in the preview pane
2. Click the **Save** button
3. Choose a location to save the file
4. The file will be downloaded

**Keyboard Shortcut:** Tab to the Save button and press `Enter`

## Other modules

Mail is one of six areas Wixen Mail holds in the same window, each with its
own key that reaches it from anywhere:

| Area | Key |
| --- | --- |
| Mail | `Ctrl+Shift+1` |
| Contacts | `Ctrl+Shift+2` |
| Calendar | `Ctrl+Shift+3` |
| Reminders | `Ctrl+Shift+4` |
| Tasks | `Ctrl+Shift+5` |
| Notes | `Ctrl+Shift+6` |

Every area shares the same shape: a sidebar, a list, and the same reading
pattern. `Space` reads the item under the cursor, once for a short summary
and again for the whole thing; `Ctrl+N` makes a new one of whatever the area
is for; `Delete` removes the one you are on, asking first and naming what it
will delete. [Keyboard shortcuts](KEYBOARD_SHORTCUTS.md) has every key for
every module in full.

## Keyboard Shortcuts

### Application Control
- `Ctrl+Q` - Quit application
- `Ctrl+,` - Open settings
- `F1` - Help documentation
- `Esc` - Close dialogs

### Window Navigation
- `F6` - Cycle through panes (folders → messages → preview)
- `Tab` - Navigate within pane
- `Arrow Keys` - Navigate lists
- `Enter` - Activate selected item

### Message Actions
- `Ctrl+N` - New message
- `Ctrl+R` - Reply
- `Ctrl+Shift+R` - Reply all
- `Ctrl+L` - Forward
- `Delete` - Delete message
- `S` - Star/flag message
- `Space` - Toggle read/unread

### Navigation
- `N` - Next unread message
- `P` - Previous unread message
- `Up/Down` - Navigate messages
- `Home/End` - First/last message

### Composition
- `Ctrl+Enter` - Send message
- `Ctrl+S` - Save draft

### Search & Mail
- `Ctrl+F` - Open search
- `F5` - Refresh folder
- `F9` - Check mail

## Accessibility Features

This is a short summary. [Accessibility](accessibility.md) is the complete
page, organised by who each part is for, including what has and has not
been confirmed with real assistive technology.

### Screen Reader Support

NVDA is the primary target, and the one a small automated suite drives for
real in CI. Windows Narrator is spot-checked. JAWS has not been run against
this application.

Wixen Mail announces a new message arriving, a change in what is selected, a
folder change with its unread count, search results, the outcome of an
action such as sending or deleting something, and errors with what to do
next.

### Keyboard Accessibility

Every function is meant to be reachable by keyboard: every button, every
menu, every dialog with `Tab` and `Shift+Tab`, and every context menu with
`Shift+F10` or the Menu key.

### Focus and Contrast

Every interactive control is meant to carry a visible focus indicator.
Settings offers light, dark, and a high contrast choice that hands the
colours back to Windows entirely, along with adjustable font size and
zoom with `Ctrl+Plus` and `Ctrl+Minus`.

## Sound Schemes and Earcons

Open Settings (`Ctrl+,`) and the Feedback tab to control the short sounds
Wixen Mail plays for events like new mail arriving or a message having an
attachment, alongside speech and the status line. The Sound scheme picker
chooses which set of sounds plays, starting with a built-in, synthesised
scheme. Choose **Import sound scheme** to bring in a `.zip` someone else
packaged; choose **Delete sound scheme** to remove one you no longer want.
Delete stays disabled while only the built-in scheme is present, since one
scheme must always exist. [Accessibility](accessibility.md#hearing-and-non-speech-audio)
has the fuller explanation of what earcons are for.

## Troubleshooting

### Connection Issues

**Problem:** Cannot connect to email server

**Solutions:**
1. Check your internet connection
2. Verify server address and port are correct
3. Ensure TLS/SSL settings match your provider's requirements
4. Check if firewall is blocking the connection
5. Try disabling antivirus temporarily to test

### Authentication Issues

**Problem:** Username or password not accepted

**Solutions:**
1. Verify your username is correct (usually your full email address)
2. Check password is correct (case-sensitive)
3. For Gmail/Yahoo/iCloud: Use an **app password**, not your regular password
4. Ensure 2FA is properly configured
5. Check if IMAP/SMTP is enabled for your account
6. Contact your email provider if issues persist

### Missing Folders

**Problem:** Folders not appearing after connection

**Solutions:**
1. Click **View → Refresh** or press `F5`
2. Try disconnecting and reconnecting
3. Check if folders exist in webmail interface
4. Some providers may use different folder names

### Messages Not Loading

**Problem:** Message list is empty

**Solutions:**
1. Verify folder is selected in the folder pane
2. Check if folder actually contains messages
3. Try refreshing the folder (F5)
4. Check error messages for connection issues

### Slow Performance

**Problem:** Application is slow or unresponsive

**Solutions:**
1. Large message lists may take time to load
2. Consider archiving old messages
3. Close other resource-intensive applications
4. Restart Wixen Mail
5. Check system resources (RAM, CPU)

### Attachment Issues

**Problem:** Cannot save attachments

**Solutions:**
1. Ensure you have write permissions to the save location
2. Check available disk space
3. Try a different save location
4. Verify the attachment downloaded properly

### Screen Reader Issues

**Problem:** Screen reader not announcing changes

**Solutions:**
1. Ensure screen reader is running before starting Wixen Mail
2. Try restarting both the screen reader and Wixen Mail
3. Check screen reader verbosity settings
4. Update to latest version of screen reader

## Getting Help

### In-App Help

- Press `F1` to open documentation
- Click **Help → Documentation** in the menu bar
- View keyboard shortcuts: **Help → Keyboard Shortcuts**

### Provider-Specific Help

When you need an app password, the Add Account dialog says so next to the
password box, and the account dialog points you to
[Setting up your provider](PROVIDER_SETUP.md), which has the full steps for
Gmail, Outlook.com and Office 365, Yahoo, iCloud, and ProtonMail Bridge.

### Report Issues

If you encounter issues not covered in this guide:
1. Check the application logs for details
2. Note any error messages you receive
3. Report issues on the GitHub repository

## Tips for Best Experience

1. **Use app passwords** for providers that support them (Gmail, Yahoo, iCloud)
2. **Press `Enter` on a message in a conversation** to read the whole thread as one document
3. **Use keyboard shortcuts** for faster navigation
4. **Star important messages** for quick access later
5. **Use search** to quickly find messages
6. **Right-click for quick actions** on messages
7. **Keep folders organized** by archiving old messages
8. **Check for updates** regularly for new features and fixes

## Conclusion

Learning the keyboard shortcuts pays off quickly, since almost everything in
Wixen Mail is reachable that way. Folders, stars, and tags keep a growing
mailbox organised, and search finds a specific message faster than scrolling
to it.

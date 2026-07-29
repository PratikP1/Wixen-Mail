# Wixen Mail Keyboard Shortcuts

Complete reference of all keyboard shortcuts in Wixen Mail for efficient, accessible email management.

## Quick Reference Card

### Most Common Shortcuts
| Action | Shortcut |
|--------|----------|
| New, whatever the area is for | `Ctrl+N` |
| New Message | `Ctrl+Shift+M` |
| Reply | `Ctrl+R` |
| Reply All | `Ctrl+Shift+R` |
| Reply to Sender Only | `Alt+Shift+R` |
| Forward | `Ctrl+L` |
| Search | `Ctrl+F` |
| Delete Message | `Delete` |
| Send Message | `Ctrl+Enter` |
| Quit Application | `Ctrl+Q` |

## All Shortcuts by Category

### Module Navigation

Wixen Mail holds six modules in one window. These shortcuts switch between them
from anywhere in the main window. The module name is announced on each switch,
and focus moves to that module's content area.

| Action | Shortcut | Description |
|--------|----------|-------------|
| Mail | `Ctrl+Shift+1` | Switch to the mail module |
| Contacts | `Ctrl+Shift+2` | Switch to the contacts module |
| Calendar | `Ctrl+Shift+3` | Switch to the calendar module |
| Reminders | `Ctrl+Shift+4` | Switch to the reminders module |
| Tasks | `Ctrl+Shift+5` | Switch to the tasks module |
| Notes | `Ctrl+Shift+6` | Switch to the notes module |

### Reading the Item Under the Cursor

The same two keys work in mail, contacts, calendar, reminders, tasks, and
notes. A list row is read as its visible columns and nothing else, so
everything else the record holds is invisible until you open it. These keys
answer that without leaving the list.

| Action | Shortcut | Description |
|--------|----------|-------------|
| Read the short form | `Space` | Subject, sender, and snippet in mail; the equivalent line in every other module |
| Read the whole item | `Space` again | Adds recipients, dates, flags, attachments, and any description the record holds |
| Read the whole item outright | `Shift+Space` | The full reading without counting presses |

Pressing `Space` a third time goes back to the short form. Moving to another
row starts again at the short form, so the same key always gives the same
answer for a row you have just arrived at.

There is no double-press timing window. The second press does the second thing
however long you took, because a timing window is a timing trap (WCAG 2.2.1)
and locks out anyone who types slowly.

`Ctrl+M` mutes this reading without silencing status and error
announcements.

### Working on the Item You Are On

Every module could make things and none of them could remove one. These work in
Contacts, Calendar, Reminders, Tasks and Notes, on the row the list is sitting
on.

| Action | Shortcut | Description |
|--------|----------|-------------|
| Delete | `Delete` | Asks first, naming what it will delete. In Mail this deletes the message instead |
| Mark done or not done | `Ctrl+Shift+K` | Tasks and Reminders. Says which way it went |
| Pin or unpin | `Ctrl+Shift+P` | Notes. Pinned notes sort to the top |

`Delete` is one key that acts on whatever is in front of you, the same way
`Ctrl+N` makes whatever the area you are in is for. In Mail it is the message
delete, with the server behind it. Everywhere else it is the row you are on.

**Deleting always asks, and the question names the row.** "Delete \"File the tax
return\"? This cannot be undone." Somebody who arrowed onto the wrong row finds
out from the question, which only works because the question says which row.

The two toggles say which way they went: "Buy milk, done" or "Buy milk, not
done". A toggle you cannot see is a toggle you have to be told about. They are
greyed out in modules where they mean nothing, so your screen reader says
"unavailable" rather than leaving you to press a key that does nothing.

### The Reader Window

`Enter` on a message opens it in the reader. `Enter` on a message that belongs
to a conversation opens the conversation tree first; both choices from there
open into the same reader window.

The reader is a read-only text control, not a browser, so everything you expect
from a text control works: arrow keys move by character, word and line, `Home`
and `End` work, text can be selected and copied, and your screen reader reports
where the caret is.

| Action | Shortcut | Description |
|--------|----------|-------------|
| Close the window | `Esc` | Back to the message list |
| Close this tab | `Ctrl+W` | Closes the window when the last tab goes |
| Next tab | `Ctrl+Tab` | Standard notebook navigation |
| Previous tab | `Ctrl+Shift+Tab` | |
| Next message in the conversation | `Ctrl+Down` | Announces the message it lands on, and says "Last message" at the end |
| Previous message in the conversation | `Ctrl+Up` | |
| Security warning | `F7` | Moves between the message and the warning above it, when there is one |
| Attachments | `F8` | Moves between the message and the list of attachments, when there is one |
| Read an attachment | `Ctrl+O` | Opens a PDF as a tab of its own. `Enter` on a row does the same |
| Save an attachment | `Ctrl+S` | Saves the attachment the list is on, to a file |

#### Attachments

A message with attachments gets a list of them below the message text, so it is
the next thing after the message in the tab order. Each row reads as the name,
what kind of file it is in plain words, and how big it is: "Report.pdf, PDF
document, 240 KB".

`F8` jumps to the list from anywhere in the message, and `F8` again goes back to
the message.

`Ctrl+S` opens the standard Save dialog with the name already filled in. The
file is downloaded when you save it rather than kept on your computer in
advance, so saving a large attachment takes as long as downloading it does. The
status line says when it is saved and where it went.

`Enter` on a row, or `Ctrl+O`, reads the attachment here, in a tab of its own.
That works for PDFs. Anything else says so and names `Ctrl+S` instead, rather
than opening a screenful of nonsense or doing nothing.

#### Reading a PDF

A PDF opens as another tab in the reader, so everything that works on a message
works on it: arrow keys, `Ctrl+F`, selection and copy, `Ctrl+Down` and `Ctrl+Up`
to move between pages and headings. Each page starts with a line saying which
page it is, so a long document can be moved through a page at a time.

**The first thing the tab says is where its structure came from.** That matters
more than it sounds:

- A tagged PDF has headings its author marked, and they can be trusted.
- A PDF with incomplete tagging has some structure declared and some worked out.
- An untagged PDF has none, so any headings were guessed from the size and
  position of the text. They are usually right and they are still a guess.
- A PDF with no text at all is a scan: a picture of a page rather than words.
  Nothing can read it aloud, and the tab says so plainly instead of opening
  empty and leaving you to work out why.

Most applications do not draw that distinction, which is how a scanned document
comes to be handed over as though it were readable.

Very long documents stop after 200 pages, and the note says so. Save the file to
read the rest.

**A file Windows would run is called a program.** If an attachment ends in
`.exe`, `.msi`, `.scr`, `.bat`, `.ps1`, `.lnk` or anything else Windows executes,
the row says "program" rather than whatever the message claimed the file was.
The opening announcement says so too, before you have reached the list. The type
a message gives its own attachment is written by whoever sent it, so it is a
claim rather than a fact, and on a malicious attachment the claim is usually the
harmless one.

Messages with nothing attached have no list at all, so there is nothing extra to
tab past on ordinary mail.

#### Reading a conversation as headings

The reader is a text control, which has no headings, so `H` does nothing in it.
`Ctrl+Down` and `Ctrl+Up` move between the messages of a conversation instead,
and they always have.

When a thread is long enough that one key at a time is the slow way round, the
conversation dialog has an **As Headings** button. It opens the whole
conversation as a page in a window of its own, where every message is a real
heading and `H` moves between them at the level the reply sits at.

That window shows the conversation and nothing else, which is what makes it safe
to use a browser control here at all. What used to trap people was the preview
pane: a browser sharing a window with a folder tree and a message list, where
`F6` has to cycle panes and `Escape` has to return to the list, and the browser
swallowed both. Here there is nowhere else to go, so closing the window is the
way out and there is nothing to escape from.

The text reader stays the default and stays where it was. This is a second way
of reading the same thread, not a replacement.

#### The security warning

When your mail provider's filter marked a message as spam, or when the message
looks like a phishing attempt, the reader puts a warning above it. The warning
is a read-only text box, so you can read it as many times as you like, move
through it with the arrow keys, and copy it.

It comes before the message in the tab order, so `Shift+Tab` from the message
reaches it, and `F7` jumps to it from anywhere in the text. `F7` again goes
back to where you were reading.

The warning is also announced when the message opens. That announcement is an
ordinary feedback event, so you can turn it off, change which channels it uses,
or make it a tone instead of speech, in Settings. Turning the announcement off
does not remove the warning: the bar is still there and `F7` still reaches it.

Messages with nothing wrong with them have no warning bar at all, so there is
nothing extra to tab past on ordinary mail.

### The Preview Pane

The preview is a visual pane. It never takes focus, and `F6` does not stop
there.

That is deliberate. The preview is a WebView, which hosts a browser: once focus
is inside it, the browser consumes `Esc`, `F6` and every menu accelerator, and
when it holds its host window rather than the page, those keys reach nothing at
all. There is no way for this application to intercept them first, so the only
reliable answer is to keep focus out.

To read a message, use `Space` on the message list: once for the summary, again
for the whole message, or `Shift+Space` for the whole message outright. That
path works with the screen reader you already have configured, and you never
leave the list.

A readable, focusable text view of the message body is the proper long-term
answer and is not built yet.

### Conversations

| Where | Key | Result |
|-------|-----|--------|
| A message with no conversation | `Enter` | Opens that message in the preview. No tree in the way. |
| A message in a conversation | `Enter` | Opens the conversation tree |
| Conversation tree, first row | `Enter` | The whole conversation in one document, every message in order |
| Conversation tree, any message | `Enter` | That message alone |
| Conversation tree | `Esc` | Back to the list, focus on the row it came from |

The message list stays one row per message. The conversation structure is
behind `Enter` rather than in the list itself, so arrowing never walks branches
you did not ask for.

The first row of the tree is labelled "Whole conversation, 5 messages" rather
than the subject, because `Enter` does two different things in that tree and
the row has to say which one it will do.

In the combined document every message is numbered and `Ctrl+Down` and
`Ctrl+Up` move between them, announcing the one they land on.

`H` does not work here yet. The reader is a plain text control, which has no
headings for a screen reader to find, so the message boundaries exist as
positions this application jumps to rather than as structure your screen reader
can navigate. Making them real headings is being worked on.

### Speech Control

| Action | Shortcut | Description |
|--------|----------|-------------|
| Mute Message Reading | `Ctrl+M` | Stop reading message text aloud. Status and error announcements keep working, so muting before a screen share does not cost you your error messages. |

### Application Control

| Action | Shortcut | Description |
|--------|----------|-------------|
| Quit Application | `Ctrl+Q` | Exit Wixen Mail |
| Open Settings | `Ctrl+,` | Open settings dialog |
| Open Help | `F1` | Show help documentation |
| Close Dialog | `Esc` | Close the current dialog or window |

### Window and Pane Navigation

| Action | Shortcut | Description |
|--------|----------|-------------|
| Next Pane | `F6` | Move focus between the folder tree and the message list |
| Navigate Forward | `Tab` | Move to next element in current pane |
| Navigate Backward | `Shift+Tab` | Move to previous element in current pane |
| Navigate List | `↑` `↓` | Move up/down in lists |
| First Item | `Home` | Jump to first item in list |
| Last Item | `End` | Jump to last item in list |
| Activate Item | `Enter` | Activate selected item (open folder, select message) |

### File Menu

`Ctrl+N` makes the thing the area you are in is for: a message in Mail, a
contact in Contacts, an event in Calendar, and so on. The six `Ctrl+Shift`
keys each make one particular kind from anywhere, so you never have to switch
module first.

New items go to your default account when that account can hold them, and to
this computer when it cannot. Anything kept here still appears in the panels
alongside your account's own items, so a note you make is where you expect it. A plain mail account, which is every POP account
and most IMAP ones, holds mail and nothing else, so contacts and events made
while it is the default are kept here rather than pretending to sync. Tasks,
notes and reminders are kept here for everybody. Wixen Mail says where each new
item went as it makes it.

Your first account becomes the default on its own. Change it in the accounts
dialog once you have more than one.

| Action | Shortcut | Description |
|--------|----------|-------------|
| New | `Ctrl+N` | Makes what the area you are in is for |
| New > Message | `Ctrl+Shift+M` | Compose a message |
| New > Contact | `Ctrl+Shift+C` | |
| New > Event | `Ctrl+Shift+E` | |
| New > Reminder | `Ctrl+Shift+D` | `D` for due. `Ctrl+Shift+R` is Reply All, here and in every other mail client |
| New > Task | `Ctrl+Shift+T` | |
| New > Note | `Ctrl+Shift+N` | |
| New > Account | (none) | Open Account Manager |
| Open Draft | `Ctrl+D` | Reopen a message you saved to finish later |
| Save | `Ctrl+S` | Save current draft |
| Save As | (none) | Save message or attachment to file |
| Check Mail | `F9` | Check for new messages |
| Get Older Messages | `Ctrl+Shift+G` | Bring down the next page of older mail in the folder you are in |
| Quit | `Ctrl+Q` | Exit the application |

### Edit Menu

| Action | Shortcut | Description |
|--------|----------|-------------|
| Search | `Ctrl+F` | Open search dialog |

### Account Management

| Action | Shortcut | Description |
|--------|----------|-------------|
| Open Account Manager | `Ctrl+A` | Open multiple account management dialog |

### Contact Management

| Action | Shortcut | Description |
|--------|----------|-------------|
| Open Contact Manager | (Tools menu) | Open contact / address book manager |

### Rules Management

| Action | Shortcut | Description |
|--------|----------|-------------|
| Open Rules Manager | (menu only) | Open message filter rules manager |

### View Menu

| Action | Shortcut | Description |
|--------|----------|-------------|
| Folder Pane | `Alt+1` | Show or hide the folder pane |
| Preview Pane | `Alt+2` | Show or hide the message preview |
| Module Buttons | `Alt+3` | Show or hide the module navigation buttons |
| Columns | `F8` | Choose which message list columns are shown and in what order |
| Refresh Folder | `F5` | Read the current folder again from the local store |
| Next Pane | `F6` | Move focus between the folder tree and the message list |
| Thread View Toggle | `Ctrl+T` | Not available yet. It would collapse the list to one row per conversation. To read a conversation now, press `Enter` on a message that belongs to one. |
| Check Mail | `F9` | Check for new messages |

#### Inside the Columns dialog

A bare `F8` on purpose: choosing columns is a verbosity control, something you
reach for often when you navigate a list by ear, and it should not cost a
three-finger stretch.

| Action | Shortcut | Description |
|--------|----------|-------------|
| Show or hide a column | `Space` | Toggles the highlighted column. The last remaining column cannot be hidden, and the refusal is announced. |
| Move a column up | `Alt+Up` | Also the Move Up button |
| Move a column down | `Alt+Down` | Also the Move Down button |
| Restore the defaults | `Alt+R` | Puts back the default columns for this kind of folder |

Every column reads its own position, for example "Subject, shown, position 3 of
6", because moving something in a list you cannot see is otherwise a silent
action with an invisible result.

#### Sorting

Clicking a column header sorts by it; clicking the same header again reverses
the order. Dates start at newest first, text at A to Z. The keyboard path is
View, Sort Messages, which uses radio items so a screen reader announces which
sort is currently selected. The two are kept in step: sorting from a header
moves the menu, and sorting from the menu moves the headers.

### Message Actions

| Action | Shortcut | Description |
|--------|----------|-------------|
| Reply | `Ctrl+R` | Reply where the sender asked. On a mailing list this is the list. |
| Reply All | `Ctrl+Shift+R` | Reply to everyone the message reached, leaving out your own address |
| Reply to Sender Only | `Alt+Shift+R` | Reply only to the person who wrote it, never to the list |
| Forward | `Ctrl+L` | Forward selected message |
| Delete | `Delete` | Move selected message to trash |
| Flag Message | `Ctrl+Shift+S` | Flag or unflag the selected message |
| Mark as Read | (Message menu) | Mark the selected message as read. This is not on `Space`: `Space` reads the item aloud, in every module. |

### Message Navigation

| Action | Shortcut | Description |
|--------|----------|-------------|
| Next Unread | `Ctrl+]` | Next unread message, wrapping at the end. Says so when there is none, rather than doing nothing. |
| Previous Unread | `Ctrl+[` | Previous unread message, wrapping at the start |
| Next Message | `↓` | Move to next message in list |
| Previous Message | `↑` | Move to previous message in list |
| First Message | `Home` | Jump to first message |
| Last Message | `End` | Jump to last message |

### Composition Window

**Keyboard shortcuts:**

| Action | Shortcut | Description |
|--------|----------|-------------|
| Send Message | `Ctrl+Enter` | Send the current message |
| Save Draft | `Ctrl+S` | Save message as draft |
| Close Window | `Esc` | Close composition window without sending |

**Formatting.** Every one of these is also on the Format menu, which the Format
button opens, so none of them has to be memorised to be used. Each says what it
applied when you use it.

| Action | Shortcut | Description |
|--------|----------|-------------|
| Bold | `Ctrl+B` | Bold the selection |
| Italic | `Ctrl+I` | Italicise the selection |
| Underline | `Ctrl+U` | Underline the selection |
| Heading 1 | `Ctrl+Alt+1` | Make the current line a top-level heading |
| Heading 2 | `Ctrl+Alt+2` | Make the current line a second-level heading |
| Heading 3 | `Ctrl+Alt+3` | Make the current line a third-level heading |
| Normal Text | `Ctrl+Alt+0` | Turn a heading or quote back into an ordinary paragraph |
| Bulleted List | `Ctrl+Shift+L` | Start or end a bulleted list |
| Numbered List | `Ctrl+Shift+O` | Start or end a numbered list |
| Quote | `Ctrl+Shift+Q` | Indent the current line as a quotation |
| Remove Formatting | `Ctrl+Space` | Strip formatting from the selection |
| Undo | `Ctrl+Z` | Undo last edit |
| Redo | `Ctrl+Y` | Redo last undo |

Headings and lists are worth using. They are the structure the person receiving
your message navigates by, and a long message without them can only be read
straight through.

Two limitations, stated rather than hidden:

- The heading keys use `Ctrl+Alt`, which the keyboard sends as AltGr on many
  non-US layouts. Where AltGr and a digit types a character, that character
  still gets typed and the heading is not applied. Use the Format menu on those
  layouts. Taking the character away would be the worse trade.
- These keys work while the caret is in the message body. In the To, Cc, Bcc and
  Subject fields there is nothing to format, so they do nothing there.

**Button accelerators (Alt+key):**

| Button | Shortcut |
|--------|----------|
| Send | `Alt+N` |
| Undo | `Alt+U` |
| Redo | `Alt+R` |
| Attach File | `Alt+I` |

**Field accelerators (Alt+key):**

| Field / Action | Shortcut |
|----------------|----------|
| From | `Alt+F` |
| To | `Alt+T` |
| CC | `Alt+C` |
| BCC | `Alt+B` |
| Subject | `Alt+S` |
| Save Draft | `Alt+D` |
| Discard | `Alt+A` |
| Cancel | `Alt+L` |

### Folder Actions

| Action | Shortcut | Description |
|--------|----------|-------------|
| Select Folder | `↑` `↓` | Navigate folder list |
| Open Folder | `Enter` | Load messages from selected folder |
| Refresh Folder | `F5` | Reload current folder |

### Contact Manager Dialog Accelerators

| Action | Shortcut | Description |
|--------|----------|-------------|
| Search contacts | `Alt+S` | Focus the search field (live search as you type) |
| Add contact | `Alt+A` | Open Add Contact dialog |
| Edit contact | `Alt+E` | Edit selected contact |
| Delete contact | `Alt+D` | Delete selected contact |
| Close | `Alt+C` | Close Contact Manager |

### Contact Edit Dialog Accelerators

**Basic Info tab:**

| Field | Shortcut |
|-------|----------|
| Name | `Alt+N` |
| Nickname | `Alt+K` |
| Company | `Alt+C` |
| Department | `Alt+D` |
| Job Title | `Alt+J` |
| Birthday | `Alt+B` |
| Website | `Alt+W` |
| Relationship | `Alt+R` |
| Avatar URL | `Alt+A` |
| Favorite | `Alt+F` |

**Email & Phone tab:**

| Action | Shortcut |
|--------|----------|
| Add Email | `Alt+A` |
| Remove Email | `Alt+R` |
| Add Phone | `Alt+P` |
| Remove Phone | `Alt+V` |

**Addresses tab:**

| Action | Shortcut |
|--------|----------|
| Add Address | `Alt+A` |
| Remove Address | `Alt+R` |

**Notes & Custom tab:**

| Action | Shortcut |
|--------|----------|
| Notes | `Alt+N` |
| Add Field | `Alt+A` |
| Remove Field | `Alt+R` |

### Address Sub-Dialog Accelerators

| Field | Shortcut | Notes |
|-------|----------|-------|
| Country | `Alt+C` | Dropdown; country selection drives region/code labels |
| Type | `Alt+T` | Home/Work/Other dropdown |
| Street | `Alt+S` | |
| City | `Alt+I` | (C conflicts with Country) |
| State/Province | varies | Label changes per country (e.g., State, Province, County) |
| ZIP/Postal Code | varies | Label changes per country (e.g., ZIP, Postcode, PLZ) |

### Context Menu

| Action | Shortcut | Description |
|--------|----------|-------------|
| Open Context Menu | `Shift+F10` or `Menu` | Open context menu for selected item |
| Navigate Menu | `↑` `↓` | Move through menu items |
| Select Menu Item | `Enter` | Activate highlighted menu item |
| Close Menu | `Esc` | Close context menu |

### Search Dialog

| Action | Shortcut | Description |
|--------|----------|-------------|
| Open Search | `Ctrl+F` | Open search dialog |
| Execute Search | `Enter` | Perform search |
| Close Search | `Esc` | Close search dialog |
| Navigate Results | `↑` `↓` | Move through search results |
| View Result | `Enter` | Open selected search result |

### Attachment Actions

| Action | Shortcut | Description |
|--------|----------|-------------|
| Save Attachment | (focus button with `Tab`, press `Enter`) | Save attachment to disk |
| Navigate Attachments | `Tab` | Move between attachments in preview pane |

### Dialog Navigation

OK and Cancel buttons carry no `Alt` mnemonic anywhere in the application. They
use the standard dialog identifiers, so `Enter` activates OK and `Esc` activates
Cancel from anywhere in the dialog. Adding mnemonics as well would compete with
the field accelerators for letters without making anything more reachable.

| Action | Shortcut | Description |
|--------|----------|-------------|
| Next Control | `Tab` | Move to next control in dialog |
| Previous Control | `Shift+Tab` | Move to previous control in dialog |
| Activate Button | `Enter` or `Space` | Click focused button |
| Cancel Dialog | `Esc` | Close dialog without saving |

## Screen Reader Specific

### NVDA Shortcuts

| Action | Shortcut | Description |
|--------|----------|-------------|
| Read Current Item | `Insert+Tab` | Read currently focused item |
| Read Window | `Insert+B` | Read entire current window |
| Read to End | `Insert+↓` | Read from cursor to end |
| Say All | `Insert+↓` (hold) | Read entire document |

### JAWS Shortcuts

| Action | Shortcut | Description |
|--------|----------|-------------|
| Read Current Line | `Insert+↑` | Read line with cursor |
| Read Current Word | `Insert+Numpad5` | Read current word |
| Say All | `Insert+↓` | Read entire document |

### Windows Narrator Shortcuts

| Action | Shortcut | Description |
|--------|----------|-------------|
| Read Item | `Caps Lock+Tab` | Read current item |
| Read Window | `Caps Lock+W` | Read current window |
| Continuous Reading | `Caps Lock+R` | Start/stop continuous reading |

## Tips for Keyboard Navigation

### Efficient Workflow

1. **Use `F6` to move between the folder tree and the message list** - Much faster than using the mouse
2. **Learn the message action shortcuts** - `Ctrl+R`, `Ctrl+L`, `Delete` are the most common
3. **Press `Space` on any row** to hear the item, and again to hear all of it. It works the same in every module.
4. **Use `Ctrl+]` to jump to unread messages** - Quickly find messages that need attention. It wraps at the end and tells you when there are none left.
5. **Master the composition shortcuts** - `Ctrl+Enter` to send, `Esc` to cancel
6. **Context menus are your friend** - `Shift+F10` opens context menu for selected item

### Power User Tips

- **Combine shortcuts** - Use `↓` to select a message, then `Ctrl+R` to reply
- **Use search frequently** - `Ctrl+F` is faster than scrolling through long message lists
- **Flag important messages with `Ctrl+Shift+S`** - Makes them easier to find later
- **`F5` refreshes the current view** - Use when waiting for new mail
- **`Esc` backs out of anything** - Universal cancel/close shortcut

### Accessibility Best Practices

1. **Always use `Tab` and `Shift+Tab`** to navigate within dialogs
2. **Listen for screen reader announcements** after actions
3. **Use `F6` instead of mouse** to switch between main panes
4. **Context menus provide more actions** - Don't rely only on toolbar buttons
5. **Arrow keys work in all lists** - Folders, messages, search results

## Customization

Currently, keyboard shortcuts are fixed. Future versions may allow customization through Settings.

## Platform-Specific Notes

### Windows
- `Ctrl` key is used for all shortcuts
- `Alt` key activates menu bar (Alt+F for File menu, etc.)
- `Windows+H` can activate dictation in text fields (OS feature)

### Screen Reader Compatibility

All shortcuts are designed to work alongside screen reader commands:
- Wixen Mail shortcuts don't conflict with common NVDA/JAWS shortcuts
- Screen reader navigation mode works in preview pane
- Form mode automatically activates in text fields

## Accessibility Notes

- All shortcuts are keyboard-only (no mouse required)
- Shortcuts announced by screen readers when available
- Visual focus indicators show which element is active
- Shortcuts work in all accessibility modes

## Quick Start Shortcuts

If you're new to Wixen Mail, start with these essential shortcuts:

1. `Ctrl+N` - Compose new message
2. `F6` - Switch between panes
3. `↑`/`↓` - Navigate lists
4. `Enter` - Select item
5. `Ctrl+R` - Reply to message
6. `Delete` - Delete message
7. `Ctrl+F` - Search
8. `Ctrl+Q` - Quit

Master these, and you'll be navigating Wixen Mail efficiently!

## Help and Support

- Press `F1` in the app for context-sensitive help
- See the User Guide for detailed feature documentation
- All shortcuts are displayed in menu items where applicable

## Printable Quick Reference

```
ESSENTIAL SHORTCUTS
─────────────────────────────────────
Ctrl+N          New Message
Ctrl+R          Reply
Ctrl+Shift+R    Reply All
Alt+Shift+R     Reply to Sender Only
Ctrl+L          Forward  
Ctrl+F          Search
Delete          Delete Message
Ctrl+Enter      Send Message
F6              Switch Panes
↑/↓            Navigate Lists
Enter           Select Item
Esc             Close Dialog
Ctrl+Q          Quit
─────────────────────────────────────
```

Print this page or save it for quick reference while learning Wixen Mail!

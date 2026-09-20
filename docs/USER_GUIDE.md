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
9. [Import and Export](#import-and-export)
10. [Other modules: contacts, calendar, reminders, tasks, notes](#other-modules)
11. [Keyboard Shortcuts](#keyboard-shortcuts)
12. [Accessibility Features](#accessibility-features)
13. [Troubleshooting](#troubleshooting)

## Getting Started

Wixen Mail is designed to work with screen readers (NVDA, JAWS, Windows Narrator), and targets WCAG 2.2 Level AA. [Accessibility](accessibility.md) has the full detail, including what has and has not been confirmed with a real screen reader.

### System Requirements
- Windows 10 or later
- Internet connection for email access
- Optional: Screen reader (NVDA, JAWS, or Narrator) for accessibility features

### First Launch
When you first launch Wixen Mail, you'll need to configure an email account to get started.

### When a setting applies

Settings opens with `Ctrl+,`. A setting applies as soon as you press OK: the
next check, the next message you read, the next row painted, all use what you
just chose, with no restart. Two settings cannot do that, and each says so in a
sentence under its control, which a screen reader reads after the control's
name:

- **Log level**, on the Advanced tab, is set up once when the program starts.
  A change takes effect the next time Wixen Mail starts.
- **Default sort order**, on the Reading tab, is read once when the program
  starts, and only in folders whose columns you have not arranged. A folder
  you arranged with `F8` keeps the sort that arrangement carries, as
  [Choosing columns](#choosing-columns-and-what-is-remembered) explains.

Until 2026-09-20 two more waited for a restart without saying so: Mark as read
after, and how dates are written in the lists. Both apply on OK now.

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

### How often an account is checked

Each account's editor has a field, "Check Interval (min)", between 1 and 60,
5 unless you changed it. Since the build of 2026-09-18 it does what it says:
how often this account is checked for new mail when nothing is watching it,
or for the folders a watch does not cover. The field had been on the editor
since 2026-03-01 and nothing read it until that build. Most of the time the
account's inbox is watched through an open connection instead, and the server
says when mail lands; [Getting your mail](#getting-your-mail) says which is
which. There is no second interval on the Settings screen: the one on the
account is the one.

### Other tools

The Tools menu also opens:

- **Message Filters**, rules that sort, mark, or move messages as they
  arrive. Each rule matches on a field such as subject, sender, or date,
  and can mark a message read, star it, move it, tag it, or say a phrase
  first when its row is read; any rule can also play a sound when a check
  finds a match. See [Rules that change how a row is announced](#rules-that-change-how-a-row-is-announced).
- **Contact Manager**, a dialog for the contacts stored for the account you
  are looking at. The [Contacts module](#other-modules) reached with
  `Ctrl+Shift+2` is the fuller way to work with contacts; this dialog
  overlaps it.
- **Signatures**, the text added to the end of messages you send.
- **Tags**, the labels you can put on a message.
- **Sync Contacts**, **Sync Calendar**, and **Sync Tasks**, to sync with
  your provider now. Corrected on 2026-09-18: this line said "rather than
  waiting for the next automatic sync", and there is none; the three
  modules sync when you ask, from here or from each module's own Sync, and
  not on a schedule. Mail is the one thing that arrives on its own, as
  [Getting your mail](#getting-your-mail) says.

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

When you move into the message list, by `Tab` from the folder tree, by `F6`
or by a click, the cursor lands on a row and that row is read: the row you
were on in that folder if it is still there, or the first row, which is the
newest message unless you have sorted the list another way. If you leave the
list and come back, the cursor is where you left it. An empty folder's list
says "No messages". Since 2026-09-19; before that the list took focus with no
row under the cursor and you had to press `Down` to reach the first message.

### Getting your mail

Since the build of 2026-09-18, mail comes down whole and keeps coming, and you
do not have to ask for either. Until that build a check brought down the
newest 500 messages of each folder, the text of a message came down when you
opened it, and new mail stopped arriving on its own the first time the
connection to the server dropped. None of the new behaviour has run against a
real mail account yet; the alpha testing page says what to watch for.

**A check.** `F9` checks every enabled account, one after another, and says
how many when there is more than one. The program also checks every account
when it starts, and each account's inbox is watched through an open connection
so the server can say when mail lands. When that watch ends for any reason but
mail arriving it is started again after a wait, thirty seconds doubling to
half an hour, and when your network comes back it is started again at once.
Where a watch cannot cover, the account is checked on the interval its editor
sets: a POP account, a server that does not offer watching, and the folders
you keep up to date that are not the inbox. The status bar says which of
these is happening: "Watching Inbox for new mail. Checking every 5 minutes.",
"Waiting 2 minutes to watch Inbox again. Checking every 5 minutes.", or
"Checking every 5 minutes." when nothing is watching, with the account named
first when you have more than one.

**Everything comes down.** After every check, every message of every folder
you keep up to date that is not on this computer yet comes down on its own,
five hundred at a time, the folder you are looking at first, then the inbox,
then the rest in the order of the folder tree. Then the text of each message,
fifty at a time, unless the Message Text box on the Permissions tab is off or
the size you chose there has been reached. The download picks up where it was
after a restart, because what it knows is what is already on this computer.
A provider that stops answering is left alone for a growing time and asked
again; the status bar says so.

**Pause Downloading**, a check item on the Tools menu, holds it: the chunk in
flight finishes, no new one starts, and the mail already here stays readable.
Untick it and the download carries on. The pause lasts until you close the
program. **Get Older Messages**, `Shift+F9`, puts the folder you are in at the
front of the queue; it no longer fetches one page, because there is no page.

**How much is said while that happens** is yours to choose, on the Feedback
tab in Settings, at the end, under While fetching: Say what arrived, Say
every step, or Errors only. Say what arrived is where a new installation
begins. Each step of a check or a download, Connecting, Checking a folder, a
chunk downloaded, is shown on the status bar under every answer and spoken
only under Say every step. What arrived is said once when a check ends,
folder by folder with its count, "Inbox, 3 new messages; Work, 1 new
message", and never when nothing arrived; when a download of a whole account
ends, one sentence says how many folders are whole and how many messages have
their text here. An error is said whatever you chose, and so are the answers
to a key, such as "Settings saved", even while a check is running. The sound
for new mail plays when a check found mail and follows its own row on the
same tab.

### Message Status

Wixen Mail deliberately avoids icons for status that matters, since an icon
is something a screen reader user has to be taught to decode. Whether a
message is read or unread, starred, or has an attachment shows as a real
column in the message list, and reads as a word: "unread", "starred", "has
attachment". `Space` on a message reads its full status along with the rest
of the item, once for a short summary and again for everything.

### The Snippet column

The Snippet column holds a few words from the message's text, so a row gives
you a hint of what the message is about before you open it. Since 2026-09-19
those words are chosen by reading the text, not by cutting it at a fixed
length. A web address or an email address is left out rather than spelled to
you letter by letter. The lines marketing mail puts above its words, such as
"View this email in your browser", are skipped when they are recognisable. A
greeting on a line of its own, such as "Hi Pratik,", is skipped when the
message goes on after it. In a reply, the quoted lines are left out, and so is
the sender's signature after its `-- ` line. What is left is the first
sentence or two of the message itself, ended at a full stop where one falls
inside the room the column has. A message that holds nothing but the lines
these rules skip still shows its least bad line, so a row is never blank for a
message that has text. The whole message, addresses and all, is in the message
itself. Until 2026-09-19 the column held the first 200 characters as written,
so a message that opened with a link read the whole link out.

### Addresses written out are links

A web address or an email address written out in a message is a link, since
2026-09-19. That matters most in a message sent as plain text, such as a
chapter alert or a notification, where the sender typed the address on a line
of its own rather than putting it behind words: the address is a link in the
preview and in the reader window, your screen reader's link list finds it,
and it goes where any link in a message goes. Until then a plain-text message
was shown as characters only, so the address was there to read and nowhere to
go.

What is recognised: an address beginning `http://` or `https://`, one
beginning `www.`, a `mailto:` address, and a plain email address such as
`ada@example.org`. A full stop, a comma or a closing bracket after an address
is left outside it, so "see https://example.org/page." links to the page and
not to the page with a full stop on the end; a bracket the address itself
opened, as in a Wikipedia title, stays part of it. What is not recognised: a
bare site name with nothing in front of it, `example.org`; a version number,
`1.2.3`; a handle, `@ada`; and anything the program would not open, such as an
address with a name before the site, which is left as words.

The same rule reaches the other areas. A note shown as a page has its
addresses as links. An event's or a task's description read aloud with
`Space`, and a note read back, say an address as "link to" its site, "link to
example.org", rather than spelling it out letter by letter; an email address
is said whole. In a reply, the quoted text of a plain-text message carries the
sender's addresses as links too.

A link the program will not open says so beside its words. Wixen Mail hands
web addresses, email addresses and, since 2026-09-19, telephone numbers to
Windows and nothing else, because anything else could start a program on your
machine that the sender chose. A sender's link of any other kind keeps its
words, followed by "(link not opened here:" and the reason, so nothing goes
missing in silence: "Text us (link not opened here: sms)".

The row's Snippet column is the one place an address is not a link and not
said at all: a row is a hint, and the address is in the message.

### Where a link opens

Press `Enter` on a link in a message, or activate it the way your screen
reader activates links, and it opens where you have chosen. The choice is
under Settings, then Reading, then "Open links", and there are three:

- In the default browser. This is the default, and what happens if you never
  change it. The page opens in your own browser, and nothing about it stays
  with your mail.
- In the message view. The page loads in the place the message was, in the
  same window. Wixen Mail says "Opening" and the site's name as it starts,
  says the page's title when it arrives, and says why if the page will not
  open. `Backspace` or `Alt+Left` brings the message back and says "Back to
  the message". A page opened this way shares the browser profile the message
  preview uses, so a cookie it sets is sent again when a later message loads a
  picture from the same site; [What Wixen Mail sends, and
  where](privacy.md#where-a-link-opens) says what that means.
- In a separate Wixen Mail window. That window arrives with the next build.
  Until then this choice opens your browser and the status bar says so.

Whatever you chose, the link's menu offers all three. Press the `Applications`
key or `Shift+F10` on a link, or right-click it, and choose Open in Message
View, Open in Default Browser or Open in Separate Window; Copy Link and Save
Link As are below them. `Ctrl+Enter` opens the browser and `Shift+Enter` a
separate window, whatever the setting, the way those keys work in a browser.
An email address or a telephone number in a message is handed to Windows
whatever you chose, since neither is a page.

Until the build of 2026-09-20 a link opened inside the window whatever the
program meant to do, because the check that was to hand it to the browser was
never given the address; that was #80, and the finding is written there.
Nobody has heard any of this in a screen reader yet.

### Pictures in a message

A message can carry its pictures or point at them on the internet. Since
2026-09-19 both kinds are shown, in the preview pane and in the conversation
window. Until then a picture the message only pointed at was left out until
you found the switch, so most newsletters showed no pictures at all.

Two kinds of pointed-at picture are left out on purpose. A picture whose
declared size is a pixel or less is a tracking pixel, not a picture: it exists
so the sender learns you opened the message. It is not fetched, and the
message says at the top how many it left out, "1 picture that looked like a
tracking pixel was not fetched." A picture the sender marked as having nothing
to say, a decorative one, is left out as well. Fetching any other pointed-at
picture tells its sender the message was opened; [What Wixen Mail sends, and
where](privacy.md#pictures-a-message-points-at) says what that means and what
it cannot prevent.

Two settings on the Reading tab, under Settings, decide the rest. "Do not
fetch any picture a message only points at" is off by default; on, none of
them is fetched, and the message says at the top how many it held back and
that this switch is why. "An undescribed picture is read as" decides what your
screen reader says for a picture the sender gave no description: Nothing, so
it is passed over, which is the default; The word image; or The word photo. A
description the sender wrote is always kept, and a picture inside a link with
no description of its own takes the link's words, so a linked picture reads
as "Our spring range" rather than as "graphic". The same choice reaches a
picture in a note, a task or an event read aloud with `Space`.

### What the formatted view leaves out

A message written as a web page carries things its sender never meant you to
read. Since 2026-09-20 the formatted view leaves them out, and says so when it
matters.

Text the sender hid is not read. A newsletter usually opens with a hidden
block holding the line your inbox shows as its preview, padded with a few
hundred invisible characters so the preview does not run on into the message;
shown as a page, the line was read twice and the padding was read as symbols.
A block is left out only when the sender hid it in one of the ways web pages
hide things (`display:none` and its five siblings), never because of what it
says, and the invisible padding characters are dropped wherever they stand.
When a hidden block held words, other than that short preview line at the
top, a line at the top of the message says so: "1 block the sender did not
show was left out." You hear that line only when something with words in it
was left out, in the same paragraph as the line about tracking pixels, so a
message that hid nothing but its preview line says nothing.

A layout table is not a table to your screen reader. Newsletters lay their
blocks out in tables, dozens of them, and each was announced as a table with
rows and columns on the way in and the way out. A table the sender marked as
layout is now shown as one, so nothing is announced around the block inside
it; a table that holds data is still a table. A grouping the sender named for
a sighted layout, "Post header", is not announced either; a name is kept only
where it is a link's or a data table's.

The page itself says each thing once: the subject as the page's heading, the
sender in the message's heading, which is numbered only in a conversation, and
the count of messages only for a conversation. The sender's own "From" line in
the body is the sender's and stays.

None of this touches a plain-text message, which is shown as written, and none
of it touches a message you are writing: a reply that quotes a web-page message
quotes the whole of it, hidden parts and all, because it is yours to send.

### When a message counts as read

Moving through the list never marks anything. You can arrow through a folder,
let your screen reader finish every row, and the unread count stays where it
was. A message counts as read once you have read the whole of it: press
`Space` twice, or `Shift+Space`, to hear the message itself from the list, or
`Enter` to open it in its own window. The first `Space`, which reads the
subject, the sender and the snippet, does not count. Then the delay under
Settings, then Reading, then Mark as read after runs, two seconds unless you
have changed it, and the message is marked read if it is still the one you are
on. Move off it before the delay runs and it stays unread. Choose Only when I
say so and nothing is ever marked on its own; marking by hand still works as
it did. Until 2026-09-18 the delay was counted from the moment a row was
selected, so listening to a row was enough to mark it, and from the build of
2026-09-18 until this one the first `Space` counted too. Changing the delay
applies as soon as you save Settings: the next message you read is marked after
the new wait, with no restart. Until 2026-09-20 the delay was read once when
the program started, so a change made in Settings did nothing until the next
start.

### Choosing columns, and what is remembered

`F8`, or View then Columns, opens the column chooser. `Space` shows or hides
the column you are on, `Alt+Up` and `Alt+Down` move it, and `Alt+R` puts back
the columns that folder starts with. Every column you turn on is another thing
your screen reader reads on every row, so this is where you decide how much you
hear.

Three different things are remembered, and they are remembered for different
lengths of time.

**The Thread column, for each folder on its own.** It appears in folders where
messages are grouped into conversations and stays out of folders where every
message stands alone. If you show or hide it yourself, your choice wins in that
folder from then on, and it is kept when you close the program.

**The rest of the arrangement, for each kind of folder, while the program is
running.** Which columns are shown, what order they are in, and how the list is
sorted. There are two kinds of folder. Your inbox, junk, trash, archive and any
folder you made yourself all hold mail that arrived, so they share one
arrangement. Sent and Drafts hold mail you wrote, so they share another, without
an Unread column and sorted by when a message went rather than when it turned
up. Arrange your inbox, look in Sent, come back, and your inbox arrangement is
still there.

**One arrangement, saved when you close the program: the one you changed most
recently.** If you last arranged columns in Sent, that is what comes back in
Sent next time you start, and your inbox opens with its usual columns. Sorting a
column counts as changing the arrangement.

### Hearing a row's columns with their headings

`Ctrl+Shift+;` reads the row you are on column by column, each heading and
then its text, in the order the columns are shown: "Subject, Quarterly
report. Correspondent, Ada Lovelace. Unread. Received, yesterday." The same
command is on the Action menu as Read the Row's Headings and Text. It reads
once each time you press it, it works on a conversation row as well as a
single message, and because it is your mail it is silenced by `Ctrl+M` like
every other reading. Press it when a cell has left you unsure which column
you heard.

Whether the headings are also spoken on every row as you arrow is your screen
reader's setting, not this program's. Under NVDA it is Document Formatting,
"Row/column headers", and the way to turn it off for Wixen Mail alone is a
configuration profile that switches on while this program is in front: NVDA
menu, Configuration profiles, New, and under "Use this profile for" choose
"Current application"; then set "Row/column headers" to "Rows" or "Off" while
that profile is active. The step-by-step version, with the names NVDA uses
and the date they were read, is in
[Keyboard Shortcuts](KEYBOARD_SHORTCUTS.md) under NVDA Shortcuts. Nobody has
heard this program under such a profile yet.

### Rules that change how a row is announced

A rule can change what a row says, not only where the message is filed.
Since 2026-09-19 the rule editor (Tools, then Message Filters) offers three
things for that.

**A phrase said first.** Choose the action **Say this first** and type a
few words in the box under it, which is called "Phrase to say first" while
that action is chosen. Up to 40 characters, "Urgent" or "From the school".
Every message the rule matches when it arrives keeps the phrase, and the
phrase is the first thing your screen reader says for the row, before the
first column, whatever columns you have on and in whatever order: "Urgent,
Unread, Quarterly report". The comma is a pause. When the first column has
nothing to say for that row, the phrase is said on its own. The phrase is
also shown in a column of its own, **Says first**, which you can switch on
with `F8`; the column can be hidden and the phrase at the start of the row
cannot, because being heard first is the point of it. Reading the row with
its headings (`Ctrl+Shift+;`) says the phrase without the words "Says
first" in front of it.

The rules run once, when mail arrives. A message that arrived before you
wrote the rule has no phrase, and a message keeps its phrase even if you
change the rule later. When two rules both say a phrase first, the rule
lower in the list wins, as it does for the other choices a rule makes.

**A sound when the rule matches.** Tick **Play a sound when this rule
matches**, which any rule can carry whatever its action. The sound plays
once after a mail check that found any match for the rule, however many
messages matched and however many folders they were in, so a folder of
matches is one sound and not a hundred. It is one event, **Rule matched**,
shared by every rule with the box ticked; the sound scheme decides what it
sounds like, and its row on the Feedback tab of Settings decides what else
happens with it: by default it is also spoken, "Rule matched, 3 messages",
and shown on the status bar. A rule without the box ticked plays nothing.

**The labels on a row.** The labels you have put on a message, by hand or
with a rule that adds one, were shown as a colour and heard as nothing.
There is now a **Labels** column, off by default: switch it on with `F8`, or
View then Columns, and every row reads its labels by name, "Work, Money".
On a conversation row it reads every label on any message in the
conversation, once each.

Nobody has heard a phrase at the start of a row or the sound after a check
yet; the tester's copy is the first that will.

### Message Actions

**Using Context Menu (Right-Click):**
1. Right-click on a message in the message list
2. Select an action:
   - **Reply** - Reply to the sender
   - **Forward** - Forward the message to someone else
   - **Delete** - Move to trash
   - **Toggle Star** - Add or remove star/flag
   - **Mark as read**, or **Mark as unread** - Whichever the message you are
     on needs. The entry says which way it will go, and so do the Action menu's
     item and the toolbar button. Until 2026-09-18 this list named a command
     called Mark as Unread, which never existed under that name: the one
     command said Mark as read whatever the message's state, and toggled.

**Using Keyboard Shortcuts:**
- `Ctrl+R` - Reply
- `Ctrl+Shift+R` - Reply all
- `Ctrl+L` - Forward
- `Delete` - Delete message
- `S` - Star/flag message
- `M` - Mark the message as read or as unread, whichever it is not, and hear
  which. `Space` reads the message aloud; until 2026-09-18 this line said it
  toggled read and unread, and it never did.

### What a delete says

`Delete` says the one word "Delete" and then the next message's row, which the
cursor lands on: the message after the one you deleted, or the one before it
when you deleted the last. Nothing more is said when the delete went through;
the row you land on is the confirmation. If the delete failed, the reason is
spoken. The status bar at the bottom of the window shows the fuller line,
"Deleting Invoice..." and then "Moved to Trash: Invoice", for anybody who
looks. Move to Trash, Move to Folder and Copy to Folder work the same way: one
word on the key, the fuller line on the status bar, and the outcome spoken only
when the message stayed where it was, which for a copy is always, since the
copy is the only thing that tells you it happened. Since 2026-09-19 a copy
within one account is made here at once, so its sentence, "Copied to Work:
Invoice", is the answer to the key and no one word comes before it. Since 2026-09-18; until then a delete
said "Deleting Invoice..." on the key and "Moved to Trash: Invoice" after the
server had answered, two sentences with the subject in each, and the wait for
the second slowed the hand.

### Selecting more than one message

Since 2026-09-19 the message list selects more than one message, the way
every Windows list does: `Shift+Down` and `Shift+Up` grow the selection a
row at a time, `Shift+Home` and `Shift+End` select to either end, and
`Ctrl+A` selects everything shown. Your screen reader says "selected" for
each row you add, "not selected" for a row you take back, and the count on
its own after `Ctrl+A`; those are the list control's own words.

The selection is what the commands act on. Delete, Delete Permanently, Move
to, Copy to, Mark as Read or Mark as Unread, Star or Unstar, and the label
keys each act on every selected message and say one sentence with the count:
"3 messages marked read", "4 messages moved to Archive", "Important removed
from 3 messages". Delete says "Delete" once, and the cursor lands after the
last of the deleted messages once they have gone. Mark as Read marks every
selected message read when any of them is unread, and every one unread
otherwise; the command's label says which way it will go. Reply, Forward,
Open and Save As act on the message the cursor is on, whatever else is
selected.

A conversation row stands for every message in it. Mark as Read, Star and a
label on a conversation row reach every message in the conversation,
wherever it is filed, so marking a thread from its row marks the whole
thread, and the sentence says so: "1 conversation, 5 messages marked read".
Move to and Copy to take the conversation's messages in the folder you are
reading and no others. Delete reaches as far as the "Deleting a conversation
row" setting says, and asks first, naming how many messages it holds.

No command runs over more than 5,000 messages at once, the bound Select All
already had. Above it, one sentence says how many are selected and asks you
to select fewer, and nothing is written. Marking 5,000 messages read in
this computer's store took 75 ms on a release build on 2026-09-19; the
changes then queue for the server one message at a time, and what a server
makes of thousands at once has not been measured against any provider.
Until 2026-09-19 the list took one selection, so `Shift+Down` moved it
instead of growing it and every command acted on one message.

### Moving, deleting and copying happen here first

Since 2026-09-19, a move, a delete or a copy of a message happens on this
computer first, within one account and, since later that day, to a folder
on another account as well. The row leaves the list the moment
you choose the folder in the Move to window, or press `Delete`, and the
cursor lands on the next message the way it does after any delete. The
status bar shows "Moved to Archive: Invoice" or "Moved to Trash: Invoice";
nothing is spoken on success, because the row you land on is the
confirmation. A copy's row stays, so a copy is spoken instead: "Copied to
Work: Invoice". The server is told in the background straight away, and
again at the next check for mail before any folder of that account is read,
so a move made with no network goes when the network is back and you next
check for mail, and a check does not bring the message back in between.
Closing the program keeps the change: it is replayed at the next check after
you start it again.

"Back where it was" means the server refused. When it does, the message
comes back to the folder it was in, the row reappears if that folder is on
screen, and the refusal is spoken with the server's reason: "Could not move
Invoice to Archive: over quota. It is back where it was." A copy the server
refused says "Nothing was copied". A move that was already done on the
server, by another device or before a restart, is read as done and nothing
is put back.

In the Move to window, `Enter` on a folder is the move; on a folder that
holds other folders it still moves there rather than opening it, and on an
account row it does nothing.

A move or a copy to a folder on another account completes here at once too,
and the servers follow: the row leaves, the status bar shows "Moved to
Work in Home: Invoice", and the message appears in that folder of the other
account. Behind that, the message is fetched from the first account, kept on
this computer, uploaded to the second, and only then removed from the first;
that runs straight away in the background and again at the next check for
mail of either account, so a restart in between finishes it from the kept
copy without asking. If the other account refuses the message, or does not
have it after its connection dropped, the row comes back where it was and
the refusal is spoken: "Could not move Invoice to Work in Home: over quota.
It is back where it was." If the first account will not let the message go
once the second has it, you are told it is in both places. A message larger
than 25 MB cannot be kept here, so it goes the old way, servers first, and
the status bar says "Moving Invoice: larger than 25 MB, so it goes now and
the row leaves when Home has taken it". A message still on its way to
another account cannot be moved, copied or deleted again until the next
check for mail, and says so.

Until 2026-09-19 every move and delete waited for the server to agree before
the row left, which on a slow connection was a noticeable pause on every
key, and `Enter` on the chosen folder did nothing; a move to another
account waited for both servers until later that day, and a restart in the
middle of one asked whether to finish it. Nobody has yet replayed a move
against a real mail server after a restart, nor moved a message to another
account this way; the loopback servers the tests use answer the four ways a
server can, at each of the two servers, and a real account settles the
rest.

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

**What a conversation is, on Gmail and elsewhere.** On a Gmail account, since
2026-09-19, a conversation here is the conversation Gmail shows: Gmail names
each message's conversation itself, that name is asked for with the message's
other details, and it decides which messages belong together, whatever the
headers say. A reply that arrived without the headers that would have joined
it still joins, because Gmail joined it. Mail you already had before that day
gets its name once, at the next check of the account, without downloading
anything again. On every other server a conversation is built from the headers
a reply carries, so a reply a sender's program sent without them stands alone
as a conversation of one. On every server a reply that arrived before its
parent joins the parent when the parent lands.

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

Each folder remembers its own view. A folder you have never set shows
conversations, or messages if you turn off **Show conversations by default**
under Settings, Reading, in the Message List section beside Default sort
order; the next folder you open reads it, with no restart. A folder you set
yourself keeps your choice whatever that setting says, including a folder you
set to one row per message. A folder never set was flat until 2026-09-20; on a
profile from before that day, every folder you never set changes to
conversations on the next build, and turning Show conversations by default off
has them flat again.

**All Inboxes keeps a view of its own.** It follows Show conversations by
default until you press `Ctrl+T` there, and then it keeps your choice the way a
folder does, whatever the folder you came from was showing. Showing
conversations, it lists every account's inbox as conversations. A conversation
belongs to an account, so a conversation whose messages are in two of your
accounts is two rows there, one per account, each counting only that account's
messages; whatever you do to a row, reply, file, delete, mark as read, reaches
that row's own account. Until 2026-09-20 All Inboxes showed whatever the folder
before it had left, so a folder in Thread View put its own conversation rows
under the All Inboxes title, and mail arriving while All Inboxes is open still
does not refresh the list, in either view.

A label and a saved search show one row per message. Pressing `Ctrl+T` there
says so and names the places conversations can be shown: a folder, or All
Inboxes.

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

**Which message a conversation row is.** A row describes the whole
conversation, but when you land on it you are on one message of it. Since
2026-09-19 that message is the one that started the conversation when nothing
in it has been read, otherwise the first message you have not read, in the
order they arrived; and when you have read everything, the one that started it
again. Its sender is the first name the Correspondent column says, with the
other senders after it, each once; the Snippet column is its first line; the
preview under the list shows it; `Space` reads it; and `Enter` opens the
conversation window with the cursor on it, so `Enter` again opens that message
and `Up` reaches the whole conversation on the first row. Landing on a
conversation row also fetches the text of every message in the conversation
that is not on this computer yet, up to fifty at a time, in the background and
without saying anything, when the account's Message Text box allows reading.
Before this the row's snippet was the newest message's, the senders came in
the order they were stored, and the preview showed a message that was not the
row's at all.

## Attachments

### Viewing Attachments

A message with attachments is announced as having them, and Wixen Mail does
not use an icon for this: your screen reader hears it in words rather than
having to identify a glyph. Select the message to see the attachments listed
below the message body in the preview pane, or press `Alt+A` from inside an
open message to jump straight to the list, in the formatted view and in the
plain-text reader alike; `Alt+A` again goes back to the message. Until
2026-09-18 the key was `F8`, and it worked only in the plain-text reader.

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

## Import and Export

Three commands on the File menu move mail in and out of Wixen Mail. None of
them has a shortcut key, because each is done once, when you move in or move
out, and a key nobody presses twice would sit in the way of one somebody
presses every day. Press `Alt+F` to open the File menu and arrow to them.

| Command | What it takes | What it leaves |
| --- | --- | --- |
| Import Mailbox | One file: a zip of mailbox files, a single saved message (`.eml`), a mailbox file (`.mbox`), or an Outlook data file (`.pst`) | Folders under Imported, in the shape the mail was in |
| Import a Folder of Messages | A folder you choose, holding saved messages and mailbox files, with folders inside it | The same, one folder here for each folder there |
| Export Mailbox | The folder you are looking at, and everything inside it | One zip of mailbox files, one per folder, with the folder names kept |

Imported mail lands on this computer, under a folder called Imported, and
never in one of your provider's folders. That is deliberate. Mail read out of
a file has never been on your provider's server, and a folder that belongs to
a server is the one place it does not belong: every check for mail would
compare it against a provider that has never heard of it. Under Imported it is
yours and nothing can take it away. Importing the same archive twice does not
give you two of everything.

What a file holds is decided from its first bytes rather than from its name,
so a mailbox file called `Inbox` with no ending is read as one. Anything that
is not mail is counted and named in the sentence at the end, not quietly
skipped. The status bar and your screen reader say when an import starts, how
far it has got, and what it did.

Two things to know before you rely on this:

- **No real Outlook data file has been through the `.pst` import yet.** The
  reader is tested against what it hands over, item by item, and against its
  own reading of each kind of item, because neither Wixen Mail nor the library
  it reads with can write a data file to test against. The sentence at the end
  of an import says this too. Check what arrived against Outlook.
- **A Thunderbird profile folder is not recognised as one.** Thunderbird keeps
  each folder as a mailbox file with no ending beside a `.msf` index and, for
  a folder with folders inside it, a `.sbd` folder holding them. Import a
  Folder of Messages reads that as one folder of mail per mailbox file, refuses
  and counts each `.msf`, and puts the folders inside a `.sbd` one level away
  from where they belong, under the `.sbd` name.

To save one message as a file, use File, Save As instead; it writes the message
you are on as `.eml`, named after its subject.

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
- `M` - Mark as read or as unread, and hear which
- `Space` - Read the message aloud

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

**The sounds are on from the start.** Since 2026-09-18 a new installation
plays every event's sound; until then the sounds were off until you turned
them on. One box on the Feedback tab, "Play a short sound for each event",
turns them all off, and if you had turned them off before this change they
stay off.

**A message with an attachment is said once.** When you land on it, the
Attachment column in the row reads "Has attachment" as your screen reader
reads the row, and the attachment sound plays. The event's own words are not
spoken for it, and are not sent to a braille display, because either would
be the same word a second time: the words go to the status bar instead.
Until 2026-09-18 the event was spoken as well, so the word was heard twice,
or three times with the sound on. To have it spoken again:

1. Open Settings (`Ctrl+,`) and go to the Feedback tab.
2. Under **One event at a time**, choose "Has attachment" in the list of
   events.
3. Tick "Announce this event through your screen reader".
4. Press OK.

The line under the boxes says what the event will really do once you have
changed it. **Use the default for this event** puts it back the way it
started, the sound and the status bar.

**The sounds follow your output device.** A headset or a monitor with
speakers coming or going does not silence them: when the device the sounds
were playing through goes away, or when you have chosen a different default
device in Windows, the next sound after a few seconds' quiet opens the
default device again and plays through it. Until 2026-09-19 the device was
opened once when the program started and never again, so the sounds could
stop after a few hours with nothing said. If no device can be opened at
all, the status bar says so once, "The sounds have stopped: no audio output
device could be opened", with the reason Windows gave, and the sounds come
back on their own when a device can be opened again.

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

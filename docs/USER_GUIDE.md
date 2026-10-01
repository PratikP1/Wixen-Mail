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

### Numbers you set

Every number in Settings and in the account editor is a spin control. `Up`
adds one, `Down` takes one away, and the control stops at the ends of its
range, so a number outside it cannot be set. You can also type a number over
the one shown. `Page Up` and `Page Down` do nothing in a spin control yet.
The field you type in carries the same name as the arrows beside it, on both
of the channels screen readers read. Until 2026-09-24 the field had no name of its own, and a
screen reader said nothing for it or said the label before it, colon
included; that was measured on 2026-09-23 and fixed on 2026-09-24.

| Where | Number | Range |
|---|---|---|
| Settings, General | Font size | 8 to 72 |
| Settings, Compose | Hold a message before sending for, in seconds | 0 (no hold) to 60 |
| Settings, Compose | Save drafts automatically every, in minutes | 0 (never) to 10 |
| Settings, Reading | Mark as read after, in seconds | 1 to 600 |
| Settings, Calendar and PIM | Default reminder, in minutes | 0 to 1440 |
| Account editor | Check Interval, in minutes | 1 to 60 |
| Account editor, POP | Days before mail is removed from the server | 0 (never) to 3650 |

The server ports in the account editor are typed rather than stepped: a port
is a number you copy from your provider, not one you count up to.

Font size, Default reminder and Check Interval became spin controls on
2026-09-23 (#35, #73); before that they were text fields that accepted
anything and corrected it when you saved. Mark as read after's seconds were a
fixed list of waits until the same day.

## Account Setup

### Adding an account

1. Press `Ctrl+Shift+A`, or open the Tools menu and choose Account Manager.
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

The Account Manager (`Ctrl+Shift+A`) is also where you manage the accounts
you have already added:

- **Edit** changes an account's settings.
- **Look People Up at Work** (`Alt+L`) sets up your organisation's directory
  for the account you chose, so a colleague's name typed into a message can
  be found. See [Looking people up](#looking-people-up).
- **Other Addresses to Send From** (`Alt+O`) keeps the other addresses the
  account you chose sends from. See
  [Sending from another address](#sending-from-another-address).
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

Each account's editor has a spin control, "Check Interval (min)", holding 1 to
60, 5 unless you changed it: `Up` and `Down` step it by a minute and stop at
the ends, or type the number. It was a text field until 2026-09-23 (#73).
Since the build of 2026-09-18 it does what it says:
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
  and can mark a message read, star it, move it, label it, or say a phrase
  first when its row is read; any rule can also play a sound when a check
  finds a match. See [Rules that change how a row is announced](#rules-that-change-how-a-row-is-announced).
- **Contact Manager**, a dialog for the contacts stored for the account you
  are looking at. The [Contacts module](#other-modules) reached with
  `Ctrl+Shift+2` is the fuller way to work with contacts; this dialog
  overlaps it.
- **Signatures**, the text added to the end of messages you send, and which
  account uses which. See [Signatures](#signatures).
- **Labels**, the Label Manager, where you make, rename, recolour, delete
  and order the labels you put on messages. See [Labels](#labels).
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

### Labels

A label marks a message as belonging to something, such as Work or Later,
without moving it. The **Label** submenu of the Action menu lists your
account's labels in their order, each with its key: `Ctrl+1` puts the first
label on the messages you have selected, `Ctrl+2` the second, up to `Ctrl+9`.
Pressing the key again takes the label off. A tenth label and any after it
have no key and are on the submenu alone, which shows up to fifty.
`Ctrl+0` takes every label off.

An account starts with five labels: Important, Work, Personal, To Do and
Later, in that order, which is Thunderbird's.

**Edit Labels**, at the end of the submenu, and **Labels** on the Tools menu
open the Label Manager. Its list shows each label with the key that applies it
and its colour. From it you can:

- add a label, rename one, or change its colour with **Add** and **Edit**
- delete a label with **Delete**
- move a label up or down the order with **Move Up** and **Move Down**, or
  `Alt+Shift+Up` and `Alt+Shift+Down` in the list. Each move says where the
  label is now, such as "Later, 2 of 5.", and the key that applied it now
  applies the label it swapped with.

The order and the names are saved when you close the Label Manager, and the
submenu and the sidebar's Labels branch show them straight away.

If you used labels before 2026-09-24, they keep the order their keys applied
them in, which was alphabetical: Important, Later, Personal, To Do, Work. Until
then the submenu listed the five starting labels in Thunderbird's order
whatever your labels were, so `Ctrl+2` said Work and applied Later. The
submenu now says what each key does, and you can put the labels in any order
you like.

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
- In a separate Wixen Mail window. Since 2026-09-22 this opens a window with
  nothing in it but that page: no mail, no folders, and a browser profile of
  its own, so a cookie the page sets is never sent with a message's pictures.
  `Escape` or `F6` closes it, `Backspace` or `Alt+Left` goes back, and a link
  on the page opens in the same window. It is a second copy of Wixen Mail,
  and closing the window ends it. If it will not start, your browser opens
  instead and you are told why.

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
on. Move off it before the delay runs and it stays unread. Mark as read after
offers three choices: Immediately, After a number of seconds, and Never. The
seconds are a spin control beside the choice, holding 1 to 600, which you can
reach only while After a number of seconds is chosen. Choose Never and nothing
is ever marked on its own; marking by hand still works as it did. Until
2026-09-23 the choice was a list of seven, with five fixed waits, and Never was
called Only when I say so; a wait you had chosen from that list is kept. Until 2026-09-18 the delay was counted from the moment a row was
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
Since 2026-10-01 you can also run a rule over a folder yourself, which
gives the messages already there the phrase; see Running a rule over a
folder, below.

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

**A rule that adds a label.** Until 2026-09-28 a rule that adds a label put
nothing on at all. Now the name you type in the rule is matched against the
account's labels, in any capitals, so a rule that adds "money" puts on your
label Money. Make the label first, in the Label Manager: a rule naming a label
the account does not have puts nothing on, and after the check you hear which
label was missing, once however many messages the rule matched. A rule's label
stays on this computer and is not sent to your mail server, and a later check
can take it off again when the server reports that message.

Nobody has heard a phrase at the start of a row or the sound after a check
yet; the tester's copy is the first that will.

### Running a rule over a folder

A rule runs on its own when mail arrives. Since 2026-10-01 you can also
run one over a folder whenever you choose, for example to file the
newsletters that arrived before you wrote the rule. This is
experimental: no rule run has yet met a real mail server.

There are two ways in, and both end in the same question.

From the folder you are reading:

1. Open the folder in the folder tree.
2. Press `Alt+A` for the Action menu, then This Folder, then **Run a Rule
   on This Folder** (`L`).
3. Choose a rule from the list and press **Count** (`Alt+C`). A rule that
   is switched off is listed as "Newsletters (switched off)": you can run
   it by hand, and the list tells you it does not run on its own.

From the Filter Manager (Tools, then Message Filters):

1. Move to the rule in the list.
2. Press **Run on a Folder** (`Alt+R`). The manager saves the rules as you
   left them and closes, so the rule that runs is the rule as saved.
3. Choose a folder from the list of the account's folders and press
   **Count** (`Alt+C`).

Wixen Mail then counts what the rule would change in that folder, which
on a large folder can take a moment, and asks before it changes anything,
for example: "The rule Newsletters would move 214 messages in Inbox to
Archive. No rule run has met a real mail server yet. Run it?" Messages
the rule matches that are already the way it would leave them are not
counted as changes, and the question says both numbers: "The rule Mark
read matches 230 messages in Inbox; 12 would be marked read and 218 are
read already."

- **Yes** runs the rule, and one sentence says what it did:
  "Newsletters: 214 messages moved to Archive".
- **No** changes nothing.
- **Enter** answers Yes, except before a rule that deletes, where Enter
  answers No, so a key pressed before the question has finished cannot
  delete a folder's mail.
- When the rule would change nothing, you hear that instead, and nothing
  is asked.

The buttons read Yes and No, rather than Run and Don't Run, because the
toolkit this program is built on cannot rename them; the question ends
"Run it?" so that Yes and No answer it.

One run changes at most 5,000 messages. When a rule would change more,
the question says so, and running it again changes the next 5,000,
because the ones already changed are no longer counted.

A run goes through the same steps as marking, flagging, labelling, moving
or deleting messages by hand. It changes your mail at your provider only
when Allowed Changes lets Wixen Mail change your mail in that account; if
it does not, you are told so instead of being asked. Edit, Undo takes back
a run the way it takes back the same change made by hand.

### Meeting invitations

When a message carries a meeting, Wixen Mail says what the meeting is before a
word of the message. The sentence is at the top of the bar above the message,
so it is spoken as the message opens, and it is repeated at the top of the
message itself, under the header lines. The reader window, the formatted
window and the preview pane all say it. On a message that is also signed, it
comes before the account of the signature, so it is still spoken.

For an invitation, the sentence names the meeting, when and where it is, and
who sent it, then says what it means for your calendar:

> Meeting invitation: Quarterly review, 05/03/2026 at 09:00 to 10:00, in Room
> 4, from Ada Lovelace, and it is new to your calendar.

The date is written in full, the way you chose on the Reading tab of Settings,
even where a list would say how long ago.

The time is always said on your computer's clock. When the organiser wrote the
meeting in a time zone whose clock differs from yours, the sentence also says
that clock, once, right after the time, because the covering note usually gives
the organiser's own hour. A meeting set for nine in the morning in Tokyo, heard
on a computer in New York:

> Meeting invitation: Quarterly review, 04/03/2026 at 19:00 to 20:00, which is
> 05/03/2026 at 09:00 to 10:00 Tokyo Standard Time, in Room 4, from Ada
> Lovelace, and it is new to your calendar.

The other clock's date is said only when it is a different day there. Nothing
is added for a meeting that lasts all day, for a time a server wrote in
universal time (UTC), or when the other clock agrees with yours. The answer
buttons say your time only. Wixen Mail knows the time zone names Outlook
writes, such as Pacific Standard Time, and the ones Google and calendar
servers write, such as America/Los_Angeles, said as "Los Angeles time". A time
zone it cannot place, such as one an organiser built by hand in Outlook, leaves
the time as it was written, and the sentence says so: "05/03/2026 at 09:00 to
10:00, as written in Customized Time Zone, a time zone this computer cannot
place".

The end of the sentence is one of four:

| The sentence ends | What it means |
|---|---|
| and it is new to your calendar | Nothing on your calendar goes by this meeting's name. |
| a change to the meeting on your calendar, which was ... | Your calendar holds the meeting at another time, or you answered an earlier version of it, and the sentence says when it was. |
| and you accepted this version, and you said you might attend this version, or and you declined this version | You answered this version here already, or a later one, and that was your answer. |
| and you have answered this version | The same, for an answer given before Wixen Mail kept which answer it was. |
| and it is already on your calendar | Your calendar holds the meeting at this time: Google put it there when the invitation arrived, or the organiser's update already moved it there, when you opened the update or at the provider. |

Other messages about meetings say:

- A cancellation: "Meeting cancelled: Quarterly review. It is on your
  calendar." or "It is not on your calendar."
- Somebody's answer to a meeting you called: "Grace Hopper accepted your
  meeting: Quarterly review." The answer is accepted, declined, or "said they
  might come to".
- A calendar file that asks nothing, such as a published calendar: "This
  message carries a calendar file."

The attachment row for the calendar part says the same thing in a word or
two: meeting invitation, meeting cancellation, reply to your meeting, or
calendar file. A file that Windows would run is still called a program,
whatever it claims to be.

#### Answering an invitation

A message whose invitation you can answer has three buttons: Accept,
Tentative and Decline. In the plain-text reader they come after the bar and
before the message; in the formatted window, which is how a message opens
unless you chose plain text, they come after the message and before its
attachments. Each button's name is its answer, and your screen reader reads
after the name what pressing it will do and who will be told:

> Accept Quarterly review, 05/03/2026 at 09:00 to 10:00. Ada Lovelace will be
> told.

If you answered this meeting here before, the same sentence says so and
whether pressing the button says the same again or replaces that answer.

An invitation can be for one day of a repeating meeting, such as one
Thursday of a weekly meeting. Each button then says so:

> Accept one day of Weekly sync, 12/03/2026 at 09:00 to 10:00. Ada Lovelace
> will be told.

Your answer names that day, so the organiser's calendar program reads it as
an answer to that day only, and every other day keeps the answer you gave the
whole meeting. What happens on your calendar depends on where the repeating
meeting is:

| Where the repeating meeting is | What your answer to one day does there |
|---|---|
| A calendar server, or a calendar kept on this computer | That day becomes an appointment of its own showing your answer, and the repeating meeting skips it. Declining one Thursday leaves that Thursday free and every other Thursday as it was. |
| A Google or Outlook calendar, or a calendar this program can only read | Your answer is sent and your calendar is left as it was. The status line says so after the answer: "That day on your calendar was left as it was, because one day of a repeating meeting cannot be changed on its own in your Google calendar from here." |
| Not on your calendar at all | That day is put on your calendar on its own, where a new meeting goes. |
| On your calendar as a single meeting on that day | Your answer is filed on it, as for any meeting. |

The next time the invitation for that day is opened, its buttons say how you
answered that day, not how you answered the whole meeting.

You do not have to reach the buttons. From anywhere in the message:

| Key | Answer |
|---|---|
| `Alt+C` | Accept. It is C rather than A because `Alt+A` is the attachments. |
| `Alt+T` | Tentative, that you might attend. |
| `Alt+D` | Decline. |

The same three are at the end of the message list's context menu
(`Shift+F10` or the `Menu` key) on a message with a calendar file attached,
as Accept invitation, Tentatively accept invitation and Decline invitation,
and on the Action menu as Answer Invitation.

When an invitation cannot be answered there are no buttons at all, rather
than greyed ones you would pass without hearing why, and the bar says why.
In a new installation that is usually "Sending mail is switched off, so no
answer can reach the organiser", with the setting to change; it can also be
that the invitation was not addressed to you, or that it changes a repeating
meeting from one day onwards, which you answer by hand. A conversation of
several messages shows no buttons; open the message on its own to answer it.

After you answer, the status line says what happened, once. The answer goes
into the outbox under the same hold as any message, so Undo Send takes it
back for the first seconds. It is sent as a reply to the invitation, with the
headers a mail program uses to file a reply under the message it answers. The
meeting is put
on your calendar, and your answer is kept beside it, so the next time the
invitation is opened the sentence says which way you answered. A repeating
meeting goes on your calendar repeating, without the days its organiser
called off.

If your calendar provider has not sent the meeting yet, your answer puts it
on My Calendar. The next calendar check moves it to the provider's calendar
as the provider's copy of the meeting, rather than adding the meeting a
second time. Your answer, and whether the meeting takes up your time, stay
with it. Google keeps only busy or free, so a Tentative answer shows as busy
there after the next check. A title, place or category you typed onto the
meeting before that check is replaced by the provider's.

#### When the organiser moves or cancels a meeting

Opening an update or a cancellation in the reader window or the formatted
window can change your calendar, but only when it comes from the organiser
your calendar records for that meeting. Anybody can send a message that names
a meeting, so a change from anybody else is said and never applied.

| The message | What happens |
|---|---|
| An update from the organiser that moves the meeting | The meeting moves on your calendar when you open the message, and the bar says so: "Moved on your calendar from 05/03/2026 at 09:00 to 06/03/2026 at 14:00." |
| A cancellation from the organiser | The bar says "The organiser has called this meeting off. Remove from Calendar takes it off yours." and there is one button, Remove from Calendar, on `Alt+R`. Pressing it marks the meeting cancelled, so the time is free, and keeps it on your calendar marked that way. Nothing is sent to the organiser. |
| Either, from somebody other than the organiser | Nothing on your calendar changes, and the bar says who it came from and who the organiser is. |
| Either, for a meeting your calendar does not record an organiser for | Nothing changes, and the bar says the meeting on your calendar does not say who organised it. A meeting put on your calendar by an earlier version records nobody until your calendar provider sends it again, or you answer it here. |
| An update from the organiser that moves one day of a repeating meeting | That day moves on your calendar when you open the message, and every other day stays where it was. The day becomes an appointment of its own, and the repeating meeting skips it. The bar says so: "Moved one day of this repeating meeting on your calendar, from 12/03/2026 at 09:00 to 13/03/2026 at 14:00." A later update for the same day moves that appointment again. The meeting's own sentence says "one day of a repeating meeting" and compares the day, not the day the meeting started. |
| A cancellation from the organiser for one day of a repeating meeting | The bar says "The organiser has called off one day of this repeating meeting, 12/03/2026 at 09:00 to 10:00. Remove from Calendar takes that day off yours." Remove from Calendar, on `Alt+R`, takes that one day off and keeps the repeating meeting and every other day. The bar then says, for example, "Weekly sync: that one day is taken off. The other days are unchanged." Nothing is sent to the organiser. |
| An update or a cancellation for every day of a repeating meeting | It is applied as a single meeting's is: an update moves the whole repeating meeting and keeps how it repeats and the days it skips, and Remove from Calendar marks the whole meeting cancelled. |
| Either, for one day of a meeting in a Google or Outlook calendar, or in a calendar this program can only read | Nothing changes, and the bar names the calendar: "Your calendar was not changed, because one day of a repeating meeting cannot be changed on its own in your Google calendar from here." Google and Outlook are not told how a meeting repeats when it changes, so the day kept apart would arrive there as an extra meeting. |
| Either, changing a repeating meeting from one day onwards | Nothing changes, and the bar says "Your calendar was not changed, because this changes the meeting from one day onwards, and that is not done here." |
| Either, inside an encrypted message | Your calendar is left as it was, and the bar says the change came inside encrypted mail, which is never applied on opening. [Signed and encrypted mail](#signed-and-encrypted-mail) says why. |

The preview pane never changes your calendar. It says what the message is,
as above, because it opens a message just by moving past it, and a meeting
should not move because the cursor did. The account's Allow Changes answer
applies: with changes to calendars switched off for the account, the bar says
so and nothing moves. A change made here is sent to your calendar provider
like any other change to an event, the next time the calendar is checked. On
a calendar server, one day moved is sent as two entries, the appointment for
that day and the repeating meeting with the day skipped, as the event
editor's "Just this one day" already sends it, so other calendar programs
show them as two.

What this does not do yet: no invitation, update or cancellation from a real
Outlook, Google or calendar server organiser has been read here, for one day
or for a whole meeting, nor has any answer reached one, and a move or a
removal sent back to Google or Microsoft after they applied the same update
themselves has not been tried. A sender's
address can be forged, and nothing here checks the provider's own verdict on
it yet. Nobody has listened to the buttons with a screen reader yet, nor to the
other clock the sentence says for a meeting set in another time zone, and no
invitation from a real organiser in another time zone has been read here. The
calendar shows and says such a meeting on your clock too, as Time zones under
Other modules describes.

A message whose text was downloaded in the background before this version
had no record of its attachments. The first time you select
one, it is downloaded once more, whole, from its own account, and from then on
it lists its attachments and says its meeting. Nothing is said while that
happens, and the message is not read out a second time; if the preview is
showing it when a meeting arrives, the preview loads once more with the
meeting at the top. Three cases are not reached that way. An invitation sent
only as part of the message's text rather than as a file, which is how
Outlook often sends one, is not found, because the server's description of
such a message says it carries no attachment. A message you only ever open
inside a conversation window is not downloaded again. And a reader window
opened in the moment before the download finishes shows no attachments;
close it and open the message again.

### Signed and encrypted mail

A message can be encrypted to a certificate, so that only somebody holding the
certificate's key can read it. This is S/MIME, the kind Outlook and most
workplace mail use. When one arrives, Wixen Mail offers it to the certificates
in your Windows certificate store, and Windows opens it with the key there.
Opening encrypted mail is experimental.

The bar above the message says one of four things, and the same sentence is at
the top of the message itself:

| The sentence | What it means, and what to do |
|---|---|
| This message was encrypted to your certificate and was opened here. Opening encrypted mail is experimental. | It opened. Its words are below and its files are in the attachment list, as in any message. |
| This message is encrypted to a certificate this computer does not hold a key for, so it cannot be opened here. | It was sent to a certificate whose key is on another computer, or to somebody else. Open it where that key is, or ask the sender to send it to a certificate you hold. |
| This message is encrypted to a certificate on this computer, and Windows would not let its key be used, so it was not opened. | The key is here and Windows would not use it: a prompt was cancelled, a PIN was wrong, or the card holding the key is not in its reader. Open the message again once the key can be used. |
| This message is encrypted, and what arrived is damaged, so it cannot be opened. | The message was changed or cut short on the way. Ask the sender to send it again. |

An encrypted message is opened again every time you read it, and what was
inside is never stored: the mail kept on this computer holds the message
still encrypted. Two things follow. Search does not look inside encrypted
mail, so a word that is only in an encrypted message is not found. And a file
inside one is opened from the message again each time you save or read it.

Pictures in encrypted mail are never fetched, whatever the Reading tab says,
because fetching one would tell the sender the message was opened here. The
message says how many were not shown and why. A meeting inside encrypted mail
is said and can be answered with its buttons, as any other can, and an
organiser's update or cancellation inside one is said and never applied when
you open it.

A message that was signed and then encrypted has its signature checked once it
opens, and the signature is said the way any signature is.

PGP mail is the other kind, the kind Thunderbird and Proton Mail use. Import
your private key in File, PGP Keys, and a PGP message encrypted
to that key opens, whether its encrypted text sits in the body of the message
or in a separate part, which is called PGP/MIME. Reading PGP mail is
experimental, and the menu item says so. A PGP message that opens shows its
words and its files with nothing said above them. A PGP/MIME message that
opens to files and no words says so where its words would be, "This message
was encrypted with PGP and was opened here. It holds files and no words.", and
nothing in the bar. One that does not open shows
its encrypted text, and the bar says why: there is no private key on this
computer, the key here is not the one it was encrypted to, the key could not be
read back, the encrypted part is damaged, or the key is locked with a
passphrase nobody has typed yet. Everything above about opening
again each time, search, pictures and meetings holds for PGP mail too.

What this does not do yet: no encrypted message from Outlook or Thunderbird,
and no certificate somebody really uses, has been opened here; the messages
tested were made with OpenSSL for a key held only while the tests run. A key
that asks for a PIN or a password, or that lives on a smart card, has not been
tried, so what Windows shows then is not known. No PGP/MIME message from
Thunderbird or Proton Mail has been read here either: the one tested was made
with GnuPG. A picture sent inside an encrypted message is not shown yet, and it
is counted among the pictures not shown. Sending signed or encrypted mail is
offered in the composer, experimentally: see [Signing and encrypting what you
send](#signing-and-encrypting-what-you-send).

When a message signed with an S/MIME certificate arrives, its signature holds,
and the certificate names the address the message came from, Wixen Mail keeps
that certificate, so a message to that person can be encrypted to them. A
signature that does not hold, or a certificate naming some other address,
keeps nothing.

#### PGP keys

File, PGP Keys opens the key manager, which lists every PGP key on this
computer: your private keys first, then other people's public keys. Each row
starts with the name and address the key carries, then says whether it is a
private or a public key, its key id, its fingerprint, when it was made, when it
expires, and whether it can encrypt, sign or both. The two dates are written the
way you chose on the Reading tab of Settings, day or month first and in numbers
or words, each the day on this computer's clock; a key that never expires says
Never. The manager is experimental, and the menu says so.

The box at the top of the window says what keys can and cannot do in this
build:

- A private key opens PGP messages sent to it.
- A key locked with a passphrase is kept locked. Its passphrase is asked for
  when you open a message that needs it, and remembered until Wixen Mail
  closes. It is never saved.
- Public keys are kept here, and every key here checks the PGP signatures made
  with it. See [Signed PGP mail](#signed-pgp-mail). Your key signs and your
  correspondents' keys encrypt what you send when you tick Sign or Encrypt in
  the composer; see [Signing and encrypting what you
  send](#signing-and-encrypting-what-you-send).
- Removing a key here removes it from this computer.

A locked key's row says "Private key, locked with a passphrase".

A key says whose it is, and nothing here checks that claim. Before you rely on
somebody's public key, check its fingerprint with them another way, such as on
the phone.

The buttons:

| Button | Letter | What it does |
|---|---|---|
| Import from File | `Alt+F` | Reads every key in a file you choose, private or public, and says what became of each |
| Paste a Key | `Alt+P` | Opens a box to paste a key's text into, then imports it |
| Export Public Key | `Alt+X` | Writes the chosen key's public half to a file you choose. The private half never leaves |
| Copy Public Key | `Alt+C` | Puts the chosen key's public half on the clipboard, to paste into a message |
| Remove | `Alt+R` | Asks first, naming the key and its fingerprint, and says that signatures made with it will no longer be checked. For a private key it also says that messages encrypted to it will stop opening. `Enter` answers No |
| Close | `Alt+O` | Closes the manager. `Esc` does the same |

Every answer is shown on the line above the buttons and said aloud. A button
pressed with no key chosen says to choose one first.

Somebody may send you their public key as an attachment, usually a `.asc` or
`.key` file. Press `Enter` on it in the attachment list, in either reader
window, and Wixen Mail says what kind of key it is, the name it gives and its
key id, and asks whether to import it. `Enter` answers No, so nothing is
imported by accident. A `.asc` file that holds no key, usually a signature, is
read as text instead.

#### A key locked with a passphrase

Most keys exported from another program carry a passphrase, and Wixen Mail
keeps such a key locked, as it came. Nothing asks for the passphrase until a
message needs it:

1. Open a message encrypted to that key in either reader window. A dialog,
   Unlock a PGP Key, opens first. It says
   whose key it is, taking the name from the key itself, never from the
   message.
2. Type the passphrase, or paste it from your password manager. Focus starts
   in the field, and `Alt+P` returns to it.
3. Press `Enter`. The message opens to its words.

If the passphrase is wrong, the dialog opens again and says so first: "That
passphrase did not open the key. Try again." Press `Esc` to stop asking. The
message then shows its encrypted text, with a sentence saying the key is
locked and that opening the message in the reader lets you type it.

The preview pane never asks, because a dialog appearing while you arrow
through your messages would be in the way. It says the same sentence instead.
A conversation opened in the reader asks once for each locked key its
messages need.

Once typed, the passphrase opens every message to that key until you close
Wixen Mail. It is held in memory only. It is never written to your disk, the
mail database or the Windows credential store, and it is gone when Wixen Mail
closes or when you remove the key. There is no command to forget it sooner:
close Wixen Mail.

#### Signed PGP mail

A PGP signature is checked against the keys in File, PGP Keys: the public keys
kept there and the public half of each of your private keys. It is checked
whether the signature is written into the message's text or sent in a separate
part, which is called PGP/MIME, and it is checked again every time you open the
message. Checking signatures is experimental.

The bar above the message says one of six things:

| The sentence | What it tells you | What it does not tell you |
|---|---|---|
| This message's PGP signature holds: it was made by the key in your list for (a name), fingerprint (the fingerprint). That says the key made it; it does not say who holds the key. | The words are exactly the ones that key signed. | Who holds the key. A key carries whatever name and address its maker typed into it. |
| This message carries a PGP signature by key (a key id), which is not in your list, so it could not be checked. | The message is signed, by a key you do not have. | Anything about the words. Nothing was checked, so the signature neither holds nor fails. |
| This message's PGP signature does not hold against the key in your list for (a name). It was changed after it was signed, or the signature is not that key's. | Something is wrong: the words were changed after they were signed, or the signature names a key that did not make it. | Which of the two happened. Read the message as one you cannot trust. |
| This message's PGP signature is damaged, so it could not be checked. | The signature arrived in a form that cannot be read. | Anything about the words. Ask the sender to send it again. |
| This message is signed, and it was stored before Wixen Mail kept the form signed mail arrives in, so the signature cannot be checked. | The message was on this computer before you first ran a version of Wixen Mail that keeps every form signed mail arrives in, and a signature can only be checked against that form. This is said about S/MIME signatures too. | Anything about the words. Nothing was checked. |
| This message carries a signature in a form Wixen Mail does not check. | The message carries a signature as a file of its own, or inside a part another program added around the message, such as a mailing list's footer. Wixen Mail checks only a signature over the whole message as it arrived. This is said about S/MIME signatures too. | Anything about the words. Nothing was checked, so read it as you would an unsigned message. |

To turn the second sentence into the first, get the sender's public key from
them, check its fingerprint with them another way, such as on the phone, and
import it in File, PGP Keys. The next time you open the message, the signature
is checked against it.

The sentence is spoken as the message opens. Under it, after "More about this
signature:", the bar says what the signature was checked against and what a
signature does and does not show. The subject line and the sender line travel
outside the signature, so a signature never covers them.

A message whose signature is written into its text shows the words that were
signed, without the lines of signature text around them. When the message has
words outside the signed text, such as a mailing list's footer, it is shown as
it arrived, lines and all, because those lines are the only thing showing which
words the signature covers.

In a conversation, each message says its own verdict under its own heading,
before its words.

What this does not do yet: no signature from a real correspondent's key has
been checked here; the signatures tested were made with GnuPG. Whether a key
has expired, or its owner has withdrawn it, is not checked. A signature written
only into the formatted half of a message cannot be read, and is said to be
damaged. A signature written into text sent in a character set other than
UTF-8, with letters outside plain English in it, has not been tried, and may
read as not holding. The first time this version opens your mail, it notes the
last message already stored, and only mail up to that one can get the
stored-before sentence. If you ran an earlier version that already kept the
form signed mail arrives in, a message with a lone signature file, or a signed
message a mailing list wrapped, that arrived between that version's first run
and this one's still gets the stored-before sentence rather than the last one
in the table, because nothing stored can tell those days apart.

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
- `Ctrl+Shift+S` - Star or unstar the selected messages. Until 2026-09-20 this
  line said `S`, and `S` on its own has never been bound to anything.
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

**Undo takes a move, a delete or a copy back.** Since 2026-09-25, `Ctrl+Z`
in the message list, or Edit, Undo, puts the messages of your last move or
delete back in the folder they came from, and the cursor lands on the first
of them when that folder is on screen. The menu names what it will take
back, "Undo Move to Archive: Invoice" or "Undo Delete: 3 messages", and one
sentence says what happened: "Undid Move to Archive on Invoice." How it does
that depends on how far the change got:

- **The server has not been told yet**, because there was no network or the
  change is still on its way. Undo takes the change back on this computer and
  sends nothing, so the server never hears of either.
- **The server has done it.** Undo moves the message back from where the
  server now holds it, the same way a move goes: here at once, then the
  server.
- **A copy.** Undo sends the copy to the Trash, here and at the server. It is
  never deleted outright, so a copy that turned out to be the wrong message
  can still be found.

Some things cannot come back, and Undo says so in a sentence naming the
message:

- A message deleted with Delete Permanently, once the server has it. Before
  that, Undo brings it back.
- A message moved or copied to another account. Move it from that account
  instead.
- A message the server moved but whose new place this computer has not read
  yet. Refresh the folder it went to, and move it back from there.
- A message whose change is reaching the server at that moment. Try again
  shortly.

Redo (`Ctrl+Y`) does the move, delete or copy again. Undoing a change at the
server is experimental, and the Undo item's help on the Edit menu says so:
nobody has undone a move against a real mail server yet.

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

### Reporting junk and blocking a sender

Action, Report as Junk (`Ctrl+Shift+J`, or `Alt+A` then `J`) moves the
selected messages to the junk folder and tells the mail provider they are
junk, where the provider has a way to be told. It acts on every selected
message, the way Move to does, and a conversation row contributes the
messages in the folder you are reading. It is experimental: it has never
been run against a real mail server, and the item's description on the menu
says so.

What happens depends on the account, and one sentence for each account says
which:

| Account | What is done | What you hear |
|---|---|---|
| A mail server that keeps a junk mark | The mark is set, then the messages move to the junk folder | "3 messages reported as junk and moved to Junk." |
| Gmail | The messages move to Spam, which Google's own help page says is the report | "3 messages moved to Spam, which tells Google they are junk." |
| A mail server that keeps no junk mark | The messages move to the junk folder, and nothing else can be told | "3 messages moved to Junk. This server does not keep a junk mark, so only the folder says they are junk." |
| Outlook.com or Microsoft 365 | The messages move to Junk Email | "3 messages moved to Junk Email. Microsoft offers no supported way for a mail program to report junk, so Microsoft has not been told." |

Microsoft's only way for a program to report junk is a preview interface it
does not support in production, and it needs permission to read and change
all of your mail, which Wixen Mail does not ask for. So the sentence says
plainly that Microsoft was not told.

Some reports send nothing, and say why in one sentence:

- An account that collects its mail with POP, which has no junk folder at the
  server.
- An account whose folders include no junk folder. Make one on the account,
  check for mail once, and try again.
- An account that has not learned its folders yet. Check for mail once, and
  try again.
- An account whose mail changes are off under Allow Changes in Settings, or
  for that account alone. The sentence is the one a refused move says.

A message already in the junk folder is passed over, and the sentence counts
it: "3 messages reported as junk and moved to Junk, and 1 already in Junk was
passed over." If the mark could not be set, the messages still move and the
sentence adds "The junk mark could not be set", with the server's reason.

The move is the same as Move to: it happens on this computer first and the
server is told in the background, so `Ctrl+Z` in the message list moves the
messages back. A junk mark already set stays on them.

Reporting junk is not blocking. A report deals with the messages in front of
you and tells the provider; a block files everything a sender sends from now
on into the junk folder and tells the provider nothing.

To block a sender, open or select a message from them and choose Action,
Block, This Sender (`Ctrl+Shift+B`), or Everyone at This Domain, which has no
key. Blocking is experimental: the move of the mail already here has never
been run against a real mail server.

1. The block is saved as a rule, and mail from them goes to the junk folder
   from now on. You can see it under Tools, Blocked Senders.
2. If messages from them are already on this computer, one question asks
   whether to move them too, with the count: "Also move the 14 messages
   already here from ada@example.com to Junk?" Enter answers Yes. No leaves
   them where they are, and so does closing the question without an answer.
3. One sentence says what happened, for example "14 messages already here
   were moved to Junk."

What the question counts and what it leaves alone:

| Case | What happens |
|---|---|
| Messages in the inbox and your other folders | Counted, and moved on Yes |
| Messages in Junk, Trash, Sent, Drafts or the Outbox | Left alone and not counted |
| More than 5,000 messages | Nothing is asked and nothing moves. The sentence gives the count and says to search for the sender and use Move to |
| Mail changes switched off under Allow Changes | Nothing is asked. The sentence says the messages stay where they are and that the block waits for mail changes |
| None already here | Nothing is asked |

The move is the same as Move to: it happens on this computer first and the
server is told in the background, and it is refused whole if mail changes
are off for the account. `Ctrl+Z` in the message list moves the messages back.

A block uses the account you have open. In All Inboxes that may not be the
account the message came to.

### Quick Steps

A Quick Step is a command you make yourself that does several things to the
selected messages at once, such as marking them read and moving them to
Archive. Each account has its own. Quick Steps are experimental: none has
been run against a real mail server, and the menu item's description says
so.

To run a Quick Step:

1. Select the messages in the message list. A conversation row brings in the
   messages of its conversation that the step's actions reach.
2. Press the step's key, `Ctrl+Shift+7` for the first step in the account's
   order, `Ctrl+Shift+8` for the second and `Ctrl+Shift+9` for the third. Or
   choose the step on Action, Quick Steps (`Alt+A`, `Q`), where every step is
   listed with its key.
3. One sentence says what the step did, such as "Archive and read: 3
   messages marked read and moved to Archive".

A step changes your mail at your mail provider only when Allowed Changes lets
Wixen Mail change your mail, the same as marking or moving a message by hand.
When a step marks messages and moves them, the marks reach the provider
first and then the move, so the marks stay on the messages where they land.

A step does nothing, and says why, when:

- nothing is selected;
- more than 5,000 messages are selected, the most Select All chooses;
- a selected message belongs to another account than the step, which can
  happen in All Inboxes, since each account's steps name that account's
  folders and labels;
- the step moves to a folder or puts on a label the account no longer has,
  for example after you renamed the folder. Edit the step in Manage Quick
  Steps and choose again;
- the step was written by a newer version of Wixen Mail.

A key past the account's last step says which step it would run and how
many steps the account has.

Edit, Undo (`Ctrl+Z` in the message list) takes back the last thing a step
did, not the whole step. For a step that marks messages read and moves them,
Undo moves them back and leaves them marked read.

To make a Quick Step:

1. Choose Action, Quick Steps, Manage Quick Steps (`Alt+A`, `Q`, `M`). The
   Quick Step Manager opens for the account you are in, on the list of its
   steps.
2. Press Add (`Alt+A`). The step editor opens.
3. Type a name in Name (`Alt+N`).
4. Answer the questions you want the step to do, and leave the rest as they
   are.
5. Press `Enter` for OK. The step is added to the end of the list.
6. Press Close (`Alt+C`) when you are done. Nothing is saved until you close
   the manager.

The editor asks one question per control, in this order:

| Question | What the step does |
|---|---|
| Name | What the step is called. Two steps in one account cannot share a name, in any mix of capitals |
| Mark as read or unread | Leaves the messages as they are, marks them read, or marks them unread |
| Flag | Leaves the flag as it is, flags the messages, or takes the flag off |
| Label | Puts on one of the account's labels, or none. A step puts on one label at most |
| Move to | Moves the messages to one of the account's folders, chosen from the folders the folder tree shows, by path, or leaves them where they are |
| Delete it | Sends the messages to the trash. A step that deletes does nothing else, because a message is not marked, flagged or moved on its way to the trash |
| Phrase to say first | A few words, up to 40 characters, that each message keeps and your screen reader says first on its row, as a rule's Say this first does |

If a step cannot be kept, pressing OK says why in a message box and leaves
the editor open with what you chose: a step with no name, a name another
step has, a step that does nothing, a delete beside anything else, or a
phrase that is too long.

If you open a step whose label or folder the account no longer has, the
editor shows it chosen with "(not in this account any more)" after its name,
so keeping it is a choice you see.

The manager's list has three columns: the name, the key, and what the step
does, "Mark read, move to Archive". The first three steps have a key,
`Ctrl+Shift+7` to `Ctrl+Shift+9`, in the order of the list. Move Up
(`Alt+U`) and Move Down (`Alt+W`), or `Alt+Shift+Up` and `Alt+Shift+Down` in
the list, move a step and its key. Edit (`Alt+E`) changes the step you are
on, and Delete (`Alt+D`) removes it.

A step written by a newer version of Wixen Mail says so in the list. You can
move it or delete it, and Edit does not open it, because this version cannot
read everything it does and saving it would lose that part.

One thing does not work yet. A step given the name another step had when
the manager opened may fail to save, and closing the manager says which one
failed. To pass a name from one step to another, rename the first step,
close the manager, then open it again and rename the second.

### Printing

Press `Ctrl+P`, or choose File, Print, to print what you are on. Windows' own
print dialog opens, the same one other programs use, and you choose the
printer, the number of copies and which pages there. The dialog belongs to
the window you printed from, so closing it takes you back to that window.

What each place prints:

| Where you are | What `Ctrl+P` prints |
|---------------|----------------------|
| A message in the message list | That message |
| A conversation's row in the message list | The whole conversation, every message in order |
| The reader window | What the tab you are on holds: a message, a conversation, or an attachment you opened there |
| The formatted message window | The message or the conversation it shows |
| Contacts, Calendar, Tasks, Notes or Reminders | The contact, event, task, note or reminder you are on |

In the reader window, Print is on its own File menu too, with the letter `P`.

What is printed for a message:

- The header lines, as the reader window shows them: Subject, From, To, Cc
  when there is one, the date written in full, and the names of any
  attachments.
- The words of the message, as the reader window shows them.

A conversation prints each message under a heading that says its place in
the conversation, who sent it and when, with the date written in full, even
where the screen shows how long ago it was. An attachment prints as its tab
shows it.

A contact, event, task, note or reminder prints the same details `Shift+Space`
reads aloud, one to a line, with its name or title first and every date
written in full. A note's text prints as you wrote it. If nothing is chosen
in the list, Print says so, for example "Choose an event first.", and
nothing is printed.

At the top of every page is a line saying "Wixen Mail", the subject or the
name, and the page number out of how many, such as "page 2 of 3". The page is
11 point Segoe UI, black on white, whatever theme or reading size you use on
the screen. There is no setting for it.

What is not printed:

- Pictures and formatting. The page carries the words only, as plain text.
- A web page opened from a link in a message. That page opens in a window of
  its own, and `Ctrl+P` does not print it.

There is no print preview, because Windows' print dialog does not have one.

After the job is sent, Wixen Mail says one sentence naming what was printed,
the printer and how many pages went, for example "Sent Quarterly report to HP
LaserJet 1022, 2 pages." If you close the dialog without printing, it says
nothing was printed. If something goes wrong, it says what, and that nothing
was printed.

In Windows' print queue the job is named by what it is, such as "Wixen Mail
message", "Wixen Mail conversation" or "Wixen Mail contact", and never by its
subject or a name, because other people can see the queue of a shared
printer.

Nobody has yet worked the print dialog with a screen reader from any of these
places, or looked at a page on paper from this build.

## Composing Email

### Creating a New Message

1. Click **File → New Message** or press `Ctrl+N`
2. Enter recipient(s) in the **To:** field
3. Optionally add CC and BCC recipients
4. Enter a subject
5. Type your message in the body field
6. Click **Send** or press `Ctrl+Enter`

### Sending from another address

One mailbox can send as more than one address: a help desk address beside your
own, or an old address that still reaches the same inbox. Wixen Mail calls
these other addresses, and keeps them per account.

**Adding one.**

1. Open the Account Manager (`Ctrl+Shift+A`) and choose the account.
2. Choose **Other Addresses to Send From** (`Alt+O`).
3. Choose **Add**, then fill in **Address** (`Alt+A`) and, if you want one,
   **The name people see** (`Alt+N`), such as "Help Desk".
4. Choose **OK**, then **Close**. The addresses are kept when the window closes.

**Choosing one.** The **From** list (`Alt+F`) sits above To in the message
window. It lists each account's own address, and after it the other addresses that
account sends from, read as "help@example.com, another address on Work". The
message goes out from the address you choose, with the name kept beside it,
through that address's account, and it waits in that account's Outbox, even
when another account is the one you have open. A draft you save keeps the
address, and so does a message you take back with Undo Send: both reopen with
From on it.

**Replying and forwarding.** A reply, a reply to all and a forward open with
From on the address the message was sent to, when that is one of the
account's other addresses. Mail that arrived at help@example.com is answered
from help@example.com, so answering support mail does not send it from your
own address by accident. The message's To and Cc lines are both read, and
capital letters do not matter. When none of the account's other addresses is
there, From opens on the account's own address, as before. You can still
choose another entry in the From list before you send.

**Reply All leaves your addresses out.** A reply to all never sends a copy to
any address you send from: not to the account's own address, and not to its
other addresses. A reply to mail sent to help@example.com does not copy
help@example.com back to itself.

**The signature.** An other address signs with its account's signature, so
moving From between an account's own address and one of its other addresses
leaves the signature as it is. Moving to another account's address changes it,
as described under [Signatures](#signatures).

**What a provider may do.** Your provider decides whether a message may go out
from an address. One that is not set up with the provider too may be refused,
or the provider may send it from your account's own address instead. In Gmail
that is "Send mail as" in Gmail's settings; in Outlook and Microsoft 365 it is
a permission your organisation gives. Sending from another address has not
been tried against a real provider yet, so this is **experimental**.

**Not built yet.** Three things are later work, and issue 59 stays open for
them. Shared mailboxes: opening a mailbox that belongs to a team, not to you.
Sending on behalf of somebody else: a message that says it is from them and
sent by you. And delegation: somebody else reading and answering your mail
with your permission. An other address here is an address your own account
sends as, not somebody else's mailbox.

### Looking people up

Type three letters or more of a name into To, Cc or Bcc and pause. A list of
the people who match, **People found**, appears under the recipient lines, and
how many were found is read out with `Alt+E`, the key that goes to the list.
Each row says the name, the address, and whether the person came from your
contacts, from your organisation's directory or from Microsoft. `Enter` on a row
puts that person in the line you were typing in.

Your contacts on this computer are always searched. Your organisation's
directory is searched only once you have set one up:

1. Open the Account Manager (`Ctrl+Shift+A`) and choose the account.
2. Choose **Look People Up at Work** (`Alt+L`).
3. Fill in **Directory address** (`Alt+D`), such as
   `ldaps://directory.example.com`, and **Where in it to look** (`Alt+W`),
   such as `ou=people,dc=example,dc=com`. Whoever looks after the directory
   at work will know both.
4. If the directory asks you to sign in, fill in **Sign-in name** (`Alt+N`)
   and **Password** (`Alt+P`). Leave both empty for a directory that answers
   anybody.
5. Choose **OK**.

The password is kept in the Windows credential store, never in the settings,
and it is only ever sent to an address beginning `ldaps://`, which is
encrypted. An address beginning `ldap://` is not, so a password is never
sent there, and looking a name up says to ask for an `ldaps://` address
instead.

When you open the window again, the password box is empty and never shows
the saved password. The box, and a line under it, say that one is saved:

- Leave the box empty and choose OK to keep the saved password.
- Type a new one to replace it.
- Clear the sign-in name and choose OK to forget it.

A sign-in name with an empty password box and no password saved is refused:
the window stays open, says so, and puts you in the password box.

This is **experimental**. Looking people up in a directory has not been
tried against a real directory yet, and the window says so first.

On an Outlook or Office 365 account you signed in to through the browser,
Microsoft's people search is asked as well, with nothing to set up. It looks
through the people that mailbox writes to and your organisation's directory,
and its rows end "from Microsoft". A person already in your contacts or found
in the directory keeps that row, so nobody is listed twice. An account signed
in before the build of 2026-09-28 needs signing in again once for this, and
until then each search says so. This is **experimental** too: it has not been
tried against a real Microsoft account.

### Structure, typed as Markdown

The message body is a live editor. Headings, lists, quotes, bold, italic, code
and links are on the Format menu and on keys, and every one of them can be
typed instead, without leaving the sentence. Type the marker, then a space,
then keep writing. The space is what turns the marker into structure, and a
marker with no space after it stays as it was typed. Each marker says what it
made, so you hear "Heading level 2" or "Bulleted list" after the space, and
`Ctrl+Z` puts the characters back if you meant them literally.

| Type this | What it makes | What is said |
|-----------|---------------|--------------|
| One number sign, then a space, at the start of a line | A level 1 heading | Heading level 1 |
| Two number signs, then a space, at the start of a line | A level 2 heading | Heading level 2 |
| Three number signs, then a space, at the start of a line | A level 3 heading | Heading level 3 |
| A hyphen or a star, then a space, at the start of a line | A bulleted list | Bulleted list |
| A number, a dot, then a space, at the start of a line | A numbered list | Numbered list |
| A greater-than sign, then a space, at the start of a line | A quote | Quote |
| Two stars or two underscores on each side of the words | Bold | Bold, at the closing star |
| One star or one underscore on each side of the words | Italic | Italic, at the closing star |
| A backtick on each side of the words | Code | Code, at the closing backtick |
| The words in square brackets, then the address in round brackets | A link on those words | "Link to" and the address at the closing bracket, or that the address is not one this can link to and the words are left as they are |

A marker counts at the start of any line: the first line of a new message, a
line of the quoted text in a reply, a line after Enter and a line after
Shift+Enter. It only counts when it is the whole line so far, so a sentence
that ends in a hyphen stays a sentence. The word after a closing star,
underscore or backtick is plain again. Until 2026-09-20 a marker on a line
after a line break, which is every line of a reply to a message that arrived
as plain text and every line after Shift+Enter, was left as typed with nothing
said; that is fixed, and a marker the editor met and would not turn into
structure is now written to the log, so a report can say which line it was on.

### Saving Drafts

- Click **Save Draft** button or press `Ctrl+S`
- The draft will be saved to your Drafts folder
- You can return to edit it later
- Files you attached and any formatting are kept with it. If a file has been
  moved or deleted by the time you reopen the draft, it says which one
- Whether Sign and Encrypt were ticked is kept with it too, so a draft you
  reopen goes out the way you meant it to

### Signing and encrypting what you send

Two check boxes sit beside Send and Schedule: **Sign (experimental)**,
`Alt+G`, and **Encrypt (experimental)**, `Alt+Y`. Both keys work from
anywhere in the composer, the message included, and from inside the message
the new state is read out, "Sign on" or "Encrypt off". Neither box is ever
greyed out; when one cannot be honoured, Send says why.

Signing needs a certificate or a PGP key of your own for the address the
message goes from. Encrypting needs that too, because a message is always
encrypted to you as well, so the copy in Sent is one you can open. It also
needs something kept here for every person the message goes to: a
certificate, which Wixen Mail keeps when that person sends you S/MIME signed
mail, or a PGP public key, which you import in File, PGP Keys.

**Which kind is used.** S/MIME when you have a certificate for your address
and, to encrypt, every recipient has a certificate kept here. OpenPGP
otherwise, when you have a PGP key and every recipient has a public key. The
sentence after the message goes says which, for example "Message sent, from
the Outbox, signed and encrypted with OpenPGP".

**When it cannot be done.** Send, `Ctrl+Enter` and `Alt+N` check the boxes
before anything is put in the Outbox. When a box cannot be honoured the
reason is read out and shown, the composer stays open with your message as it
was, and nothing is sent:

| What you hear | What to do |
|---|---|
| You have no certificate or PGP key for (your address) that can sign, so this message cannot be signed. | Import your PGP private key in File, PGP Keys, or install your certificate in Windows. |
| There is no certificate or PGP key here for (an address), so this message cannot be encrypted to that address. | Ask that person for signed mail or their public key, or send without Encrypt. |
| No one kind of encryption reaches everybody this message is to. | Some recipients have only a certificate and others only a PGP key. Send them separate messages. |
| An encrypted message names everybody it is encrypted to, so a blind copy would not stay blind. | Move the Bcc addresses to Cc, or send them a message of their own. |

Each of these ends "Nothing was sent."

**A locked PGP key.** When your key is locked with a passphrase, Send asks for
it before the message is signed. It is remembered until Wixen Mail closes and
never saved. Cancel leaves the composer open with nothing sent.

**Later on.** A message waits in the Outbox for the Undo Send hold, or until
the time you scheduled. The check is made again when it goes, and if a key or
certificate has been removed in between, the message stays in the Outbox with
the reason instead of going out in the clear.

**What is sent, and what Sent holds.** The copy filed in Sent is the message
exactly as it went, signed or encrypted, and it opens here because you are one
of the people it is encrypted to.

This is experimental. No message signed or encrypted here has yet been opened
by Outlook, Thunderbird, Apple Mail or Proton Mail, so another program may say
a signature does not hold, or may not open a message, when Wixen Mail says it
went.

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

### Signatures

Your signatures are one list, whichever account you were looking at when you
wrote each one. **Tools → Signatures** opens the Signature Manager, which lists
them all. Its **Used by** column names the accounts each signature is given to,
and says "everyone else" on the default.

Each email account uses one signature, and you can choose it in either of two
places. Both change the same setting, so what you choose in one is what the
other shows the next time you open it.

- **On the account.** In the Account Manager, edit the account. On the first
  page, **Signature for this account** (`Alt+F`) lists the signatures. Its first
  entry is the default, written as "Use the default:" and the default's name, or
  "Use the default (none is set)" when there is no default.
- **On the signature.** In the Signature Manager, add or edit a signature. Under
  **Use for these accounts** (`Alt+U`) there is a check box for each email
  account, ticked for the accounts that use this signature. A box says which
  signature that account uses now, if it uses another. Ticking it moves the
  account to this signature.

One signature can be the default. It is used for every account you have not
given a signature. Tick **Default signature** (`Alt+D`) on a signature to make
it the default. Untick it on the default and there is no default, so an account
you have not given a signature sends mail with none.

A new message starts with the signature of the account in its **From** line.
If you change From to another account's address, the signature changes to the
new account's, and you hear "Signature changed to" and its name. An other
address signs with its account's signature, so moving between an account's
addresses changes nothing. If you have typed into the
signature, it stays as you left it and nothing is said, so you never lose what
you wrote. A reply or forward keeps the signature above the quoted message.

Mail from an earlier build keeps every account's signature. The first time this
build opens your mail, each account that had a default signature of its own is
given that signature, and the oldest of those defaults becomes the default for
everyone.

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

### Saved Searches

A saved search keeps a question about your mail under a name, and the folder
tree shows it under **Saved Searches** in the account you made it in. There
are two ways to make one.

To keep a search you have just run, choose **Edit → Save This Search** and
give it a name.

To make one from nothing:

1. Choose **Action → Saved Searches → New Saved Search**, or **Edit → Save
   This Search** with nothing searched. The New Saved Search window opens.
2. Type a name in **Name for this search** (`Alt+N`).
3. In **Look in** (`Alt+L`), choose everywhere in the account you are in, or
   one of its folders.
4. Press `Enter`. The conditions window opens, with no conditions yet.
5. Press `Alt+A` to add a condition, such as the subject containing
   "invoice". Add as many as you need.
6. In **Find messages that match** (`Alt+M`), choose **every condition** or
   **any condition**.
7. Press `Alt+C` to close the window. The search is saved and Wixen Mail says
   so, such as "Invoices saved. It is in the folder tree under Saved Searches."

Nothing is saved until the conditions window closes with at least one
condition. Pressing `Esc` in either window before then makes no search.

To change a saved search later, put the cursor on its row and choose **Action
→ Saved Searches → Edit Conditions**. The same conditions window opens, with
the search's own answer to every or any. Changing that answer is saved like
any other change, so a search made to match any condition can be made to
match every condition.

A saved search reads only the mail on this computer, and its list shows the
newest 500 messages it finds.

### Arranging Saved Searches

Saved searches sit in the folder tree under **Saved Searches**, in a branch for
each account. To move one, put the cursor on it and press `Alt+Shift+Up` or
`Alt+Shift+Down`, or choose Move this search up or Move this search down from
its menu; it moves among its own account's searches and Wixen Mail says where
it now is, such as "Invoices, 2 of 4." Searches saved before this version keep
the order they were made in until you move them, and a new search goes at the
end of its account's list.

`Alt+4` to `Alt+9` run the first six saved searches of the account you are in,
in that order, and put the cursor on the search's row, as `Enter` on the row
does. The account's searches, up to fifty, are also on the Saved Searches
submenu of the Action menu, each of the first six with its key beside it.

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
having to identify a glyph. The preview pane does not list them. Open the
message with `Enter` to see them listed below the message body, and press
`Alt+A` from inside the open message to jump straight to the list, in the
formatted view and in the plain-text reader alike; `Alt+A` again goes back to
the message. Until
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

### Undo in the other modules

`Ctrl+Z` in a module's list takes back the last thing you did to an item
there, and `Ctrl+Y` does it again. With the list focused, the Edit menu names
it, such as "Undo Delete: Dentist".

| What you did | What Undo does |
| --- | --- |
| Mark Done or Not Done | Puts the task or reminder back the way it was |
| Pin or Unpin | Puts the note back the way it was |
| Move to | Moves it back to the calendar, list, folder, group or account it came from |
| Copy to | Takes away the copy, after asking, and leaves the one you copied |
| Delete | Brings the item back |

A deleted item comes back in one of two ways, and Undo says which:

- **As it was.** Your account has not been told about the delete yet, so the
  item comes back with everything it had and the delete is never sent. This
  is the usual case when you undo straight away, or when the network is off.
- **As a new item.** Your account has already deleted it, so there is nothing
  there to put back. The item is made again here from what it held, and your
  account receives it as a new one on the next sync, with a new identity
  there.

While that account is syncing, Undo changes nothing: it says the item is being
synced and asks you to try again shortly, rather than racing the sync.

Undo lasts until your next action, on anything, with no time limit. There is
one step for the whole window, so undoing in Tasks after marking a message
read says the last thing you did was in Mail. Undoing at your account is
experimental: nothing in these modules has been tried against a real account
yet.

### The contact editor

The Basic Info tab asks for a name in this order: Name, Prefix, Given name,
Middle name, Family name and Suffix, then Nickname, Company, Department, Job
Title, Birthday, Website, Relationship, Avatar URL and Favorite. Prefix and
Suffix offer the common titles and endings, such as Dr., Mrs, Jr. and PhD, and
take anything you type instead.

The whole name and its parts fill each other as a first guess. Type
"Grace Brewster Murray Hopper" in Name and the parts fill in as Grace, Brewster
Murray and Hopper; fill in the parts with Name empty and Name is written from
them. A box you typed in is never written over, and neither is one a saved
contact opened with, so correcting a name does not undo a family name you set
by hand. Empty a box and it can be guessed again.

The birthday is three controls, month, day and year, in the order your
computer writes a date. Tick Birthday to give a contact one; with it unticked,
the contact has none. Tick No year for a birthday whose year you do not know,
and the year is left out.

An email address is checked when you add it. One that is not the shape of an
address, such as "grace.example.com", is refused with a sentence naming it,
and the dialog stays open so you can correct it.

A phone number is read against its own country's numbering plan. Type it with
`+` and its country code and its country is found from the code, whatever
Country says; type it the way the country writes it at home and it is read as
a number of the country chosen in Country. Country opens on the country
Windows says you are in, and its entries are Windows' own country names,
each with its code. A number the check accepts is saved in the international
form, grouped the way its country writes it, so "0121 234 5678" with the
United Kingdom chosen is saved as "+44 121 234 5678". A number the check
doubts, because it looks too short, too long or not a number that country
uses, is not refused: a sentence says what looks wrong, and pressing OK again
without changing anything keeps it exactly as you typed it. The only text
refused is text with no digit in it at all.

Google, Outlook and address book servers each receive all five parts of a
name, each in the field that service keeps for it.

### Times in events and reminders

Times move in blocks of 15 minutes, 30 minutes or 1 hour. The block is set by
**New events last** in the Calendar section of the Calendar and PIM tab in
Settings, and it is 30 minutes until you change it. A change there applies to
the next event or reminder you open, with no restart.

A new event starts at the next whole block after the moment you open it and
lasts one block. With 30 minutes, an event opened at 2:37 starts at 3:00 and
ends at 3:30, not at 2:30. Opened at 11:50 at night with 15 minutes, it starts
at midnight on the next day. A new reminder is set for the next whole block in
the same way.

On a time's minutes:

| Key | What it does |
|---|---|
| `Up` | Moves the time forward one block, or to the next whole block when it sits between two: 2:37 becomes 3:00, and 3:00 becomes 3:30 |
| `Down` | Moves the time back the same way: 2:37 becomes 2:30, and 2:30 becomes 2:00 |
| `Right` | Moves the time forward one minute |
| `Left` | Moves the time back one minute |

The hour keeps its own step of one hour. You can still type over the hour or
the minutes; the arrow keys take the place of moving through the digits, so
`Shift` or `Ctrl` with `Left` and `Right` is what still selects and moves
within them. A time moved past midnight moves its date with it.

An event's end moves with its start by the same amount, so moving a 3:00 to
3:30 event to 4:00 makes it end at 4:30. Once you change the end yourself, by
a key or by typing, it stays where you put it. Choosing a time with **Put this
time in the event** sets both.

A task has a due date and no time, so none of this changes the task window.

### Time zones

Every time in the calendar is on your computer's clock: the list, what `Space`
says, the full reading, the printed page, the Calendar window's Date/Time
column, the header's range of dates and the alert when a meeting is about to
start. A meeting keeps the time zone it was written in, and Wixen Mail works
out the hour it is where you are each time it shows or says it.

Outlook is the case where this matters most. Microsoft sends every Outlook
event in universal time (UTC), so a meeting at ten in the morning in New York
in summer arrives as 14:00. It is shown, said and alerted at 10:00 on a
computer in New York, and at 15:00 on one in London.

The full reading and the printed page also say the clock the meeting was
written on, once, when that clock differs from yours and names a place. A
meeting set for nine in the morning on 5 March in Tokyo, read on a computer in
New York with dates written day first, says "04/03/2026 at 19:00 to 20:00,
which is 05/03/2026 at 09:00 to 10:00 Tokyo time". The rows, the short reading
`Space` gives and the alerts say your time only, so a calendar that is syncing
does not read out a second clock on every line. Nothing is added for universal
time, which says how a server sent the time rather than where anybody is.

The event editor shows your clock too. Opened in New York, that meeting from
Tokyo shows 19:00 on the 4th. When you change the time, Wixen Mail saves it in
the meeting's own time zone, so moving it to 20:00 here reaches Outlook or
Google as 10:00 on the 5th in Tokyo. Opening a meeting and saving it without
changing anything changes nothing. A time you type in the hour the clocks skip
in spring, such as 2:30 on the morning they go forward, is saved at the first
quarter hour after the skip, 3:00.

These keep the hour as it was written:

| Case | What you see |
|---|---|
| An event you made here | The hour you typed, on your clock, as before |
| An event that lasts all day | Its day, wherever you are |
| A time zone this computer cannot place, such as one an organiser built by hand in Outlook | The hour as it was written; the full reading says the zone could not be placed |

Near midnight, a meeting from another time zone can sit in the day it was
written on in the Day and Week views while saying the day it is here. A
meeting at five in the morning on the 5th in Tokyo, seen in New York, is listed
under the 5th and says 15:00 on the 4th. The row's own words are right; the
view it sits in follows the organiser's day. This is known and not yet
changed.

### Finding a time everyone is free

**Find when everyone is free**, in the event window under Who is coming, asks
when the people on the guest list are free and answers in sentences. It is
experimental: no answer from a real account has been read yet, only answers
written in its tests.

**What is asked, and where.** Every place the account keeps a calendar is asked
at the same time: each calendar server it signs in to, Microsoft's service and
Google's. The question names the guest list's addresses and the window of dates,
and nothing about the meeting. What the places say about one guest is put
together, so busy time any of them knows about is kept.

**The answer.** The first sentence gives up to three times that suit everybody,
the most useful first; the rest are in Times offered. A time outside somebody's
working day, or one they have pencilled something into, is still offered, and a
sentence after it says whose.

**A guest nobody could check** is never counted as free. The answer names them
and gives the reason, which is one of these:

| The answer says | What it means |
|---|---|
| there is no calendar to ask | The account keeps no calendar anywhere that can be asked, its calendar server does not answer this question, or the guest's address could not be sent |
| the server would not say | A place was asked and refused, could not be reached, or passed over this guest. Asking again may work |
| the reply could not be read | An answer came back that Wixen Mail could not make sense of |
| their calendar is not shared with you | Google could not find this guest's calendar for you, usually because they are outside your organisation |

**Where each guest is.** A time is judged against each guest's working day in
their own time zone. The working day is the one set in the Working Day section
of the Calendar and PIM tab in Settings. Microsoft says which time zone a
colleague keeps their working hours in, so a colleague on Outlook or Office 365
is judged on their own clock. For anybody else nobody said where they are, so
their day is judged on your clock and the answer says so: "Nobody said where Bo
is, so the times were judged against the working hours set here."

**Each time on their clock.** Where a guest's clock says a different hour from
yours, each time says theirs too, for up to three guests in the order they were
invited, worded the way your settings word dates and times. Asked in March
from London about a colleague in Karachi, with dates written day first, a time
reads "03/03/2026 at 10:00, which is 15:00 for Ada". When it is already
another day there, the date is said too: "03/03/2026 at 23:00, which is
04/03/2026 at 04:00 for Ada". A guest whose clock agrees with yours adds
nothing. Once times carry a clock they are kept apart by semicolons.

A time zone an organiser built by hand in Outlook cannot be placed, so that
guest is treated as one nobody said the place of.

## Keyboard Shortcuts

### Application Control
- `Ctrl+Q` - Quit application
- `Ctrl+,` - Open settings
- `F1` - Help documentation
- `Esc` - Close dialogs

### Editing
- `Ctrl+Z` - Undo a step in the box you are typing in, such as a note or the
  contacts search. Each box remembers up to 100 steps, and a step is a word
  you typed with the space after it, a paste, a cut, or a run of deleting.
  Choosing another note starts its boxes afresh. With nothing to undo it
  says so. Boxes in dialogs keep the same steps: the composer's address and
  subject lines, the account settings, the contact editor and every other
  box you type in. What a dialog opens holding is where Undo stops, and in a
  dialog, with nothing left to undo, the key says so in the same words as the
  main window
- `Ctrl+Y` - Redo, putting back the steps Undo took, one at a time. Once you
  type something new, there is nothing to redo, and it says so

A few boxes keep Windows' own single step, where a second Undo puts the
change back: number fields such as the minutes between checks for mail, and
the Describe the picture and Insert Link boxes in the composer. A password box
keeps no steps at all, so your password is never held in memory as a list of
the ways you typed it.
- `Ctrl+Shift+Z` - Undo Send, while a message you just sent is still being
  held. It is third on the Edit menu, after Undo and Redo, and its letter is
  N. Some programs use `Ctrl+Shift+Z` for Redo; here Redo is `Ctrl+Y`

The Edit menu greys Undo and Redo when there is nothing for them to do, and
a screen reader says they are unavailable.

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
- `Ctrl+Shift+S` - Star or unstar the selected messages. Until 2026-09-20 this
  line said `S`, which has never been bound
- `M` - Mark as read or as unread, and hear which
- `Ctrl+Z` in the message list - Undo the last mark as read or unread, star,
  label, delete, move or copy, each message back the way it was. The Edit
  menu names it, such as "Undo Mark as Read: Quarterly report" or "Undo Move
  to Archive: Invoice", and `Ctrl+Y` does it again. It lasts until your next
  action on messages, with no time limit, and the change goes to the mail
  server the way the action did. What a move, a delete or a copy can and
  cannot take back is under "Moving, deleting and copying happen here first"
- `Space` - Read the message aloud

### Navigation
- `Ctrl+U` - Next unread message. Until 2026-09-20 this line said `N`, which
  has never been bound
- `Ctrl+Shift+U` - Previous unread message. Until 2026-09-20 this line said
  `P`, which has never been bound
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

- Press `F1` to open the help page for whatever is open. **Help → Contents**
  lists every topic.
- Open one topic directly from the Help menu: Getting started, Keyboard
  shortcuts, Using Wixen Mail, Setting up a provider, When something goes
  wrong, Accessibility, Privacy, and What changed.
- **Help → Send Feedback**, or `Ctrl+Shift+F`, sends a report about a problem, an
  idea or a question; see [Report Issues](#report-issues).
- **Help → Check for Updates** asks whether a newer version has been published.
- **Help → About** says which version you have and who makes Wixen Mail.

Until 2026-09-23 this list named **Help → Documentation** and **Help → Keyboard Shortcuts**,
which the menu does not carry by those names; they are Contents and the Keyboard shortcuts topic.

### About Wixen Mail

**Help → About** shows who holds the copyright: Pratik Patel and the Wixen
Project, with other contributors, under the MIT licence. It shows the whole
version with its build number, such as `1.0.0-alpha.1+114.g44bff634`: the
version, then how many commits the build is past the point that version was
set, then the commit it was made from. A later build has the larger number,
so quote the whole string when you report something. After the copyright come
two links, wixen.app and wixen.app/support, then a Send Feedback button, and
then OK. OK has the focus when About opens, so Enter closes it. Press
`Shift+Tab` once to reach Send Feedback, which closes About and opens Send
Feedback (`Alt+F` does the same), and again to reach the links; Enter on a link
opens that page in your browser.

### Provider-Specific Help

When you need an app password, the Add Account dialog says so next to the
password box, and the account dialog points you to
[Setting up your provider](PROVIDER_SETUP.md), which has the full steps for
Gmail, Outlook.com and Office 365, Yahoo, iCloud, and ProtonMail Bridge.

### Report Issues

Use **Help → Send Feedback**, `Ctrl+Shift+F`, or the Send Feedback button on
About. One window asks what you need, shows you the message, and sends it from
your own account.

1. Choose what it is about in the first box: Report a problem, Request a
   feature, Something is hard to use with a screen reader, Ask a question,
   Report a security concern, or Something else. The questions below change
   with your choice.
2. Answer the question. A problem asks two: what you were doing and what you
   heard or saw, and what you expected instead. Only the first needs an answer.
3. Choose what goes with it. Five boxes each say what they send: the version,
   your Windows version and language, the screen reader running, the end of
   the log, and the kinds of account you have. The version and the log
   are ticked to start with. In the log, every address and every subject is
   hidden.
4. Check "How to reach you". It starts with your account's address and is used
   only to answer you; leave it empty if you do not want an answer.
5. Read "What will be sent" (`Alt+B`). It holds the whole message, who it goes
   to and from, and the attached log, and it changes as you type.
6. Press Send (`Alt+S`). The report goes into your Outbox as an ordinary message
   from your default account, or from the account you are using when none is
   marked as the default, and is held and sent like any message you write. It
   goes to support@wixen.app.

A copy of every report you send is kept in the `feedback` folder inside the
`logs` folder where Wixen Mail keeps its files, with its log excerpt beside it.

**A security concern works differently in three ways.** It goes to
security@wixen.app rather than the support address. Its log starts unticked,
because a log can hold the very thing the concern is about and the hiding
covers addresses and subjects only; tick it if the concern needs it, and read
it in the window first. And its subject says only "Report a security concern",
because a subject shows in message lists and notifications. GitHub's private
reporting page is offered beside Send, as the other way to report one.

**When Send cannot be used**, because no account is set up or because sending is
turned off under Settings, Allow Changes, the line under the buttons says which.
Copy to clipboard (`Alt+C`) puts the whole message on the clipboard, and the
GitHub button (`Alt+G`) opens the issue page, or for a security concern the
private reporting page, where you can paste it.

Until 2026-09-23 this section said the program had no Send Feedback command yet
and that About would offer one when it arrived; both are here now.

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
Wixen Mail is reachable that way. Folders, stars, and labels keep a growing
mailbox organised, and search finds a specific message faster than scrolling
to it.

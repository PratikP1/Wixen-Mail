# Testing Wixen Mail

Thank you for trying this. It is an alpha, which here means something specific:
large parts of it have never been run against a real mail account by anybody.
This page says which parts, so you can decide what to point it at.

## The short version

**Reading your mail is the part that has been used.** Signing in, listing
folders, fetching messages, reading them, searching, threading. Since the
build after 2026-09-20 a folder you have never set shows one row per
conversation, on a new profile and on one from before that day; a folder you
set yourself through View, Thread View keeps your choice. If you want a folder
flat, `Ctrl+T` in that folder does it, and Show conversations by default under
Settings, Reading turns the default off for every folder you never set. Nobody
has yet heard a folder never set announce conversations on a fresh profile.

**Between 2026-09-18 and 2026-09-20 the message list and the reader changed in
most of the ways the first three days of testing asked for, and none of it has
been heard by anybody yet.** Moving through the list marks nothing read; a
message counts as read after you read the whole of it from the list, `Space`
twice or `Shift+Space`, or open it with `Enter`, and then after the delay the
setting names. `M` marks the message you are on read or unread, whichever it is
not, and says which; the command says which way it will go on the Action menu,
the context menu and the toolbar. `Shift+Down` and `Ctrl+A` select more than
one message, and Delete, Move to, Copy to, Mark as Read, Star and the label keys
act on every selected message with one sentence saying how many. A conversation
row stands for the message that started it when nothing in it has been read and
for the first unread message otherwise; that sender is said first and the
preview shows that message. `Ctrl+Shift+;` reads the row you are on column by
column with each heading, once, on request. Folders to Keep Up to Date is on the
Tools menu, as a tree with a check box beside each folder. Pictures a message
points at are shown, except tracking pixels and pictures the sender marked
decorative; an undescribed picture is passed over unless Settings says image or
photo. `Delete` says the one word and the row you land on is the confirmation,
and a move, a delete or a copy happens on this computer first with the server
told afterwards. `Tab` into the list lands on a row. A row's snippet is the
message's first real words. An address written out in a message is a link, and
where a link opens is a setting. Text a newsletter hid is not read and its
layout tables are not tables. A rule can say a phrase first. A Markdown marker
typed with its space counts on any line of a message you write. A setting saved
in Settings applies without a restart, apart from the two that say so under
their controls. The log starts at Debug while the version says alpha, so a
report comes with the lines it needs. Every one of those was proved by a test
over a built window, a fixture or a stand-in server, and the list further down
says, one thing at a time, what nobody has heard.

**Since the build of 2026-09-18, your mail comes down whole and keeps coming,
and none of that has met a real account yet.** After every check for mail,
every message of every folder you keep up to date comes down on its own, for
every enabled IMAP account, and the text of each message with it unless the
Message Text box on the Permissions tab is off or you chose a size there. A
check is what `F9` runs and what happens when the server says something
arrived, so the download starts on its own from the first check after you
install this build. Pause Downloading on the Tools menu holds it. Each
account's inbox is watched for as long as the program runs, the watch is
started again after a wait when it ends, and every account is checked on the
interval its own editor sets. Until that build, 500 messages came down per
folder, the text of a message came down when you opened it, and the watch
ended the first time the connection dropped. Every part of the new behaviour
has been driven against a stand-in written for the tests and none of it
against a mail provider; the account you point it at is the first one it
meets. The list further down says what nobody has seen it do.

**Between 2026-09-22 and 2026-09-24 the editors, About and the ways to reach
us changed, and none of it has been heard by anybody yet.** Every number in
Settings and in the account editor is a spin control: `Up` and `Down` step it,
it stops at the ends of its range, and the field you type in has the same name
as its arrows. The contact editor has Prefix, Middle name and Suffix; a whole
name typed in Name fills the parts, and parts typed with Name empty fill Name,
never writing over a box you typed in. A contact's birthday is a month, a day
and a year with a No year box, and a phone number is read against its
country's numbering plan and saved in the international form. A new event or
reminder starts at the next block of 15, 30 or 60 minutes, `Up` and `Down` on
the minutes move by the block and `Left` and `Right` by a minute, and an
event's end follows its start. Each email account has its own signature, with
one default for the rest, chosen on the account or in the Signature Manager.
The Label submenu lists your own labels with the key that applies each, and
you put them in order in the Label Manager. About names who holds the copyright and
links to wixen.app and wixen.app/support. Help, Send Feedback, `Ctrl+Shift+F`,
sends a report you have read from your own account. A link can open in a
separate Wixen Mail window with nothing in it but the page. Every sentence the
status bar shows was rewritten to one shape, and a command with nothing chosen
says one sentence, such as "Choose a message first.", in every window. Coming
back to the window that shows a conversation as headings puts the keyboard in
the message. Each of these was proved by tests on a built window, and the list
further down says what only your ear, your account or a mailbox can settle.

**Everything that writes is experimental.** Sending a message, deleting one,
moving or copying one to another folder, marking one read on the server, filing
a copy of what you send in Sent, changing which folders you are subscribed to,
and sending changes to your tasks, contacts or calendar back to Google or
Microsoft. None of that has run against a real account. Expect bugs.

Wixen Mail splits that answer in two, under a setting called Allow Changes.
Mail starts switched off: a message that has been sent cannot be recalled.
Changing your tasks, contacts and calendar starts switched on: those changes
are sent to Google or Microsoft, and a task in the wrong place can be moved
back. The next section says what each answer covers and how to change it, but
read this first.

## What Wixen Mail is allowed to change

There are two separate permissions, because getting them wrong costs different
amounts.

| | What it covers | Default |
|---|---|---|
| Mail | Sending, deleting, moving, copying, marking read on the server, filing a copy in Sent, sending read receipts, changing subscriptions | **Off** |
| Tasks, contacts and calendar | Sending your changes back to Google or Microsoft | On |

A message that has been sent cannot be recalled, and a message deleted from a
server may have been the only copy. A task in the wrong place can be moved
back. That is the whole reason they are separate.

Change them in Settings, under Allow Changes. That answer covers every account
you have signed in.

One account can be allowed less than that. Open the account from the accounts
window, go to its connection page, and under "Allow Changes for this account"
there are three boxes: mail, tasks, contacts and calendar, and fetching the
text of a message that is not already stored. Untick one and it stops for
this account only, so a real account can stay read-only while a throwaway one
is tried against a server. A box here can only ever allow less than Settings
allows, never more: where Settings has an answer off for every account, the
box is unavailable and says which heading in Settings to go to. One thing to
know: when a box here is holding a change back, the sync still says "turn on
Allow Changes in Settings", because that sentence does not yet know which of
the two places is holding it. If Settings already has it on, look here.

To use a real account with nothing at risk at all, start Wixen Mail with
`--read-only`, which is next.

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
3. **Signing in.** Especially with anything that is not Gmail or Outlook.
4. **Anything that claims to have worked and did not.** A status line saying a
   message was sent when it was not, a count that does not match what you can
   see, a setting that does not take effect.
5. **Keyboard traps.** Anywhere Tab or Shift+Tab cannot get you out of.
6. **Whether the ticks are announced in Folders to Keep Up to Date.** Since
   2026-09-18 the window is a tree, on the Tools menu, with a check box beside
   each folder that belongs to the tree control itself, so what a screen reader
   says for it comes from Windows and not from anything this program adds. It
   is worth a specific listen: arrow down the tree and say whether you hear
   "checked" and "not checked" as well as the folder name, whether it changes
   when you press Space, and whether a folder inside another is read with its
   level. Until 2026-09-18 this item described a list whose rows reported their
   own state through an object this program wrote, and asked for the same
   listen; the listen happened on 2026-09-17 and the answer was "read-only, not
   checked", which is why the window changed.
7. **Whether the folder list matches what you see in your webmail.** New in
   this version, and the part most likely to differ between one mail service and
   another. On Gmail in particular: whether your labels are all there, whether
   anything appears twice, and whether the count beside a folder matches what
   the website shows.
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
11. **What your provider does with the download, and whether mail keeps
   arriving over a day.** New in the build of 2026-09-18. The first check
   after you install it starts bringing every kept folder down, and a
   provider is entitled to slow that down, refuse it or disconnect you; if
   yours does, the status bar says the download stopped and when it will be
   tried again, and that sentence is worth quoting. Then leave the program
   running for a day and say whether mail turned up on its own the whole
   time, whether it turned up again after your network went and came back,
   and what the status bar said while you waited.
12. **How much is said while mail is fetched.** The Feedback tab in Settings
   has a new choice, "While mail and the other modules are fetched, say", with
   three answers. Nobody has listened to any of them. Under the default, Say
   what arrived, a check should say nothing until it ends and then one
   sentence naming each folder that received something; say whether anything
   was spoken on the way that should not have been, and whether the one
   sentence reads as an ending.
13. **A walk through a folder with unread messages.** Since the build after
   2026-09-18 moving through the list marks nothing, and the first `Space` on
   a row marks nothing either. Arrow through an unread folder letting your
   screen reader finish every row, and say whether the unread count is where
   it was; then `Space` twice on one message and say whether it is marked
   after the delay and not before.
14. **A selection of many.** `Shift+Down` a few rows, then Mark as Read or
   Delete. Say what your screen reader said as the selection grew, whether one
   sentence with the count came after the command, and after Delete, whether
   you heard "Delete" once and then the row you landed on and nothing else.
15. **A shown picture.** Open a newsletter in the preview. Say whether its
   pictures are there, whether the line about tracking pixels was heard once
   at the top, what a picture with no description was read as, and whether a
   picture inside a link read as the link's words.
16. **A contact typed whole.** Make a contact and type a full name in Name,
   such as Grace Brewster Murray Hopper. Say whether you heard the parts fill
   in, whether the tab reads Prefix, Given name, Middle name, Family name and
   Suffix in that order, and whether a box you had typed in was left alone.
17. **A report sent from a real account.** Send one report with Help, Send
   Feedback, read What will be sent first, and say whether it left your Outbox
   and whether it came back as undeliverable, which it may until public
   testing begins.
18. **An event's time moved by a block.** Open a new event, press `Up` and
   `Down` on the start's minutes, then `Left` and `Right`, and say what was
   spoken after each key and whether you could tell the end had moved with
   the start.

## What is already known to be missing or unproven

Written down so you do not spend time reporting things already on the list.

- **Nothing that writes has run against a real account.** Sending, deleting,
  moving, copying, filing a copy in Sent, sending a read receipt, changing
  which folders you are subscribed to, and the three syncs that push changes.
- **The download of everything has never met a real provider.** Since the
  build of 2026-09-18, every check for mail ends by bringing down every
  message of every folder you keep up to date, five hundred headers at a time
  with the folder you are looking at first, then the text of each message
  fifty at a time, for every enabled IMAP account. It picks up where it was
  after a restart, because what it knows is what is already on this computer.
  Get Older Messages, `Shift+F9`, puts the folder you are in at the front of
  the queue, and Pause Downloading on the Tools menu holds the whole thing
  between chunks for the rest of the session. Download This Whole Folder and
  Fetch Missing Message Text are gone, because this is what both did.

  Every part of it has been driven against a stand-in written for the tests,
  so we know the program asks for the right things in the right order and we
  do not know what a real provider says back. What we do not know, one thing
  at a time: whether a provider tolerates a whole mailbox coming down chunk
  after chunk, whether it slows the download, refuses it or disconnects you
  part way, and what it does after that. If it does refuse, the program stops
  asking, says so on the status bar, waits thirty seconds, then a minute, then
  two, doubling to half an hour, and asks again, for as long as the program
  runs. Whether those waits suit your provider is a guess until somebody has
  watched one.

  Two things to expect. The first download of a large mailbox is a long run,
  and the cache on this computer grows with your mailbox while it happens.
  Gmail's All Mail and the Spam folder are not kept up to date unless you
  tick them in Folders to Keep Up to Date, so they do not come down, and
  neither does any folder you chose not to keep.
- **Mail keeps arriving on its own, and no real server has dropped the watch
  yet.** Since the same build, each enabled account's inbox is watched through
  an open connection for as long as the program runs. When the watch ends for
  any reason but mail arriving, it is started again after the same growing
  wait; when your network comes back it is started again at once, and the
  accounts that are due are checked at once, without pressing Go Back Online.
  Where a watch cannot cover, the account is checked on a schedule: a POP
  account, a server that does not offer watching or refuses it three times in
  a row, and the folders you keep up to date that are not the inbox. The
  interval is the Check Interval on the account's own editor, which has been
  there since 2026-03-01 and did nothing until this build: between 1 and 60
  minutes, 5 unless you changed it. Starting the program checks every enabled
  account, so the first hours are no different from the hours after the first
  `F9`.

  What nobody has seen: whether your provider drops the watch, after how long,
  and whether the restart carries mail in over hours; whether it counts a
  watch per account against a connection limit, since each account now holds
  two connections, one watching and one working. A connection that dies
  without saying so can go unnoticed for up to 29 minutes, which is how often
  the watch renews itself; the scheduled check is what covers that. The
  schedule does not run while this computer has no network. An account whose
  password is not saved on this computer is told so out loud at every check,
  at the start and then every interval, until the password is entered again
  or the account is disabled; that was seen by running this build against
  such an account, and it is an error said as errors are. The status bar says
  which of watching, waiting to watch again, and checking every so often is
  happening, and nobody has heard those three lines.
- **How much is said while mail is fetched is a choice, and nobody has
  listened to the three answers.** At the end of the Feedback tab in Settings,
  "While mail and the other modules are fetched, say" offers Say what arrived,
  Say every step and Errors only, and starts on Say what arrived. Each step of
  a check or a download is shown on the status bar under every answer and
  spoken only under Say every step; what arrived is said once at the end,
  folder by folder with its count, and never when nothing arrived; an error
  is said whatever you chose. Which sentence is a step and which a result was
  decided by reading each line's words, and a line sorted wrongly is silent
  under the default or spoken under it. Say which you hear.
- **Nothing the message list and the reader gained between 2026-09-18 and
  2026-09-20 has been heard.** Each of the following was proved by a test on a
  built window, a fixture or a stand-in server and by nobody's ear, and each
  names what only your ear or your account can settle.
- **The folder chooser's check boxes.** Folders to Keep Up to Date is a tree
  on the Tools menu with the tree control's own check boxes, so what a screen
  reader says for one comes from Windows. Nobody has heard whether a kept
  folder is "checked", whether Space says the new state, whether a nested
  folder's level is read, whether the title is heard as the account's name, and
  whether the sentence under the tree about Gmail's All Mail is reached.
- **The log at Debug.** The lines a report needs are written at Info and
  Debug, and nobody has yet written a report from a log at that level or seen
  what a day at Debug on a real account costs on the disk; the two size rows
  the page below owes are still owed.
- **Alt+A to the attachments from inside a message.** The landing sentence
  "Attachments, N", "Message" on the way back, F7 to the warning bar and the
  two "No attachments" and "No warning" answers have not been heard, in the
  formatted view or in the plain-text reader.
- **Read state on a walk.** Whether the unread count stays put while you arrow
  through a folder, stays put on the first `Space`, and moves after the second
  `Space`, `Shift+Space` or `Enter` and then the delay, is your ear's; so is the
  sentence under Mark as read after being read once.
- **Mark as Read's label and `M`.** The item heard as Mark as Unread on a read
  message and Mark as Read on an unread one, on the Action menu, the context
  menu and the toolbar; `M` heard as one word, "read" or "unread", with the
  list staying on the row; none of it has been heard.
- **Where the cursor lands after a delete.** Whether the next message's row is
  read once and not again when the folder is re-read a moment later, and the
  previous row after deleting the last, has not been heard; a delete could not
  be driven on this machine while the tester's copy was open.
- **A delete's one word.** "Delete" once at the key and nothing after it when
  the delete went through, the refusal heard with its reason when it failed,
  Move to Trash and Move to Folder the same, and "Copied to" still heard for a
  copy, have not been heard.
- **Tab into the list.** Whether the row is read once on arrival by Tab, F6 or
  a click, whether coming back to the list is quiet, and whether "No messages"
  is heard once for an empty folder, are your ear's.
- **Selecting more than one message.** Your screen reader's own "selected" and
  "not selected" as the selection grows and shrinks, the count after `Ctrl+A`,
  one sentence after a command over many, "Delete" once and the row after the
  set, and the refusal above 5,000 messages, have not been heard.
- **A move that completes here first.** The row leaving at once, Enter on the
  chosen folder in the Move dialog moving the message, a move made offline put
  back with the refusal spoken when the server says no, and a move replayed at
  the next check after a restart, have met no real server; the same for a move
  or a copy to a folder on another account, which has met no pair of servers,
  and what a real destination does with a message it already holds is unknown.
- **Which message a thread row is.** The sender said first on a conversation
  row, the preview showing that message, Enter opening the conversation window
  on it, and the conversation's text arriving from Gmail when you land on the
  row, are your ear's and your account's.
- **Gmail's own conversations.** Whether your split threads become one
  conversation row after the next check, with the count matching Gmail's own
  client, and what one fetch of the thread names for every stored message costs
  on your mailbox, has not met a Gmail account.
- **The row's columns on request.** `Ctrl+Shift+;` heard whole and once, each
  heading then its text, and the list quiet under an NVDA configuration
  profile with Row/column headers off, which nobody has set up; what Narrator
  and JAWS need is unread.
- **Attachment said once.** Whether landing on a message with an attachment is
  heard as the row with the attachment tone beside it and nothing spoken for
  the event, and whether a fresh profile hears every event's tone from the
  start, has not been heard.
- **The sounds after hours.** The earcons stopping after hours open was
  reported and could not be reproduced on purpose here; whether they now come
  back after the output device changes or a headset is unplugged needs a hand
  on the machine's Sound settings under a running build, and if they stop again
  the log at Debug holds the moment.
- **The snippet.** Whether a row now says the message's first sentence rather
  than an address or "View this email in your browser", and whether the rows of
  mail downloaded before this build are put right at the next start, is your
  ear's.
- **A rule's phrase said first.** "Urgent, Unread, ..." on a matching row, the
  Says first column, the Rule matched sound once after a check with several
  matches, and the Labels column read as part of the row, have not been heard.
- **Addresses written out as links.** The chapter address of a plain-text
  alert in your screen reader's link list, an address in a description read as
  "link to" its site, and a sender's `sms:` link heard with "link not opened
  here" after it, have not been heard.
- **Pictures shown by default.** A newsletter's pictures in the preview, the
  tracking-pixel line heard once at the top, a linked picture read as the
  link's words, an undescribed picture passed over or called image or photo,
  and a decorative one passed over, are your reader's.
- **What the formatted view leaves out.** A newsletter read once under NVDA,
  with no run of symbols where the hidden padding was, no table announced
  around a layout block, no grouping name, and the subject and the sender each
  heard once, has not been heard.
- **Where a link opens.** Enter on a link going to the browser, or to the
  message view with "Opening" and the site's name and the page's title said
  once, and "Back to the message" on Backspace, have not been heard. Nor has
  the separate Wixen Mail window, which arrived on 2026-09-22 and is a copy of
  the program holding one page: its title, a link on the page opening in the
  same window, and Escape closing it have been driven by nothing but a test.
- **A setting saved applies at once.** Mark as read after changed in Settings
  and a message marked after the new wait without a restart, the dates in the
  list changing at once, and the two sentences under Log level and Default sort
  order heard after each control's name, have not been heard.
- **Show conversations by default.** A folder never set heard as conversation
  rows on a fresh profile, the setting turned off and the next folder heard
  flat without a restart, and the check box heard by name and state after Then
  by, are your ear's.
- **All Inboxes' own view.** All Inboxes heard as conversations on a fresh
  profile, `Ctrl+T` there and the view kept when you come back, a conversation
  in two of your accounts heard as two rows with a command on one reaching that
  account, and the refusal on a label or a saved search, have not been heard.
- **Markdown typed into a message.** "Heading level 2" heard after the marker
  and its space on the first line, on a line of the quoted text in a reply,
  after Enter on the empty first line and after Shift+Enter, and the word after
  a closing star read plain, have not been heard; the steps are on the
  listening page.
- **Nothing the editors, About and Send Feedback gained between 2026-09-22 and
  2026-09-24 has been heard.** Each of the following was proved by a test on a
  built window or a reading of the code, some by the accessibility scan of the
  running program as well, and by nobody's ear; each names what only your ear,
  your account or a mailbox can settle. The separate window is the entry on
  where a link opens, above.
- **The status bar read on its own.** Every sentence on it now has one shape,
  and nobody has read the bar by itself with NVDA+End to hear whether a step
  and an answer are still told apart, whether "Choose an account first." and
  "Choose a message first." read as the same kind of sentence, and whether
  "The mail on this computer is not open." is clearer than what it replaced.
- **Coming back to a conversation shown as headings.** Switching to another
  program and back, then pressing K or H at once, should find the keyboard in
  the message; a test sends the window the message Windows sends, and only
  the NVDA run on a pull request has made a real switch. Nobody has heard it.
- **About.** The copyright line and the licence sentence read on opening, each
  link heard as a link with its address, and Enter on a link opening your
  browser once and leaving About open, have not been heard. Both pages
  answered 522 when they were last fetched, on 2026-09-23, and are due up by
  public testing.
- **Send Feedback.** The categories and the questions that follow them, the
  five boxes with the log excerpt ticked, the excerpt unticking for a
  security concern, What will be sent read line by line, and Send's sentence
  have not been heard. No report from a real account has reached
  support@wixen.app or security@wixen.app.
- **Spin controls.** Check Interval said once with its name and its sentence,
  the value spoken after `Up` and `Down`, and Mark as read after's three
  entries with the seconds unavailable until a wait is chosen, have not been
  heard.
- **The contact editor.** The Basic Info tab in its new order, the parts
  heard filling after a whole name, the Birthday and No year boxes, an address
  refused in a sentence naming it, the Country entries and a doubted number's
  sentence have not been heard. No real Google account, Outlook account or
  address book server has received a prefix, a middle name or a suffix.
- **Times in blocks.** The time spoken after each arrow key on an event's or a
  reminder's minutes, whether the end moving with the start is heard, and the
  New events last list have not been heard.
- **Signatures per account.** The Signature Manager's Used by column, the
  account's Signature for this account choice, the Use for these accounts
  boxes and "Signature changed to" on a From change have not been heard.
- **Labels.** The Label submenu's items with their keys, Edit Labels at its
  end, the Label Manager's Key column, a move said as "Later, 2 of 5." and
  "There is no label 6" have not been heard.
- **Notes can now go to a calendar server, and no build has ever sent one to a
  real server.** If you added a calendar by its address, that same server is
  where your notes for that account now go, under the same sign-in. Settings
  says so, on the Calendar and PIM tab, and says it is experimental.

  Every part of it has been driven against a stand-in written for the tests, so
  we know the program asks for the right things in the right order and we do
  not know what a real server says back. What we do not know, one thing at a
  time: whether a server accepts the document this writes at all, whether its
  own listing names a note the way this reads it, whether the version marker it
  gives survives a round trip, whether it takes a deletion, and whether it
  reports a clash the way the stand-in does.

  If you try it on notes you care about, keep a copy. Turning Allow Changes off
  for that account stops anything leaving this computer while you look.

  One thing that does change on the way: **a note's line endings.** The format
  calendar servers exchange has one way of writing a line break, so a note you
  typed on Windows comes back with plain line breaks rather than Windows ones.
  Nothing else about the text changes, and notes kept on this computer are not
  affected at all.

- **Notes on an Outlook or Office 365 account now go to OneNote, and no build
  has ever opened a real notebook.** Each section of your notebooks becomes a
  note folder here, named by where it sits, so a section called Q3 inside a
  section group called Projects inside a notebook called Work shows as
  "Work / Projects / Q3". Settings says so on the Calendar and PIM tab and says
  it is experimental.

  Your sections arrive at the first sync rather than before it, so the Notes
  list looks empty until you press Sync now once. That is because the list of
  sections is Microsoft's answer and has to be asked for.

  Everything has been driven against a server written for the tests, so we know
  what this program sends and we do not know what Microsoft says back. What we
  do not know, one thing at a time: whether Microsoft accepts the page this
  builds, whether a page you make here looks like a note when you open it in
  OneNote, whether changing a page works the way the reference describes,
  whether the permission can be granted on a personal account without an
  administrator, and whether the sections of a shared or a class notebook can
  be read at all.

  **A note loses things on the way, and this is not a bug we can fix.** A
  OneNote page has no way to hold bold, italic, struck-out text, a quotation,
  code, or a line across the page. A note carrying any of those comes back
  without it, the sync tells you how many notes that happened to, and the copy
  kept here becomes the copy OneNote kept. Headings, lists including nested
  ones, tables, links and pictures all survive. A picture comes back with its
  description and with Microsoft's address for it rather than the one it went
  out with, because OneNote stores the picture itself.

  Struck-out text is the one to watch. A job crossed off and a job still to do
  read the same afterwards.

  If you try it on notes you care about, keep a copy. Turning Allow Changes off
  for that account stops anything leaving this computer while you look.

  **Signing in again is needed for an account set up before version 0.111.0.**
  Permission is granted once at sign-in, and the permission this needs is new.
  Open the account, switch the browser sign-in off and back on, and approve the
  list the browser shows. Until you do, every notes sync on that account says
  you need to sign in again.

- **Contacts can now come from an address book on a server, and no build has
  ever reached a real one.** Tools, "Add an Address Book by Address", asks for
  the address and the sign-in, asks the server which address books it has, and
  lets you choose one. Its contacts then sync both ways with the same merge
  Google and Outlook contacts already use, so a contact that changed in two
  places is held for you to choose rather than written over. The screen says it
  is experimental before you type a password, and Settings says the same on the
  Permissions tab.

  Every part of it has been driven against a stand-in written for the tests, so
  we know the program asks for the right things in the right order and we do
  not know what a real server says back. What we do not know, one thing at a
  time: whether a real server's answer about its address books parses, whether
  its answer with the cards in one parses, whether a card this writes is
  accepted, whether the version marker it gives survives a round trip, and
  whether it reports a clash the way the stand-in does.

  If you try it on contacts you care about, keep a copy. Turning Allow Changes
  off for that account stops anything leaving this computer while you look.

  **The card-matching limitation below is more serious for an address book on a
  server than it was for a file.** A card that names an address one of your
  contacts already uses is read as being about that person, and a card can now
  arrive from a server rather than from a file you chose. Nothing about the
  matching rule has changed.

- **Moving a task to another list is the one thing on this page that could lose
  a task.** Press the menu key on a task and choose "Move to another list", or
  press `Ctrl+Shift+V`. Events and notes have the same command.

  This used to work only on tasks you had made here, and refused any task that
  came from Google or Microsoft. It no longer refuses them.

  Here is what happens when you move one. Wixen Mail asks Google to make the
  task again in the new list, and then, once that has worked, asks it to remove
  the task from the old list. That order is on purpose. If something goes wrong
  between the two steps, you are left with the task in **both** lists, which you
  can see and tidy up. The other order would risk leaving it in neither, which
  you could not see at all. On this computer the task is in exactly one list the
  whole time, whatever happens.

  **We have never done this with a real Google or Microsoft account.** Nobody
  has, with this program. Every test so far has used a stand-in that says yes to
  everything, so we know the program asks for the right things in the right
  order, and we do not know what Google says back.

  So if you try it on an account you care about, check your task lists
  afterwards, in a browser or on your phone:

  | What you see | What it means |
  |---|---|
  | One copy, in the new list | It worked |
  | The task in both lists | The second step has not happened yet. The next sync should tidy it |
  | Still in the old list, and nowhere else | Nothing was sent, and nothing will be. See below |
  | The task in neither list | Tell us straight away. This is the failure the whole design exists to prevent |

  Moving a task takes two syncs to finish, so seeing it in both lists for a
  short while is normal.

  The third row happens when you move the task into a list you made on this
  computer, rather than into one that came from Google. Lists you make here stay
  here, so Google is never told about them, and it is never asked to create the
  new copy or to remove the old one. In Wixen Mail the task has moved and shows
  in the list you chose. In your browser nothing has changed at all.

  Nothing is lost, and nothing needs reporting. To move the task at Google as
  well, move it into one of the lists that came from Google.

  When you move a task that came from Google, Wixen Mail says "and has not
  reached the account yet", or tells you to turn on Allow Changes if that is
  what is stopping it. Nobody has heard that said aloud. Whether it helps, or
  becomes tiring after the twentieth move, is worth telling us.

  Moving an event to another calendar is still refused, and says so. It is the
  same idea for a different kind of thing, and it has not been built.
- **Moving a note to another folder can now lose a note, in the same way and
  for the same reasons.** Each place your notes go is its own note folder here:
  one for each calendar you added by its address, and one for each section of
  your OneNote notebooks. So moving a note between two of those folders is
  moving it between two places at the server, which is the same two steps as
  moving a task.

  Wixen Mail asks the server to make the note again in the new place, and then,
  once that has worked, asks it to remove the note from the old one. That order
  is on purpose. If something goes wrong between the two steps, you are left
  with the note in **both** folders, which you can see and tidy up. The other
  order would risk leaving it in neither, which you could not see at all. On
  this computer the note is in exactly one folder the whole time, whatever
  happens.

  **We have never done this with a real account.** Nobody has, with this
  program. Every test so far has used a stand-in that says yes to everything,
  so we know the program asks for the right things in the right order, and we
  do not know what a real server says back.

  So if you try it on an account you care about, check your notes afterwards,
  wherever else you read them:

  | What you see | What it means |
  |---|---|
  | One copy, in the new place | It worked |
  | The note in both places | The second step has not happened yet. The next sync should tidy it |
  | Still in the old place, and nowhere else | Nothing was sent, and nothing will be. See below |
  | The note in neither place | Tell us straight away. This is the failure the whole design exists to prevent |

  Moving a note takes two syncs to finish, so seeing it in both places for a
  short while is normal.

  The third row happens when you move the note into a folder you made on this
  computer, rather than into one that came from a server. Folders you make here
  stay here, so your server is never told about them, and it is never asked to
  create the new copy or to remove the old one. In Wixen Mail the note has
  moved and shows in the folder you chose. Wherever else you read your notes,
  nothing has changed at all.

  Nothing is lost, and nothing needs reporting. To move the note at the server
  as well, move it into one of the folders that came from it.

  Folders you made here sit under "On this computer" in the notes list, which
  is where the mail folder list puts the folders that are on no server. Nobody
  has heard that branch with a screen reader, and whether it is clear that the
  folders under it go nowhere is worth telling us.
- **Moving a contact or a reminder stays on this computer, by design.** A
  contact moves between groups and a reminder moves to another account, using
  the same two keys. Neither contact groups nor reminders are sent to Google or
  Microsoft at all, so nothing needs to be told about the move and there is
  nothing to go wrong at the far end. Nobody has heard either command with a
  screen reader.
- **Importing a file can join two people who share a name.** Two cards in one
  imported file are read as one person when nothing but their addresses tells
  them apart. Two people with the same name, and nothing else on their cards,
  come out as one contact holding both addresses, and that joined contact is
  then sent to your real address book. Turning Allow Changes off before the
  next sync holds it back.
- **Conversations on Gmail are worked out from the message headers**, so a
  conversation here may be split differently from the same one in Gmail's web
  interface. Gmail publishes its own grouping and the library this is built on
  gives no way to read it. **Until the build after 2026-09-19.** Since then a
  conversation on a Gmail account is the one Gmail shows: Gmail's own name for
  each message's conversation is asked for with the message and decides which
  messages belong together, whatever the headers say, and mail you already had
  gets its name once at the next check. The name was out of reach while the
  library's own reader stood between this program and the server's answer;
  once this program read the answer itself, asking for the name was a decision
  rather than a limit, and the sentence had outlived that. Nobody has yet seen
  split threads
  become one on a real Gmail account; on every other provider the headers still
  decide.
- **A folder can be created, renamed, moved, deleted, emptied, and marked read.**
  Renaming or moving the inbox is refused on purpose, because on a mail server
  that empties the inbox into a new folder rather than renaming it, and deleting
  the inbox is refused because a server does not allow it. Deleting a folder that
  has folders inside it is several commands with no way to undo half of it, so if
  it stops partway it says exactly where it got to. Emptying works the same way,
  and emptying does to every message what deleting one of them does, so emptying
  the inbox moves the mail to the trash and emptying the trash removes it. Two
  settings on the Reading page decide whether either command reaches the folders
  inside the one you chose; both start switched on.
- **A POP account has never been run against a real POP server.** Everything
  about it is new in this version: the client, the local folders, the sync, and
  the policy that removes mail from the server. Mail is left on the server
  unless you turn that off, which is the setting to be careful with.
- **Notes sync for two kinds of account and no others.** An account whose
  calendar you added by its address sends its notes to that same server, and an
  Outlook or Office 365 account sends them to OneNote. Everything else keeps
  its notes on this computer, Gmail included: Google Keep's interface is for
  Workspace accounts only, so a personal Gmail account cannot use it at all and
  that will not change. The entries higher up this page say what happens for
  the two that do sync.
- **The cached mail on this computer is not encrypted.** Anybody who can read
  your user folder can read your mail. Passwords and tokens are not in there,
  they are in the Windows credential store.
- **The installer is not signed**, so Windows will warn about it. Signing it
  would not stop that warning either, and nothing this project can buy will.
  [Installing and uninstalling](installing.md) has the keyboard steps for
  getting past the box, which are worth reading first: the button you land on
  is the one that cancels.
- **Updating itself will not work yet, and it is meant not to.** Wixen Mail can
  now fetch the installer for a newer version on its own, and it refuses to run
  anything this project did not sign. There is no signing key yet, so an update
  it fetches gets thrown away, and you are sent to the releases page to fetch
  the new version by hand. Do report it if that message is unclear or
  unhelpful, or if Wixen Mail ever offers to run the file anyway; that last one
  is a defect rather than a wording problem. Do not report the refusal itself.
  Choosing a kind of version under Settings, then General, then "New versions"
  is what turns the fetching on, and it begins switched off.

## How to report something

The quickest way is **Help → Send Feedback**, `Ctrl+Shift+F`, or the Send
Feedback button on About. Choose what it is about, answer the question or two
it asks, and read the message in "What will be sent" before you press Send. The
version and the end of the log go with it unless you untick them,
with every address and subject in the log hidden. Send puts the report in your
Outbox as a message from your default account, and a copy is kept in the
`feedback` folder inside `logs`. A security concern goes to its own address,
security@wixen.app, starts with the log unticked, and has GitHub's private
reporting page offered beside Send.

On 2026-09-23: until public testing begins, support@wixen.app and
security@wixen.app may not exist yet. A report sent to either shows as sent in
the Outbox and then comes back to your own inbox as undeliverable. Nothing is
lost: the copy in the `feedback` folder holds the whole report, and Copy to
clipboard and the GitHub page take it the rest of the way.

To write a report yourself instead, the rest of this section still applies.

Include what you did, what you expected, and what happened. If a screen reader
was involved, say which one and what it said.

Say which build. Since 2026-09-17 a build's name says how far along it is:
`1.0.0-alpha.1+114.g44bff634` is the version, then how many commits the build
is past the point that version was set, then the commit it was made from. The
name is in Apps and Features, in About, in `--version` and on the first line
of the log, and a later build has the larger number. The two builds handed
out before that date, `+g7d57cd49` and `+g59c5b6a4`, carry the commit alone.

Log files are in your Wixen Mail data folder, under `logs`. They do not contain
your passwords or the text of your messages. They may contain folder names and
email addresses, so read one before attaching it if that matters to you.

### What the log writes, and where the level is

The level is under Settings, then Advanced, then Log level. Each level writes
everything the levels above it write, and this much more:

| Level | What it adds |
|---|---|
| Error | What went wrong: a save that failed, a server that could not be reached. |
| Warn | What was refused or could not be done: a folder that would not open, a watch that could not start, a chunk of the download the server refused and why, the wait before the next try. |
| Info | What the program did: each check's result per folder, the watch's end and why, the download's finish per account, that the settings were saved and which log level and while-fetching level they hold, what was spoken, and how many characters were held back from speech because content is muted. |
| Debug | Each chunk of the download, headers and text, with the counts; a repeat that was not spoken. |
| Trace | Nothing more today. |

Since 2026-09-18 the level a fresh profile starts at follows the build: Debug
while the version says alpha or beta, Info from the release candidate on.
Under a testing round every report comes with a log, and the lines a report
needs are at Info and Debug, so the alpha starts where a report can be
written from. Nothing moves by hand at a cut. A profile that already holds a
level keeps it, whatever the build: the first tester's profile held Info when
it was read on 2026-09-18, so a build after that date leaves it at Info, and
Settings, then Advanced, then Log level is where to move it.

What Debug costs on disk is not measured yet, and this page does not guess.
The measurement is the harness in `tests/the_numbers_the_targets_ask_for.rs`
run twice, `WIXEN_MEASUREMENT_LOG_LEVEL=info cargo test --release --test
the_numbers_the_targets_ask_for -- --ignored --nocapture
test_cold_start_and_memory_with_a_thousand_cached_messages` and the same at
`debug`, each printing the size of the log after a two-minute start against
the measurement profile; the two rows go on `docs/development/measurements.md`
with their date. On 2026-09-18 the run was refused, because only one copy of
Wixen Mail runs at a time and the tester's copy was open on his account all
afternoon; a second start hands itself to the first and makes it say
"Wixen Mail is already running, and this is it", so the harness now looks
first and starts nothing while a copy is running. A day of real use writes
more than a two-minute start in any case, and how much more is something
only a day on a real account can say.

## Where your data is

Everything is in one folder, `%LOCALAPPDATA%\wixen-mail`: the cached mail, your
settings, and the logs. Paste that into File Explorer's address bar to open it.
Passwords and tokens are not in there, they are in the Windows credential store.

`--erase-all-data` removes all of it, including the saved passwords. The
uninstaller runs it for you. [Installing and uninstalling](installing.md) has the detail,
including what to copy if you want to keep your mail before uninstalling.

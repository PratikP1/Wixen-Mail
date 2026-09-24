# The manual accessibility pass

**None of this has happened.** This page is the list of what only a person can
check, written on 2026-09-14 so that somebody can work through it later, in
order, and know when they have finished. Nothing below is a result. Where a
line says "you should hear", nobody has heard it yet. The pass is planned for
after phase 8 of the project's roadmap, and this page is what that pass walks.

It is the other half of [what the scans can judge](wcag-coverage.md). That page
says, for each of the fifty-five WCAG 2.2 Level A and AA criteria, what the
automated checks can see and what is left for a person. This page says how a
person checks what is left. The two point at each other on purpose: a criterion
there names a walk here, and an item here names the row it came from.

## Before you start

### Which technology proves what

Windows has two accessibility channels, and a check made with one says nothing
about the other.

| Technology | What it reads | What a pass with it proves |
|---|---|---|
| NVDA | MSAA, through `IAccessible`, for native controls | The names this program sets with `set_accessible_name`, and the descriptions. This is the only channel those calls write to. |
| Narrator | UI Automation | The control's own window text, the static label beside a field, and anything Windows' own provider supplies. It never reads a name this code set on a native edit, button or list. |
| JAWS | Both, by its own rules | A spot check that the two channels agree where they should. |

They are not interchangeable here, and the project has the scars to show it.
Sixteen widgets were once "named" with a call that reached neither channel, and
the automated scan of the UI Automation tree passed them. The scan of 2026-09-14
found the reverse: the text field of every date and time spinner has its name
on the up-down arrows beside it, so NVDA meets a field with no name while
Narrator, in the Edit Event window, hears the label before it, which is a
different and shorter name than the one the code set. An item checked with NVDA
alone can be right for NVDA and wrong for Narrator, and the other way round.

So every item below says which it needs:

- **NVDA**: run it with NVDA, listening for the names and sentences this
  program set.
- **Narrator**: run it with Narrator, listening for what UI Automation gives.
- **Both**: run it twice, once with each, and note where they differ.
- **Eyes**: look, with no screen reader running, usually at both themes.
- **Keyboard**: unplug or ignore the mouse.
- **Ears**: listen for a sound and check its visible twin.
- **Tool**: a contrast analyser, Windows text scaling, or the display settings.

### What the machines already check, so this page does not ask for it

Three automated checks run in the repository's workflows. Their findings are
on the coverage page and not repeated here.

- **Axe.Windows** over UI Automation, on thirty-one windows. It can produce a
  finding against three criteria: 1.3.1, 2.1.1 and 4.1.2. A clean run says the
  structure is present; it says nothing about what is heard.
- **The MSAA walk** over the same thirty-one windows. It says whether every
  operated control has a non-empty name on the channel NVDA reads. It does not
  say the name is the right words.
- **The NVDA suite**, which is the only check that listens. Three tests run:
  - `account-manager-sign-in-failure`: NVDA announces "Signing in failed" when
    Sign In Again cannot reach a provider.
  - `calendar-immediate-actions`: NVDA hears Edit Event's, Delete Event's and
    Sync's own answers when each is pressed with nothing selected.
  - `filter-manager-delete`: NVDA announces the sentence Delete would show
    when nothing is selected.

  One test is written and skipped: `which-days-focus-and-tick`, which asks
  whether the dialog that asks which days you mean names its preselected
  answer as both focused and checked on open. The scan target it needed now
  exists; it stays skipped because it has never passed, and a test un-skipped
  before it has passed is a check nobody reads. That dialog is therefore still
  a person's, and it is item 25 under A below.

Nothing below asks for those three sentences again.

### Where each item came from

Every item names its source in brackets at the end:

- **[row N.N.N]** is a criterion row on the coverage page whose last column
  says a person has to judge it.
- **[ledger N]** is an entry in the project's defect ledger, `.planning/WINDOWS.md`,
  of the kind that records something built and never heard. There were 257
  open entries of that kind on 2026-09-14, most of them one sentence in one
  window; the items below group them into walks and name the entries each
  walk closes.
- **[guardrail]** is one of the six disability categories the project's own
  rules list, where neither a row nor a ledger entry had asked yet.

### What to write down

For each item: the date, the technology and its version, what was heard or
seen, and whether it matched what the item says it should. A line that did
not match is a defect and goes in the ledger with the item's number. A line
that matched closes the ledger entries the item names. Do not mark an item
done because the structure looked right in a tree; that is what the scans do,
and this page exists because structure present is not experience good.

## A. Blind: screen readers

Ninety-five items. Walk them in order; the first ones are the ground the rest
stand on. Items 42 and 43 were added on 2026-09-18 and 44 to 57 the same day,
for what phase 10 built; this line said forty-one until then. Items 58 to 83
were added on 2026-09-20 for what phase 11 built; this line said fifty-seven
until then. Items 84 to 95 were added on 2026-09-24 for what phase 12 built;
this line said eighty-three until then.

### The main window

1. **Start the program on a fresh profile.** You should hear the first-run
   question, "Before you start", its three choices with what each one costs,
   and the storage sentences about what stays on this computer. Answer it and
   land in the main window. Which pane you land in should be said. **Both.**
   [row 2.4.2, row 3.2.1, ledger 302]
2. **Move between the panes with F6 and Shift+F6.** Each arrival should name
   the pane: folders, messages, preview. Going back with Shift+F6 should be
   told apart from going forward, and Shift+F6 out of the preview should keep
   its direction. **Both.** [row 2.4.3, ledger 164, ledger 165]
3. **Arrow through the folder tree.** Account branches read as accounts, with
   the address only where two accounts share a name. The Favourites group
   reads as a group. A saved search reads as a search and not as a folder,
   three levels down. **NVDA.** [row 2.4.6, ledger 1, ledger 29, ledger 45]
4. **Open each of the six context menus on the folder tree** with the menu
   key. Every entry and its letter should be announced, and on a saved search
   the order of the entries should make sense by ear. **Both.** [ledger 27,
   ledger 44]
5. **Arrow down the message list.** Each row should read as its columns, in
   an order that makes sense, and the row cursor should not move when a
   message arrives or a conversation is rethreaded underneath you. A column
   of rows all saying "Message text not downloaded" should be bearable.
   **Both.** [row 2.2.2, ledger 6, ledger 74]
6. **Move from the inbox to Sent.** The columns change. Whether that is
   announced at all, and whether Restore Defaults in the Columns window says
   which folder's defaults it restored. **NVDA.** [ledger 167, ledger 168]
7. **Land on a saved search.** The working account changes when you do. You
   should be able to tell you have moved accounts by ear, and the coverage
   sentence and the "Running this saved search" line should arrive as two
   things, not one run-on. **NVDA.** [ledger 10, ledger 30]
8. **Search from the search box**, once finding something and once finding
   nothing. The count, the nothing-found signal and the coverage sentence
   should each be distinct and none should flood. **Both.** [ledger 33,
   ledger 34]
9. **Let the network go and come back.** The sentence about the network
   should be heard once and understood as a state, not an error. The status
   bar and the announcement are the same words; note whether meeting both
   reads as a repetition. The offer to go back online should be announced
   with its whole label, and you should be able to find your way back to it
   after letting it go by. **Both.** [row 4.1.3, ledger 76, ledger 77,
   ledger 78, ledger 79]

### Reading a message

10. **Open a message into the preview.** The rendered message keeps the
    sender's headings and link text: `H` should move between headings and a
    link should read as its text, not its address. This is the one surface
    the scan of 2026-09-14 never reached, so nothing about it has been judged
    by any check. **Both.** [row 1.3.1, row 2.4.4, ledger 428]
11. **Tab into the preview and out again.** It is a web view inside a native
    window, which is the likeliest keyboard trap in the program. **Keyboard,
    both.** [row 2.1.2]
12. **Open a message in the reading window.** The sentence said before the
    body, whether about encryption, a filter's verdict or a PGP signature,
    should arrive before the body and read as separate facts rather than one
    run-on. The signature sentence should sound like a disclaimer if it is
    one. **NVDA.** [ledger 101, ledger 102, ledger 103, ledger 142]
13. **Arrow through the attachment rows.** Each should read as name, kind,
    size and then the sender's description, in that order, and a picture
    with no description should be said to have none. **Both.** [row 1.1.1,
    ledger 89]
14. **Open a picture attachment in a reader tab.** Whether the picture is
    reachable at all, and whether its first lines are announced when the tab
    opens or only when you go looking. **NVDA.** [ledger 108, ledger 110]
15. **Open a message with pictures the sender marked decorative**, and one
    with thirty spacer images. The sentence about held-back pictures should
    be heard where it sits, above the body, and thirty decorative pictures
    should not flood. **NVDA.** [row 1.1.1, ledger 138, ledger 156]
16. **Open a message with a warning bar from two or three sources.** The
    sentences should be heard as separate facts. **NVDA.** [ledger 115]

### Writing a message

17. **Open the composer.** The message body is a web page inside the window,
    and as of version 0.123.1 its title is "Message body". Note what is said
    when focus enters it, before the first word of yours. **Both.** [row
    2.4.2]
18. **Mistype a word.** The engine's spelling mark should be announced by the
    screen reader itself; the program's own landing sentence and the earcon at
    the end of the word should not collide with it. Then walk the misspellings
    with the keys and take a suggestion: three suggestions should be helpful
    and "7 suggestions in all" should not be noise. **NVDA.** [ledger 120,
    ledger 121, ledger 122, ledger 123]
19. **Attach files three ways**: the picker, Ctrl+V on the attachments list,
    and a file dropped on the window. Six names in one announcement should
    read as a confirmation, not a list to sit through; Delete on the
    attachments list should say what was removed. Whether Ctrl+V is
    discoverable at all is the question, and the answer is probably no.
    **NVDA.** [ledger 126, ledger 128, ledger 129, ledger 134, ledger 172]
20. **Press F8 for the formatting toolbar**, apply a heading, a list and a
    quote. Each should say what it did and put the caret back in the message.
    **Both.** [row 4.1.3, ledger 171]
21. **Press Send.** "Sending in 10 seconds. Undo Send takes it back." should
    finish before the ten seconds do. Then Send Later: the schedule window's
    month list, day, year, hour and minute spinners and the morning-or-afternoon
    list should each be heard as a named control with its own value. The scan
    says the spinner text fields have no name on the channel NVDA reads; this
    item is where that is heard rather than measured. A refused time should
    keep the window open and say why. **Both.** [ledger 147, ledger 149,
    ledger 150; coverage rows 15 to 29 of the findings table]
22. **Answer a meeting invitation.** Accepting should be announced once, not
    twice. **NVDA.** [ledger 155]

### The other modules

23. **Switch to the calendar and arrow through a view.** The heading of the
    week view should be heard, Previous period and Next period should be told
    apart, and events should come in date order. "Changed just for this day"
    on a moved occurrence should make sense. **Both.** [row 2.4.6, ledger
    193, ledger 195, ledger 196, ledger 199]
24. **Open New Event.** Every date and time control: month list, day, year,
    hour, minute, AM or PM, for both Starts and Ends. Each should say its own
    name and its own value as you land on it, and the value as you change it.
    Compare what NVDA and Narrator say for the day field; on 2026-09-14 the
    tree said they would differ. **Both.** [coverage rows 15 to 22]
25. **Change one day of a repeating event.** The dialog that asks which days
    you mean should name its preselected answer as both focused and checked
    the moment it opens. This is the skipped NVDA test, walked by hand.
    **NVDA.** [row 4.1.2]
26. **Move and copy an item**, an event, a task, a note and a message, and a
    contact between groups. The move window's tree should read accounts as
    accounts and a folder inside its folder; Enter should choose. A copy
    should be told from a move by ear, "copied to" against "moved to". The two
    questions a contact's move asks should be told apart. The clause "and has
    not reached the account yet" should be understood. **Both.** [ledger 180,
    ledger 184, ledger 185, ledger 200, ledger 201, ledger 205, ledger 207,
    ledger 211, ledger 212, ledger 214, ledger 230]
27. **Read a note back as structure.** "Heading level 1, Shopping, bullet,
    milk" should be clearer to listen to than the flat text, a nested list
    and a table should be pleasant rather than merely correct, and the notes
    tree's "On this computer" branch should be met as a place. **NVDA.**
    [ledger 231, ledger 268, ledger 271, ledger 278]
28. **Let a reminder come due while you are typing.** It should be said and
    sounded at once while its window waits; the window should open a minute
    later whether or not you have stopped; the tone should come back once a
    minute until you reach it, ten times at most. Note whether the sentence is
    heard while another program is in front. Then a task due today, a task
    overdue and an event in fifteen minutes: each sentence should say its kind
    first. **Both, and ears.** [row 2.2.1, ledger 379, ledger 380, ledger 381,
    ledger 382, ledger 402]
29. **Run a sync of each kind with several going at once.** The calendar,
    task, contact and notes summaries should be told apart; a change waiting on
    a create should be told from one waiting on Allow Changes; the count of
    waiting conflict choices should be useful rather than a sentence you stop
    hearing. **NVDA.** [ledger 83, ledger 217, ledger 224, ledger 243]
30. **Open the conflict window** on two copies of a contact and on a note.
    The two copies should be understood as a labelled pair, and a note's
    question should be answerable without seeing both. **Both.** [ledger 82,
    ledger 244]

### Settings and the managers

31. **Walk the settings screen tab by tab.** Every section should be found
    where somebody would look for it, by moving through in order without
    skimming. The Notes section last on the Calendar and PIM tab; the
    three-valued combo box; the Say-where-a-picture-is-decorative box with
    its description. Arrow along the tab row, Right and then Left, and hear
    each tab once; then Tab into a page and back, and hear the row again
    without the page speaking twice. **Both.** [row 3.3.2, ledger 137,
    ledger 233, ledger 295, ledger 317, ledger 492]
32. **Open the Feedback tab's per-event panel.** Sixteen events in a picker;
    changing the picked event reloads three controls beneath the cursor, and
    whether that reads well is the question. The sentence saying speech
    against braille is chosen in the screen reader and not here should be
    heard once. **NVDA.** [ledger 345, ledger 346, ledger 347, ledger 348]
33. **Open the account window's Allow Changes boxes.** Three boxes, a heading
    and a note beneath; an unavailable box should say why and name the
    heading in Settings to go to. **Both.** [row 3.3.2, ledger 375]
34. **Open each manager**: accounts, contacts, filters, tags, signatures,
    blocked senders. The list should say what it holds, each row should read
    as its columns rather than one run-together string, and the status line
    under the buttons should be silent until it has something to say. After
    Delete or Unblock, what happened should be heard over the list being
    refilled. **Both.** [row 4.1.2, ledger 159, ledger 160, ledger 161,
    coverage rows 8 to 14]
35. **Open the filter rule editor.** The Match Field and Match Type lists
    should say their names; the eleven field words and eleven matching words
    should be understood ("Read is yes", "matches a text pattern"); the
    Pattern box, disabled for the four ways that read no pattern, should be
    skipped cleanly in the tab order. **NVDA.** [ledger 14, ledger 15,
    ledger 16]
36. **Open the saved-search condition editor.** The line saying what a search
    cannot find with the chosen field should be heard; an empty pattern should
    be refused out loud with focus back on the box; the tally at the end of
    every change should not become a clause you stop hearing. **NVDA.**
    [ledger 20, ledger 21, ledger 22, ledger 25, ledger 26]
37. **Block a sender**, once on a mailing list. The sentence saying what
    blocking will do and the mailing-list warning should be heard before the
    block is made, as two things. **NVDA.** [ledger 94, ledger 162]
38. **Add an address book and a calendar by address.** Every control should
    be usable by ear, including the password field and what is said about
    where the password goes. **Both.** [row 3.3.8, ledger 265]
39. **Open About and Help.** The About disclosure should read as information,
    not four paragraphs of apology, and help should be in the same place from
    every module. **Both.** [row 3.2.6, ledger 308, ledger 309]
40. **Import a PGP key, and check for an update.** Each of the five import
    answers, and the update answer, should be heard once at the moment it
    happens. **NVDA.** [ledger 146, ledger 316, ledger 337]
41. **Set the computer to another language and read a date.** A month name,
    a day name in a repeat sentence, and "2 days ago" in a list cell, in that
    language, inside an English sentence. This is 3.1.2, and nobody knows
    what a screen reader does with it. **Both.** [row 3.1.2, ledger 360,
    ledger 366, ledger 368, ledger 370, ledger 371]
42. **Check mail on an account with many folders, once under each answer to
    "While mail and the other modules are fetched, say" on the Feedback
    tab.** Under Say what arrived you should hear nothing while the folders
    are checked and then one sentence naming each folder that received
    something with its count, and the sound for new mail if that row plays
    one; under Say every step, every step and then the same sentence; under
    Errors only, nothing but the sound and "New mail" if that row says so.
    Note any sentence heard as a step under the default, and any result
    heard as a step under Say every step: which is which is a judgement by
    ear that nothing here has made. **NVDA.** [ledger 10, ledger 73, ledger
    521]
43. **Press OK in Settings while a check is running.** "Settings saved"
    should be heard, not replaced by the next line of the check. Then Draft
    saved and Refreshed the same way. **NVDA.** [ledger 78, ledger 521]

### All the mail, and what is said while it comes

Added 2026-09-18 for what phase 10 built. Everything in this group has been
proved by readings of the code and by a stand-in server inside the tests, and
none of it has been heard or met a real provider. The first check after
installing the build of 2026-09-18 is where items 49 to 56 happen on their
own, so read them before that check rather than after it.

44. **Arrow to a check box on each Settings tab after General**, Compose,
    Reading, Permissions, Calendar and PIM, Feedback, Advanced. Each should be
    heard as "check box" with its state, not as a button; Space should toggle
    it and the new state be said; Tab away and back should read the state
    again; OK should keep it. Then, from a control on the General tab, press
    Ctrl+Tab: a named control on the next page should be spoken and nothing
    before it, and note whether the page's name is said first. Every checkbox
    on the six pages answered "push button" in the build of 2026-09-17.
    **NVDA.** [ledger 513, ledger 514]
45. **Open a folder holding every message you have.** The whole folder should
    be one list, and End should land on the oldest message rather than on the
    five-hundredth; arrow to a smaller folder and back, and the keys should
    answer at once. A folder of 12,872 messages has been read back whole by a
    test and never opened in the running program. **NVDA.** [ledger 515]
46. **With All Inboxes open, choose View, Sort Messages, Oldest first**, arrow
    to a folder and back: the first row should be the oldest message. The same
    on a label and after running a saved search. Then Unread First on a folder,
    away and back: the unread rows should be first. **NVDA.** [ledger 516]
47. **Open the Permissions tab and Tab to "Keep the text of messages on this
    computer".** It should be heard as a combo box with that name and its
    answer, All of it unless you changed it; the sentence beneath it, saying
    what a size removes and when, should be read once and not with every
    answer. Choose a size, press OK, restart, and the size should still be
    chosen. **Both.** [ledger 519]
48. **Open the Feedback tab and press Alt+W.** "While mail and the other
    modules are fetched, say" should take focus as a combo box with its three
    answers in order, Say what arrived, Say every step, Errors only, and the
    sentence beneath it, that errors are always said, should be read once.
    **Both.** [ledger 521]
49. **Let the first download run, once under Say every step.** After a check
    you should hear "Downloading the mail that is not on this computer
    yet...", then a line per chunk, "Downloading Inbox: 500 of 12872
    messages.", "Inbox is downloaded: 12872 messages on this computer." as each
    folder becomes whole, and "Downloading message text: 50 of 12872
    messages." per chunk of text. Say whether a line per chunk is bearable at
    your mailbox's size, and whether the numbers are heard as a count. Under
    Say what arrived none of those should be heard, only one sentence when the
    account's download ends, "50 folders are downloaded, and the text of 12872
    messages is on this computer.", with the sound for new mail; say whether
    that sentence reads as an ending and whether the sound and the sentence
    arrive as one event. **NVDA, ears.** [ledger 523, ledger 11, ledger 72]
50. **Tools, Pause Downloading, during a download.** The item should be heard
    as a check item with its state and its description, the sentence saying
    the download has never met a real account. Ticking it should answer
    "Downloading is paused. Mail already here stays readable.", and unticking
    it "Downloading again.", each heard above the download's own lines. Then
    `Shift+F9` in a folder: "Downloading this folder first...", and, while
    paused, "Downloading is paused. Tools, Pause Downloading takes it off."
    **NVDA.** [ledger 523]
51. **If the download stops**, because your provider refused it or your
    network went, the status bar should say "Downloading Inbox stopped: the
    mail server stopped answering, and it refused. 3500 of 12872 are on this
    computer." and then "The mail server could not be reached. Trying again
    in 30 seconds." Both are steps, so under Say what arrived nothing is
    spoken about it and the download tries again on its own; say whether that
    silence is right or whether a stop should be heard once whatever was
    chosen. Nobody can make a provider do this on purpose, so note it if it
    happens. **NVDA.** [ledger 523, ledger 64]
52. **Run a saved search that reads message text before all the text is
    here.** Beside the coverage sentence you should hear "The text of 137
    messages in this account is not here yet and comes down on its own after
    the next check.", as two things and not one run-on, and no button should
    be offered. With the Message Text box off the sentence ends "so it stays
    on the server" instead. **NVDA.** [ledger 523, ledger 10]
53. **Read the status bar while nothing else is happening**, under Say every
    step. It should say one of "Watching Inbox for new mail. Checking every 5
    minutes.", "Waiting 2 minutes to watch Inbox again. Checking every 5
    minutes." or "Checking every 5 minutes.", and the three should be heard
    as three states rather than as three sentences that sound alike. With two
    accounts enabled the account should be named first, and `F9` should open
    with "Checking 2 accounts for new mail..."; with one account, neither name
    nor count. **NVDA.** [ledger 525]
54. **Open an account's editor and Tab to "Check Interval (min)".** The
    sentence saying what the interval does should be heard with the field and
    again as the text beneath it; say whether hearing it twice is too many.
    **Both.** [ledger 525]
55. **Leave the program running for a day.** Mail should keep arriving on its
    own the whole time, with the sound and the one sentence each time a check
    found something and nothing spoken when it found nothing. Unplug the
    network or turn the radio off for a minute and put it back: the network
    sentence, then the watch starting again and the due accounts checked, at
    once and without pressing Go Back Online. Say what the status bar said
    while the network was gone. **Ears, both.** [ledger 525, ledger 64,
    ledger 65, ledger 67]
56. **Disable an account whose password is not saved, or leave one that way
    for an hour.** Every check of it says the error out loud, at the start and
    then every interval, whatever was chosen on the Feedback tab. Say whether
    that is a flood or a reminder, and whether disabling the account stops it.
    **NVDA.** [ledger 525, ledger 526]
57. **Install the next build over this one and open Apps and Features.** The
    entry should read `1.0.0-alpha.1` followed by a number and a commit, the
    number larger than the build before it, and Windows should have installed
    it as an upgrade rather than beside the old one. **Both.** [ledger 517]

### Reading, and the list

Added 2026-09-20 for what phase 11 built. Everything in this group has been
proved by readings of the code, by tests on a built window and by a stand-in
server inside the tests, and none of it has been heard. Two of the phase's
plans are not in the build these items describe: the separate window a link
can open in, and the pass over the status bar's sentences, both put off to the
next phase on 2026-09-20. Corrected 2026-09-24: both arrived in phase 12, the
separate window on 2026-09-22 and the status bar's sentences on 2026-09-23, and
items 84 and 85 below are theirs.

58. **Open Tools, Folders to Keep Up to Date and arrow down the tree.** Each
    folder should be heard as a check box with its state, "checked" for a kept
    folder and "not checked" for the rest, never "read-only"; a folder inside
    another should be read with its level; Space should say the new state;
    the window's title should be the account's name; and on a Gmail account
    the sentence under the tree about All Mail should be reachable and make
    sense. **NVDA.** [ledger 533]
59. **Press Alt+A from inside an open message, in the formatted view and in
    the plain-text reader.** "Attachments, N" and focus on the list; Alt+A
    again, "Message" and focus back in the body; F7, "Security warning" and
    back; with nothing to go to, "No attachments" and "No warning". Say
    whether the list announcing itself after the sentence is one thing too
    many. **NVDA.** [ledger 538]
60. **Arrow through a folder with unread messages, letting the screen reader
    finish every row.** The unread count should be where it was. Then Space
    once on an unread message: the count should still not move, however long
    you stay. Space again, the whole message: after two seconds with the row
    still selected, the count should move. Shift+Space and Enter should do the
    same; moving off before the delay should leave it unread. On the Reading
    tab, the sentence under Mark as read after should be read once, on the
    choice's own row. **NVDA.** [ledger 539]
61. **Arrow onto a read message and open the Action menu, then onto an unread
    one.** The item should be heard as Mark as Unread and then as Mark as
    Read, the context menu's entry the same, and the toolbar button's name the
    same under the screen reader's own navigation. Press M on a message: one
    word, "read" or "unread", and the list staying on the same row. Alt+A then
    E should reach the item whichever way it goes. **NVDA.** [ledger 540]
62. **Delete a message in the middle of the list, then the last one.** After
    the first, the next message's row read once and not again when the folder
    is re-read a moment later; after the last, the previous row read once; the
    same after Move to Trash and after a move out of the folder; the preview
    showing the message you landed on. **NVDA.** [ledger 541]
63. **Listen to what a delete says.** "Delete" once at the key and nothing
    after it when it went through; with the network off, the refusal heard
    with its reason; Move to Trash and Move to Folder the same; Copy to
    Folder's "Copied to" still heard, since its row stays; and the fuller line,
    "Deleting" then "Moved to Trash", on the status bar with NVDA+End and not
    otherwise. **NVDA.** [ledger 542]
64. **Open a folder, then Tab from the folder tree into the message list.**
    The newest message's row read once and not twice; F6 into the list the
    same; back to the tree and Tab again landing on the row you left, with
    nothing moved; an empty folder's list saying "No messages" once. **NVDA.**
    [ledger 543]
65. **Grow a selection with Shift+Down and shrink it with Shift+Up, then
    Ctrl+A.** "selected" for each row added and the row read, "not selected"
    for the row that leaves, and the count on its own after Ctrl+A. Then Mark
    as Read over three: "3 messages marked read" once and nothing per message.
    Delete over several: "Delete" once and the row after the set read once
    when they have gone. M on a conversation row: "1 conversation, 5 messages
    marked read". Select more than 5,000 and press a command: the refusal
    heard once with the count. **NVDA.** [ledger 544]
66. **Move a message to another folder in the same account.** The row should
    leave at once and the cursor be read on the next message; in the Move
    dialog, Enter on the chosen folder should be the move. With the network
    off, move a message, put the network back and check for mail: if the
    server says no, the message should come back and the refusal be heard
    once. Move a message with the network off, close the program, start it
    again and check: the message should be in the destination and not brought
    back. Copy a message: "Copied to Work" once, the row staying. What a real
    server does with a replayed move is the account's. **NVDA, and a real
    account.** [ledger 546]
67. **Move a message to a folder on another account.** The row leaving at
    once, the cursor read on the next message, "Moved to Work in Home" on the
    status bar, and the message in that folder of the other account at its
    next check; with the network off, the row going and coming back with the
    refusal spoken once when a server says no; a restart with the crossing
    unfinished finishing it at the next check of either account without a
    question; a message over 25 MB saying it goes now; a copy across accounts
    heard once. **NVDA, and two real accounts.** [ledger 547]
68. **Land on a conversation row in Thread View.** The sender heard first
    should be the message that started the conversation when nothing in it is
    read, and the first unread message otherwise; the preview should show that
    message; Enter should open the conversation window with the cursor on it,
    so Enter again opens it; Space should read it; M should mark the whole
    thread; a conversation with everything read should give the oldest. On a
    Gmail account, landing on the row should bring the conversation's text
    down without a word. **NVDA, and a Gmail account.** [ledger 548]
69. **On a Gmail account whose threads showed as several rows, check for mail
    once and look again.** The split threads should be one conversation row,
    with the count matching what Gmail's own client shows; Enter should open
    the same messages Gmail shows; a reply that arrived without its headers
    should sit in its thread; two unrelated threads with one subject should
    stay two rows. Note what the once-only pass took on your mailbox. **A Gmail
    account.** [ledger 549]
70. **Press Ctrl+Shift+; on a row, then choose Read the Row's Headings and
    Text from the Action menu.** The row heard whole and once, each heading
    then its text in the order the columns are shown, a conversation row
    reading its own cells, and the refusal heard when the list does not have
    focus. Then set up the NVDA configuration profile the shortcuts page
    describes, with Row/column headers off for this program, and arrow: the
    rows should be quiet of headings. What Narrator and JAWS need is unread.
    **NVDA.** [ledger 550, ledger 551]
71. **Land on a message with an attachment.** The row read once, with "Has
    attachment" inside the row's reading, the attachment tone beside it and
    nothing spoken for the event; with "Show events in the status bar" off,
    the tone alone. On a fresh profile every event's tone should be heard from
    the start. **NVDA, ears.** [ledger 552]
72. **Change the output device while the program runs, under a build started
    against an empty data folder.** With earcons on, change the default device
    in Windows Sound settings and trigger an event with a sound, Settings
    saved being the nearest: say whether it went silent and what the log said
    at debug. Then unplug a headset with a sound due, and say whether the sound
    after it plays. The steps are in the ledger entry; never the installed
    build, and never against your own profile. **Ears.** [ledger 553]
73. **Arrow onto a row whose message opens with a web address, a newsletter's
    row, and a reply's row.** The first heard as the message's first sentence
    and not the address; the second as its first real line and not "View this
    email in your browser"; the third as the new words and not the quote. After
    the first start on this build, the rows of mail downloaded before it should
    read the new way, and the log should say how many were put right and in how
    long. **NVDA.** [ledger 554]
74. **Make a rule with Say this first and the phrase Urgent, and one with the
    sound box ticked.** A matching row heard as "Urgent, Unread, ..." with the
    phrase before the first column, and as "Urgent" alone when the first cell
    is empty; the Says first column showing the word once switched on with F8;
    Ctrl+Shift+; saying the phrase without "Says first" before it; the Rule
    matched tone once after a check that found several matches across folders,
    and its words "Rule matched, 3 messages"; the Labels column, switched on,
    read as part of the row, and on a conversation row every label once.
    **NVDA, ears.** [ledger 556]
75. **Open a plain-text message with an address on a line of its own, the
    chapter alert that was reported.** The address in NVDA's link list on the
    preview and in the reader window, and Enter on it going where Open links
    says; an address in an event's description read with Space heard as "link
    to" its site; a note's bare address a link when the note is shown as a
    page; an address with a full stop after it linking without the stop; a
    sender's mailto link on a name still working; a sender's sms link heard as
    its words with "link not opened here: sms" after them. **NVDA.** [ledger
    557]
76. **Open a newsletter in the preview, on a fresh profile and on your own.**
    Its pictures shown; a one-pixel beacon passed over with "1 picture that
    looked like a tracking pixel was not fetched." heard once at the top; a
    linked picture with no description heard as the link's words; an
    undescribed picture heard as nothing under the default and as "image" or
    "photo" once chosen on the Reading tab; a picture the sender marked
    decorative passed over, or heard as such with that box on; a note with an
    undescribed picture read with Space and the picture passed over. **NVDA.**
    [ledger 558]
77. **Read the Substack newsletter that was reported, in the formatted view.**
    The subtitle heard once, where the sender's own line stands and not at the
    top; no run of symbols where the padding was; no "table with N rows and N
    columns" around any block; no "Post header" grouping announced; the subject
    heard once as the heading and the sender once in the heading with no number
    before it; then another newsletter of your choosing read the same way.
    **NVDA.** [ledger 559]
78. **Press Enter on a link, on the preview pane and on the formatted message
    window, under each of the three answers to Open links.** The browser under
    the default; the page in the message view with "Opening" and the site's
    name, the title said once when it arrives, and "Back to the message" on
    Backspace or Alt+Left; the link's menu on the Applications key offering the
    three places on both surfaces; a page that will not load saying why; and,
    since 2026-09-22, the separate window choice opening a window of its own
    with "Opening" and the site's name said, the page's title after it, a link
    on the page staying in that window, Backspace going back and Escape closing
    it. **NVDA.** [ledger 560, 568]
79. **Change Mark as read after in Settings, press OK, and press Enter on an
    unread message.** The message marked after the new wait, without a restart;
    change how dates are written, OK, and the list's dates changing at once;
    the sentence under Default sort order and the sentence under Log level
    heard after each control's name. **NVDA.** [ledger 561]
80. **Open a folder you have never set, on a fresh profile.** Conversation rows,
    with the Thread View check mark on; turn Show conversations by default off
    under Settings, Reading, OK, and open the next folder never set: one row
    per message, without a restart; a folder set flat by hand still flat with
    the setting on; the check box heard by its name and state on the Reading
    tab after Then by. **NVDA.** [ledger 562]
81. **Land on All Inboxes from a fresh profile.** Conversation rows with the
    Thread View check mark on; Ctrl+T there heard as the flat view's sentence,
    a folder visited and All Inboxes returned to still flat with the check
    mark off; a conversation held in two of your accounts heard as two rows,
    one per account, each with its own count, and a command on one of them
    reaching that account; Ctrl+T on a label or a saved search heard as the
    refusal naming All Inboxes. **NVDA.** [ledger 563]
82. **Type Markdown into a new message and into a reply, with NVDA in focus
    mode and a moment's wait after each Space.** Two number signs, Space, a
    word on the first line: "Heading level 2", and Up then Down reading the
    line as a heading with the signs gone. In a reply to a plain-text message,
    Down into the quoted text, Home, the same marker: "Heading level 2", which
    is the case that was broken. The same after Enter on the empty first line
    and after Shift+Enter. Two signs, a word, Space, with no space after the
    signs: nothing, and the signs stay. Two stars, a word, two stars, a word:
    "Bold" at the closing star and the next word read plain; a hyphen, Space
    and a word on a new line: "Bulleted list". Ctrl+Enter to the preview, then
    H: land on each heading. **NVDA.** [ledger 565]
83. **After a day on this build, open the log and write one report from it.**
    The level starts at Debug under the alpha, and each check's line per
    folder, the download's chunk lines, the settings save line and the
    held-back line should be there; say whether they are the lines that make
    your next problem diagnosable, and what the day's log came to on the disk.
    **A person, with the log.** [ledger 535]

### The editors, and what the alpha still owes

Added 2026-09-24 for what phase 12 built. Everything in this group has been
proved by readings of the code, by tests on a built window and, for the spin
controls, the About dialog, Send Feedback and the contact editor, by the
accessibility scan of the running program on a pull request. None of it has
been heard, and nothing in it has sent mail or met a real address book.

84. **Set Open links to a separate Wixen Mail window under Settings, Reading,
    and press Enter on a link in a message.** A window holding only that page:
    its title said once when the page arrives and carried in the title bar; a
    link on the page opening in the same window; `Backspace` going back, and
    "This is the first page" when there is none; `Escape` and `F6` closing it;
    the sentence when the page will not load; and "Opening" with the site's
    name on the surface the link came from. **NVDA.** [ledger 568]
85. **Read the status bar on its own with NVDA+End while mail is checked and
    downloaded.** The watch's three state lines and the download's steps one
    after another, and whether a step and an answer are still told apart now
    that both may end in an ellipsis; a command with nothing chosen in the
    account manager and in the message list, heard as the same kind of
    sentence, "Choose an account first." and "Choose a message first."; and
    whether "The mail on this computer is not open." is clearer than the two
    wordings it replaced. **NVDA.** [ledger 574]
86. **Open a conversation in the formatted window, switch to another program
    and back with Alt+Tab, then press K or H at once.** The keyboard should be
    in the message and the key should move. Then the same after moving to the
    attachments list or the warning bar first: the keyboard should still be
    there when you come back. **NVDA.** [ledger 578]
87. **Open Help, About.** The copyright line and the licence sentence read when
    it opens; focus on OK; `Shift+Tab` reaching Send Feedback, then each link
    heard as a link with its address, wixen.app/support and then wixen.app;
    Enter on a link opening the page once in your browser and leaving About
    open. **NVDA.** [ledger 587]
88. **Press Ctrl+Shift+F.** The window opening on What is this about with its
    description read; the questions below following each category as you move
    through them; each of the five boxes read with its sentence and its state,
    the log excerpt ticked on opening; choosing Report a security concern
    unticking the excerpt, and whether you notice; What will be sent read line
    by line with the words Send queues; and Send's sentence when the report
    goes into the Outbox. **NVDA.** [ledger 589]
89. **Send one report from a real account, and one security report.** Each
    should leave the Outbox and arrive, one at support@wixen.app and one at
    security@wixen.app or the address Pratik names instead; until public
    testing either may come back as undeliverable, which is expected and worth
    saying. **A person, with an account and both mailboxes.** [ledger 590]
90. **Open the account editor's second page and Settings, Reading.** Check
    Interval said once, with its name and its sentence, and the new value
    spoken after `Up` and `Down`; Mark as read after's three entries heard, and
    the seconds beside them unavailable unless After a number of seconds is
    chosen. **NVDA.** [ledger 592]
91. **Make a contact and type a whole name, such as Grace Brewster Murray
    Hopper, in Name.** The Basic Info tab heard in its order, Name, Prefix,
    Given name, Middle name, Family name, Suffix; the parts heard filling; the
    Birthday and No year check boxes and the birthday's month, day and year; an
    address that is not an address refused in a sentence naming it; the Add
    Phone Number dialog's Country entries; and a doubted number's sentence, then
    a second OK keeping it. **NVDA.** [ledger 596]
92. **Give a synced contact all five name parts and sync it.** Google, Outlook
    and, when you have one, an address book server should each keep the prefix,
    the middle name and the suffix and send them back unchanged at the next
    sync. **A person, with an account.** [ledger 597]
93. **Open a new event and press Up, Down, Left and Right on the start's
    minutes.** The time spoken after each key; whether the end moving with the
    start is heard or goes unsaid; the reminder window's time after the same
    keys; and the New events last list on the Calendar and PIM tab. **NVDA.**
    [ledger 603]
94. **Open Tools, Signatures, then an account's own dialog, then a new
    message.** The Signature Manager's Used by column read row by row; the
    account's Signature for this account choice with its first entry naming the
    default; the check boxes under Use for these accounts in a signature's
    editor; and "Signature changed to" and the name when you change From.
    **NVDA.** [ledger 605]
95. **Open the Action menu's Label submenu, then Edit Labels.** Each label heard
    with its key, after the labels load and again after a rename and a move;
    Edit Labels at the end; the Label Manager's Key column read row by row; a
    move said as, for example, "Later, 2 of 5." after `Alt+Shift+Up`, with the
    cursor staying on the moved row; and `Ctrl+6` with five labels saying
    "There is no label 6" from the message list. **NVDA.** [ledger 607]

## B. Low vision and colour

Nine items. Both themes for every one, and Windows high contrast for the last.

1. **Measure text contrast** on the main window, the composer, the reader and
   settings, in the light theme and the dark one. 4.5:1 for text, 3:1 for
   large text. Nothing automated here measures contrast. **Tool, eyes.**
   [row 1.4.3]
2. **Measure the focus ring, control borders and icons** against their
   surroundings at 3:1, both themes. **Tool, eyes.** [row 1.4.11]
3. **Find every place a colour means something**: unread, flagged, a warned
   message, a switched-off block. Each should also carry text or a shape.
   **Eyes.** [row 1.4.1, ledger 161]
4. **Set Windows text scaling to 200%** and open every window. Nothing should
   be cut off or overlap. **Tool, eyes.** [row 1.4.4]
5. **Set display scaling to 400%, or the window to a narrow width**, and open
   every window. Nothing should need scrolling in two directions. Edit Event
   is already known to run off the bottom of a 768-pixel screen with its last
   four fields six pixels tall; start there. **Tool, eyes.** [row 1.4.10,
   ledger 417]
6. **Widen line and letter spacing in the reading size settings** and read a
   message rendered from HTML. No content should be lost. **Eyes.** [row
   1.4.12]
7. **Tab through every window and watch the focus indicator.** It should be
   visible on every control, in both themes, and nothing, a status line or a
   reminder window, should cover the focused control. **Eyes, keyboard.**
   [row 2.4.7, row 2.4.11]
8. **Open a picture attachment** and check it is drawn at a sensible size,
   legible against either theme, and that a very wide picture does not run
   off. **Eyes.** [ledger 109]
9. **Turn on a Windows high contrast theme** and open the main window, the
   composer and settings. The program should paint nothing over the theme's
   colours. **Tool, eyes.** [guardrail]

## C. Physical and motor

Eight items. Mouse unplugged for all of them.

1. **Do everything in every window by keyboard alone.** Reach every control,
   operate it, and leave. The scan says each element reports itself
   focusable; this is pressing the keys. **Keyboard.** [row 2.1.1]
2. **Check every shortcut in the keyboard shortcuts page** does what the page
   says, and that no two menus on the bar claim the same letter. **Keyboard.**
   [row 2.1.4, guardrail]
3. **Press single keys with focus in each pane.** Space reads the item under
   the cursor and acts only there; nothing else should fire on a bare
   letter. **Keyboard.** [row 2.1.4]
4. **Move a message to a folder without dragging**, and do every other drag
   the program offers, a file onto the composer among them, by keyboard
   instead. **Keyboard.** [row 2.5.7, ledger 129, ledger 132]
5. **Measure the smallest thing you click**: toolbar buttons, the up-down
   arrows on a spinner, the resize grip. 24 by 24 CSS pixels at least. **Tool.**
   [row 2.5.8]
6. **Press a button and drag off it before releasing.** Nothing should
   happen; native controls act on release, and the program's own handlers
   should too. **Eyes.** [row 2.5.2]
7. **Leave every window open for an hour.** Nothing should time out, and the
   reminder window's tone should stop on its own after ten repeats. **Ears,
   eyes.** [row 2.2.1]
8. **Rotate a tablet display.** The program should not lock orientation.
   **Eyes.** [row 1.3.4]

## D. Learning and cognitive

Ten items. Read each sentence as somebody meeting it for the first time.

1. **Cause every error you can**: a wrong password, a full outbox, a search
   that finds nothing, a time that has gone, a move the account will not
   take. Each should say what happened, why, and what to do next, in plain
   words. **Both, eyes.** [row 3.3.1, row 3.3.3]
2. **Read every field label and heading** and ask whether it says what the
   field wants. The MSAA walk says a name is present; this asks whether it
   instructs. **Eyes, both.** [row 2.4.6, row 3.3.2]
3. **Find an instruction that relies on shape, position or colour**: "the
   button on the right", "the red one". There should be none. **Eyes.**
   [row 1.3.3]
4. **Land on every control and change every setting.** Landing should change
   nothing; changing a setting should not act before OK; typing in a field
   should open or close nothing. **Both.** [row 3.2.1, row 3.2.2]
5. **Set up an account from nothing.** You should not be asked for anything
   twice. **Eyes.** [row 3.3.7]
6. **Sign in with a stored password and with a provider's page.** Neither
   should set a puzzle, and the stored path should need no retyping.
   **Both.** [row 3.3.8]
7. **Delete something, and send something.** Deleting should ask first and
   name the thing; sending should be previewable and holdable in the outbox.
   **Both.** [row 3.3.4]
8. **Listen to the long sentences** and decide whether each is a confirmation
   or a list to sit through: six attached files named in one breath, seven
   suggestions in all, a sync summary with two counts, a tally after every
   condition change. The numbers behind each were chosen by reasoning, not
   by listening. **NVDA.** [ledger 26, ledger 123, ledger 126, ledger 217]
9. **Read the decorative-picture question**, a Yes/No box whose text runs to
   four lines and whose buttons say only Yes and No. Decide whether the
   answer's consequence is clear from the buttons. **Both.** [ledger 136]
10. **Find the help from each module.** It should be in the same place every
    time, and the experimental markings on menus and in the first-run screen
    should say what could go wrong, not only that something might. **Both.**
    [row 3.2.6, ledger 36]

## E. Hearing

Five items. Every sound the program makes should have something to see.

1. **Play every earcon from the sound scheme**, with the screen open. Each
   should have a visible twin: a status line, a row change, a window. Note
   any sound with no visible equivalent. **Ears, eyes.** [row 1.2.1,
   guardrail]
2. **Let a reminder come due.** The tone's visible twin is the window and the
   status line; both should be there before the tone repeats. **Ears, eyes.**
   [ledger 381]
3. **Open a message with an audio or video attachment.** Any captions or
   transcript the sender sent should be surfaced, and their absence should be
   said plainly rather than the media presented as though it were accessible.
   **Eyes, both.** [row 1.2.1, row 1.2.2, row 1.2.3, row 1.2.5]
4. **Mute reading aloud with one key while something is being read**, and
   turn each sound off in settings while it plays. Both should take effect at
   once. **Ears.** [row 1.4.2]
5. **Tell the misspelling earcon from the landing announcement**, and the
   nothing-found signal from the coverage sentence. Each pair should be
   distinct, and none should flood under a syncing mailbox. **Ears.**
   [ledger 122, ledger 34, guardrail]

## F. Vestibular and photosensitivity

Three items.

1. **Turn on the Windows reduced-motion setting** and switch modules, open
   and close the reminder window, and change theme. Nothing should animate
   that did before, and nothing should slide or fade. **Eyes.** [guardrail]
2. **Watch the whole program for a minute of heavy sync.** Nothing should
   flash more than three times a second, and a list repainting on arrival
   should not flicker. **Eyes.** [row 2.3.1, ledger 6]
3. **Open the About window with and without the disclosure**, since it grows
   to fit. The change of size should not jump the whole window. **Eyes.**
   [ledger 308]

## When you have finished

Seventy-six items: forty-one for screen readers, nine for low vision, eight
for motor access, ten for cognitive access, five for hearing, three for
vestibular and photosensitivity. When every item has a date and a technology
against it, the pass has happened once, on one machine, with one version of
each screen reader. Write that version down. The next release owes the walk
again for whatever it changed, and the coverage page's "what has not
happened" list is where the answer to "has anybody heard this" lives until
then.

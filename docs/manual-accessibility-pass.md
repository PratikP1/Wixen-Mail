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

Forty-one items. Walk them in order; the first ones are the ground the rest
stand on.

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
    its description. **Both.** [row 3.3.2, ledger 137, ledger 233, ledger 295,
    ledger 317]
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

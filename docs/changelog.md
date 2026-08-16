# Changelog

All notable changes to this project will be documented in this file.

Versioning follows [SemVer](https://semver.org/). Development happens on plain `0.x.y`, because `0.x` already means unstable and a version should not also claim a testing programme that is not running. A suffix like `0.6.0-alpha.1` stages a release that is about to go to testers. A build handed to somebody between releases carries the commit it came from, as `0.5.0+g64c73dd`; everything after the `+` is build metadata and is ignored when comparing versions.

## [Unreleased]

### Fixed

- **Typing in the box above the contact list now narrows which contacts it
  shows, searching within whatever the sidebar already has selected.** The
  search box read what was typed and announced "Searching contacts: ..." and
  did nothing else: the list beside it never changed, so typing a name into
  it never found anyone. Typing now narrows the list by name, email address,
  phone number, or company, the same way the Manage Contacts dialog's own
  search already did. Doing this inside a chosen group, Favorites, or All
  Contacts narrows further rather than starting over from everyone: searching
  "grace" inside a group named Book Club shows only the Graces in that group,
  not every Grace in the address book.

  Known limitations: this search only reaches the fields above. The Manage
  Contacts dialog's own search also matches a nickname and every email
  address or phone number a contact holds, not only the first of each; this
  one does not, because the contacts list here keeps only the first of each
  for its rows. Reaching that same breadth would mean carrying a nickname and
  every address on the row itself, which is tracked as a follow-up rather
  than done quietly here.

- **Replying to a message, or replying to everyone on one, now actually
  sends when the person you are answering has a name attached to their
  address.** That is most real mail: an incoming message's sender is shown
  and stored as a name next to an address, and Reply carried that exact
  text into the To line unedited. Sending it that way was refused at the
  server every time, with an error that gave no way to fix it, and the
  message sat in the outbox failing the same way on every retry. The only
  way around it was retyping the address by hand with the name taken off
  first. An ordinary name now sends the same as one typed as a bare
  address, and so does a name that itself contains a comma, the way a
  "Last, First" style name does, or a literal angle bracket, which is
  unusual but legal in a name and which a sender's own mail program can
  produce.

  Known limitations. A name that needs an escaped quotation mark and also
  contains a comma or a bracket elsewhere has not been made to work; nobody
  has seen one in real mail yet.

- **A sender's name that contains a literal angle bracket, unusual but legal,
  is now read back whole instead of cut off.** `Bob <VIP>` is a real name a
  mail program can produce, and the code that pulls a name out of a stored
  "Name <address>" entry treated the bracket inside it as the start of the
  address, the same way an address wrapper starts one. Announcing a reply
  said only "Bob" rather than "Bob <VIP>", the message list's Correspondent
  column showed the same cut-off name, and replying to everyone could list
  that person twice, once under their real address and once under whatever
  the broken search had pulled out. All three now read the whole name back
  correctly, the way the rest of the program already reads a name off an
  incoming message.

- **A recipient's name now reaches the message that goes out, the same way
  this program already sends your own name in front of your own address.**
  This is what Reply pre-fills the To line with whenever the message being
  answered showed a name next to the address, and it is also what a name
  typed straight into the To or Cc line by hand becomes. Both used to reach
  the person but arrive as a bare address, so the recipient's own mail
  program had nothing to show but the address itself where every other
  message shows a name. A recipient's name comes from a message somebody
  else sent, so it is treated as untrusted text: before it is written onto
  the outgoing To or Cc line, it goes through the same check that already
  strips a stray line break out of your own name, so a name cannot be used
  to write extra, unwanted lines into the headers of the message it travels
  on.

- **Fixed: a day a calendar server has already moved or changed out of a
  repeating meeting could not be edited here on its own.** A calendar server
  keeps a day like that as its own appointment inside the same document as
  the series it came from, the same way a Google calendar keeps it as an
  appointment of its own. Editing that day here was refused outright until
  now, with changing the whole series the only thing that could be done.
  Editing the one day now works: the change reaches the calendar server as a
  change to that appointment alone, and the series, and any other changed
  day sharing the same document, are left exactly as they were.

  Known limitations. Deleting a day like this is still refused; the whole
  series can still be deleted, the same as before this fix. Editing needs
  the series itself already stored here, which is normally already true by
  the time a changed day is read at all; the one time it is not is a first
  sync whose answer names a changed day without the series it belongs to,
  something some calendar servers do. Editing that day stays refused until a
  later sync brings the series in as well.

- **Choosing a group, Favorites, or All Contacts in the contacts sidebar now
  changes which contacts the list shows.** A contact group could always be
  made, named, and have people put in or taken out of it, and the sidebar
  tree always said how many people were in each one. Choosing that row, or
  Favorites, or All Contacts, did nothing else: the contact list beside the
  sidebar never changed to match what was clicked, so a group that had been
  carefully built looked exactly like an empty sidebar to anybody trying to
  use it. All three now narrow the list, and the change is read aloud along
  with how many contacts are showing.

- **Fixed: a contact's phone number always said "Phone" in the contacts
  detail pane and when read aloud, whatever it was really labelled, and no
  postal address appeared there at all.** Every phone number and postal
  address a contact holds keeps the label a synced address book or an
  imported card gave it, such as "Work" or "Home". Storage and the contact
  editor already read that label correctly. This one summary view did not:
  a number Google or Outlook labelled "Work" was shown and read as "Phone:
  555-0100," with "Work" nowhere in it, and a postal address never showed
  here regardless of its label. Both now carry their real label, and the
  postal address has a line of its own in this view for the first time.

  Known limitations: this view still shows only the first phone number and
  the first postal address a contact holds. Seeing every number or address a
  contact has still needs the contact editor open.

### Changed

- **The account settings screen now says what a POP account's two
  delete-related checkboxes actually do, not just their names.** "Leave mail
  on the server after downloading it" and "Let me delete mail on this
  computer" used to give a screen reader a name and a checked state and
  nothing else. Neither said what happens next, and for a POP account the two
  are easy to mix up: turning the first one off, together with a number of
  days, removes mail from the server for good, with no Trash there to
  recover it from, unlike deleting mail on an IMAP account. The second only
  ever moves a downloaded message to this account's own Trash folder on this
  computer and never reaches the server at all. Both checkboxes now carry
  that sentence as an accessible description, read after the name when the
  checkbox takes focus.

### Added

- **The Theme setting now colours every module, not only Mail.** Choosing
  Light or Dark used to change three places: the folder list, the message
  list and the sidebar beside them. Every other module (Calendar, Contacts,
  Reminders, Tasks and Notes) kept Windows' own grey and white, so switching
  modules meant switching back to an uncoloured screen. The sidebar and
  content area of every module now take the same colours Mail already had,
  including the button panel used to switch between modules, Contacts'
  search box and detail pane, and the list and editor panes in Notes.

  The window a message opens into for reading is coloured now too, and so is
  the window that shows a conversation as headings. The security warning bar
  in the reader keeps the same background as the rest of the message; there
  is no separate warning colour yet.

  Changing the setting also takes effect immediately now. Closing Settings
  with OK used to leave every open window on its old colours until the next
  restart; the folder list, the message list, every module's sidebar and
  content area, and the reader window's own frame and tab strip repaint on
  the spot instead, with nothing to restart and no delay or flash while it
  happens.

  Known limitations: Compose, the Calendar window itself, and roughly
  thirty other windows that open on top of the main one still show
  Windows' own colours regardless of the Theme setting. A message
  already open in the reader when you change the setting keeps the
  colours it had; closing that tab and opening the message again, or
  restarting, is what picks up the new ones. The description under the
  Theme setting in Settings is updated to match what now works.

- **The Settings window now takes the Theme colours too, including the
  window you are reading this sentence in.** Every tab, and the three text
  boxes on them (font size, the default reminder length, and the download
  folder), now match the rest of the coloured application instead of
  showing Windows' own white regardless of what Light or Dark chose.

  Known limitations: Compose, the Calendar window itself, and roughly
  thirty other windows that open on top of the main one still show
  Windows' own colours regardless of the Theme setting.

- **The Account Manager, and the Add/Edit Account window it opens, now take
  the Theme colours too.** The list of configured accounts and every text
  box in the Add/Edit Account window (name, email, sender name, server
  addresses and ports, username, password, and the check interval) now
  match the rest of the coloured application instead of showing Windows'
  own white regardless of what Light or Dark chose.

  Known limitations: Compose, the Calendar window itself, and roughly
  thirty other windows that open on top of the main one still show
  Windows' own colours regardless of the Theme setting.

- **The window where a calendar event is added or edited now takes the
  Theme colours too.** Every text box on it (summary, start and end date
  and time, location, description, and the reminder) now matches the rest
  of the coloured application instead of showing Windows' own white
  regardless of what Light or Dark chose.

  Known limitations: the Calendar window this opens from, and the confirm
  delete dialog nested inside it, are not coloured yet. Compose and
  roughly thirty other windows that open on top of the main one still
  show Windows' own colours regardless of the Theme setting.

- **The "which days do you mean" question a repeating event's edit or
  delete can open now takes the Theme colours too.** This dialog has no
  text boxes of its own to colour; painting the window itself is what
  changes here.

  Known limitations: the Calendar window this can open from, and the
  confirm delete dialog nested inside it, are not coloured yet. Compose
  and roughly thirty other windows that open on top of the main one
  still show Windows' own colours regardless of the Theme setting.

  This closes out the round of standalone-dialog painting that started
  with Settings above: five dialogs done (Settings, the Account Manager
  and its Add/Edit Account window, the calendar event editor, and this
  one), and the rest named plainly rather than left for someone to
  discover unpainted. What is still left, in the order most people would
  reach it: Compose's own window; the Contact, Filter, Tag and Signature
  managers and their own edit dialogs; Add Calendar, Columns,
  Destination, Item Form, First Run's Allowed Changes screen, Folder
  Choice, Reminder Alert, Thread View, Search, and Ask For A Name; and
  About, which almost nobody opens twice.

- **The Check Spelling window Compose opens for a misspelled or repeated
  word now takes the Theme colours too.** The word, the field holding what
  it will become, and the list of suggestions under it now match the rest
  of the coloured application instead of showing Windows' own white
  regardless of what Light or Dark chose.

  Known limitations: Compose's own window, and roughly thirty other
  windows that open on top of the main one, still show Windows' own
  colours regardless of the Theme setting.

- **The Insert Table window Compose's formatting menu opens now takes the
  Theme colours too.** This dialog has no text boxes of its own to
  colour; painting the window itself is what changes here.

  Known limitations: Compose's own window, and roughly thirty other
  windows that open on top of the main one, still show Windows' own
  colours regardless of the Theme setting.

- **The preview Compose can show before a message sends now takes the
  Theme colours too.** The window itself now matches the rest of the
  coloured application instead of showing Windows' own white regardless
  of what Light or Dark chose. The message preview inside it is a
  browser control that renders the message's own HTML, and keeps
  showing that HTML's own colours the same way the message reader and
  the compose body editor already do.

  Known limitations: Compose's own window, and roughly thirty other
  windows that open on top of the main one, still show Windows' own
  colours regardless of the Theme setting.

- **The Calendar window itself, and the confirm delete question it asks
  before removing an event, now take the Theme colours too.** The list
  of events now matches the rest of the coloured application instead of
  showing Windows' own white regardless of what Light or Dark chose.
  This is the window every event editor and every which-days-do-you-mean
  question opens from, so this closes out the calendar family of windows
  this round has been colouring one piece at a time.

  Known limitations: Compose's own window, and roughly thirty other
  windows that open on top of the main one, still show Windows' own
  colours regardless of the Theme setting.

- **Fixed: the Default theme setting could not tell whether Windows was set
  to light or dark, so it always drew the light colours.** Default is meant
  to match whatever Windows itself prefers, the same way Light and Dark match
  what you pick by hand. Until now it never actually asked Windows, so
  anybody using Default saw the light palette even on a system set to dark.
  Default now reads the real system preference and follows it.

  Known limitations. This changes only which of the two colours Default
  picks, not how far colour reaches; see the entry above for what is coloured
  and what is not. On a system set to dark, the sidebar, message list and
  other coloured surfaces now go dark while the rest of the window, menus,
  dialogs, and every control outside what that entry lists, still renders in
  whatever colours Windows gives an application that has never switched
  itself into dark mode. That mismatch already existed for anybody who chose
  Dark by hand; Default can reach it too now.

- **Fixed: a contact edit that failed to send because of the network was
  thrown away and reported as replaced by the address book, even though the
  address book never actually saw it.** Every contact edit made here is sent
  to Google or Outlook on the next sync. Until now, if that send failed
  because the connection dropped, and the address book's own copy of the
  same contact had also changed since the last sync, the sync treated the
  address book as having won a real disagreement over the contact. It
  replaced the edit and counted it as lost. The edit is kept now instead,
  the same way an edit held back by the Allow Changes setting is kept, and
  it is sent again on the next sync.

  Known limitations. If the connection keeps failing sync after sync, the
  edit keeps waiting with no limit on how many attempts, and nothing yet
  warns you that it has been waiting a long time. An address book that looks
  at an edit and refuses it outright, for example because of the content
  itself, is handled the old way: that kind of refusal can repeat forever,
  so the address book's copy still wins and you are still told.

- **Fixed: editing or deleting a day a calendar server had already moved or
  changed out of a repeating meeting could silently delete the whole meeting,
  or fail forever with a confusing error.** A calendar server stores a moved
  or changed day at the same address as the meeting it belongs to. Deleting
  that one day through the ordinary calendar screen sent an unconditional
  delete to that shared address, removing the whole meeting from the server:
  every day of it, not just the one that was opened. Editing that one day
  saved a change that could never actually reach the server, so every sync
  after that failed the same way again, with a message that did not explain
  what was wrong. Both are refused now, before anything is sent, with a plain
  message saying that nothing has changed and that this particular change has
  to be made at the calendar server directly. This is specific to a calendar
  server: Google and Outlook already give a moved or changed day an address
  of its own, so editing or deleting one of those was never affected.

  The refusal also now covers the very first sync of a new account, and any
  other sync where this program has not yet downloaded the rest of the
  meeting a moved day belongs to. Until now, the refusal only worked once the
  rest of the meeting had already been downloaded on an earlier sync: a moved
  or changed day met for the first time on its own looked like an ordinary
  meeting, and deleting or editing it could still silently delete the whole
  meeting from the server. It is caught now the moment such a day is seen,
  whether or not the rest of the meeting has been downloaded yet.

  Known limitations: this program still cannot send a targeted change to one
  moved or changed day of a repeating meeting back to a calendar server on
  its own. Refusing cleanly is the whole fix for now.

- **Fixed: clearing an existing task's description was never sent to Google
  or Microsoft, so the old text could reappear on the next sync.** Sending a
  changed task always left the description out of the request when it had
  just been cleared to nothing, and both providers read a missing field as
  "leave whatever is already there alone" rather than as an instruction to
  clear it. A description changed to something else still went through
  correctly; only clearing it all the way was silently dropped. The
  description is now always sent, and an empty one goes out as an empty one,
  the same fix already made for a calendar event's description and location.

  Known limitations. There is no screen in this program yet for changing an
  existing task's description, so nobody using it could have hit this by
  hand; it was found by reading the sync code, not from a report. None of
  the task syncing has run against a real account.

- **Fixed: a CalDAV calendar holding a repeating meeting with one day moved
  or changed showed only the unchanged series, with the changed day nowhere
  on the diary.** A calendar server sends a moved or changed day as a second
  appointment in the same document as the series, sharing its identifier and
  marked with the day it replaces. Reading a calendar only ever looked at
  the first appointment in that document, so the changed day was silently
  dropped every time the calendar was read. Both are now kept: the series
  stays a series, and the changed day is stored as its own appointment,
  linked back to the series and shown at the time it moved to. Google
  calendars already worked this way; this is the matching fix for CalDAV.

  Known limitations. A day a calendar server has already moved or changed
  out of a repeating meeting still cannot be edited or deleted on its own
  here; the whole series can still be changed. Cutting a day out of a
  series here, for a day no calendar server has touched before, is
  different and already reaches the server: the day comes off the series
  there too, and is kept as an appointment of its own, the same as it is
  kept here. A day a calendar server marks
  cancelled outright, rather than moved, is shown as a changed appointment
  rather than taken off the diary the way a cancelled Google day already is;
  that is a separate fix still to come. A moved or changed day named in a
  different time zone from the series' own start is read as plain digits
  with no zone conversion, which matches what the calendar standard expects
  and only affects a server that does not follow it. None of the CalDAV
  calendar syncing has run against a real account.

- **Fixed: a day of a repeating meeting moved or cancelled in Outlook still
  showed here, drawn from the meeting's own repeat rule.** Outlook answers a
  calendar view with every day of a series in the window it covers, changed
  or not, and every one of those days was skipped once this program already
  held the series. So a day cancelled in Outlook went on being shown here
  for ever, and a day moved to another time never reached this computer at
  all. A cancelled day is now taken off the series and stops being drawn. A
  moved day is now stored and shown at its new time. Google calendars had
  this same fix already; this is the matching one for Outlook.

  Known limitations. Outlook does not say which day of the pattern a moved
  day replaces in a way this program can read safely: doing that needs the
  account's own time zone offset, and Outlook names its time zones in a form
  this program has no table to turn into one, such as "Eastern Standard
  Time" rather than a plain number of hours. So a day moved in Outlook is
  shown at its new time, but the day it moved from may still be drawn too,
  until this program can read that safely. A cancelled day does not have
  this problem: Outlook still gives its own time, unmoved, so it is taken
  off the series cleanly. None of the Outlook calendar syncing has run
  against a real account.

- **Fixed: a postal address changed at Google, or a postal address or phone
  number changed at Outlook, did not reach a contact this program already
  knew about.** Syncing an already-known contact took the name, the company,
  the notes and several other details from the address book, but kept
  whichever postal address and phone numbers were already stored, even after
  they changed or were removed at the provider. All three now update the same
  way an email address already did: a new value at the provider replaces what
  was stored here, and a value removed at the provider is removed here too.

  Known limitations. A phone number, an email address or a postal address
  that a provider never gave a type of its own is stored and sent back
  labelled "Other", the same word used when somebody deliberately chooses
  that label here. The two cannot be told apart once stored. Nothing about
  the address, the email or the number itself is affected, and it settles
  into "Other" after one sync rather than drifting further on the syncs
  after that.

- **Fixed: the password box in the account dialog had no accessible name,
  and the advice to use an app password reached nobody working by ear.**
  The box now announces as "Password" like every other field in that
  dialog, and for an address whose provider offers app passwords, the same
  advice the visible hint under the email box shows is read once when the
  password box takes focus. This reaches NVDA, which reads this dialog's
  native controls the way this program names them. It does not reach
  Narrator, which reads the visible label instead and is unaffected by
  this change.

- **Fixed: adding, updating or deleting an account, a contact, a filter,
  a tag or a signature said "Added", "Updated" or "Deleted: {name}" with
  nothing saying what kind of thing had changed.** The delete sentence was
  worded character for character like the sentence the mail path says when
  a message is removed from a server, so by ear the two were
  indistinguishable. All five windows now name the kind of row as well as
  its name: "Added the filter: Newsletter", "Deleted the contact: Jane
  Smith".

- **Fixed: the account manager described a missing sign-in credential four
  different ways, one of them reading a file name aloud and one of them
  jargon with no remedy.** Adding an account, editing one, and signing in
  again all now say the same sentence when this build has no browser
  sign-in credentials for a provider, naming the provider and pointing to
  Setting up a provider in Help. Two other messages on the same screen,
  raised when an address cannot use browser sign-in at all or when signing
  in fails to start on this computer, are reworded in plain language for
  the same reason.

- **Fixed: an account manager button offered to test the connection and did
  nothing.** Pressing it read out "Testing ... (not yet implemented)", which a
  screen reader could not tell from a real result. Nothing in this program
  ever implemented a connection test, so the button is removed rather than
  wired up to a promise this program cannot keep yet.

- **Fixed: a task Outlook held as in progress, waiting on somebody else, or
  deferred was reset to not started the first time it was touched here.**
  Microsoft To Do tracks five states for a task; this program only ever read
  two of them, done and not done, and sent that boolean straight back out.
  Ticking off one task, renaming another, or moving a third to a different
  list used to carry every task like it back to "not started" on the next
  sync, destroying whatever state Outlook actually held it in. The word
  Outlook gave a task is now kept and sent back unchanged unless the task is
  marked done here.

- **Fixed: a note or a task description written with headings or a list in
  Outlook or Microsoft To Do was read aloud as raw HTML tags.** The
  formatting is now turned into the same structure this program's own notes
  and events understand, so a heading is announced as a heading and a list as
  a list rather than as punctuation. Only the structure survives the trip:
  fonts and colours do not, and the first change made to a task or event
  description here replaces Outlook's original formatting with plain text
  carrying that structure.

- **Fixed: a priority set on a task in Google Tasks was silently cleared.**
  Google Tasks has no priority field at all, so reading a task back from
  Google always answered "normal" for it, and that answer used to be written
  down over whatever priority had been set here. The priority chosen here is
  now kept when Google's own copy of the task changes; a priority changed in
  Outlook still replaces the one held here, because Microsoft does track it.

- **Fixed: ticking off a task here could send Outlook the wrong hour it was
  completed at.** The completion time is now sent in the shape Outlook's own
  service expects, a clock face with no time zone offset attached, matching
  the way this program already sends every other date to Outlook.

- **Fixed: a repeating meeting added to an Outlook calendar late at night or
  early in the morning told Outlook two different days.** The meeting itself
  went out on one day and how often it comes round went out counted from
  another. That happened whenever the hour you set was on one side of midnight
  where you are and the other side in Greenwich time, which is what Outlook is
  given.

  A weekly meeting made this way repeated on the wrong weekday: set for two in
  the morning on a Wednesday in India, it went to Outlook as a Tuesday meeting
  that then repeated every Wednesday. A monthly meeting was sent as the tenth
  of the month with a repeat on the eleventh, and a yearly one the same. Both
  halves now come from the one start Outlook is actually given.

  This has not been checked against a real Outlook account. Microsoft documents
  that the two have to agree, so a meeting made this way may have been refused
  outright rather than filed on a day nobody chose. What has been checked is
  what leaves this computer, and it used to leave saying both days.

- **Fixed: a day you cancelled in Google Calendar came back on your diary, and
  a meeting you moved to another time was shown twice.** Reading a Google
  calendar as the repeating meeting itself, rather than as a list of the days it
  falls on, is new in this release. Read that way, Google sends a day you
  cancelled and a day you moved as separate entries of their own, and nothing
  here read them. So the cancelled day went on being drawn from the repeat, and
  the moved meeting was drawn twice that day, once at the old time from the
  repeat and once at the time you moved it to.

  A day cancelled in Google Calendar is now taken off the repeating meeting
  here, the same way a day cancelled on this computer or on a calendar server
  is. A day moved to another time is now one appointment at its new time, saying
  which repeating meeting it came out of, with that day taken off the meeting.
  That is the same pair of entries this program already keeps when you change
  one day of a repeating meeting yourself.

  Known limitations. Moving one day of a Google repeating meeting on this
  computer still cannot be sent to Google, unchanged from earlier in this
  release. An Outlook calendar still shows a day you moved at the time the
  repeat says rather than the time you moved it to, and a day you cancelled in
  Outlook is still drawn. None of the Google calendar syncing has run against a
  real account.

- **Fixed: an event filed under two categories in Outlook came back filed
  under one.** The two were sent back as a single category with a comma in the
  middle of its name, and Outlook reads the categories it is given as the whole
  truth, so that one replaced both. Each category now goes back on its own.

- **Fixed: editing an Outlook contact dropped all but one of their phone
  numbers.** Only one number was kept when a contact came down from Outlook,
  and Outlook replaces a contact's numbers with whatever it is next sent. So
  somebody with a home number and a work number and a mobile kept one of the
  three the first time anything about them was changed here. Every number now
  comes down and goes back up, under the label of the place Outlook keeps it
  in.

- **Fixed: editing a Google contact took that person's postal address off
  their Google contact.** An address held at Google was read into nothing here,
  so it was missing from every change sent back, and Google treats a missing
  address list as an instruction to clear the one it holds. Changing anything
  at all about a Google contact, a nickname, a phone number, a note, wiped
  their address, and nothing said so. Addresses now come down with the rest of
  the contact and go back up with it.

- **A contact sent to Google no longer carries the name Google works out for
  itself.** Google builds a person's full name from the parts and throws away
  whatever is sent in that field, so sending one was a second answer to a
  question Google already answers. Nothing changes today. It stops changing
  later: a change replaces a contact's whole name, so the day Google started
  honouring what was sent, our copy would quietly have beaten the first and
  last names sent beside it.

  A contact whose whole name was only ever recorded on one line is now read
  back under that name instead of showing as Unknown.

- **Fixed: every address sent to Google carried an explicit empty description
  line, even though nothing here ever sets one.** Google composes the address
  line it shows from the street, city and other parts already in the request,
  and this program always left that line blank so Google could keep doing
  that. It went out as an explicit empty value rather than being left off the
  request entirely. Google's documentation does not say whether an address
  update reads an explicit empty value as "work it out yourself" or as "clear
  the line", so rather than resting on that answer, the value is now left off
  the request whenever there is nothing to put in it. Closed at the same time:
  an address with no chosen label could have gone out with an empty label
  instead of none at all, though no address made or synced by this program
  currently produces one.

- **Fixed: a contact whose address at Google was recorded as a single line
  rather than a street, city and so on came back with no address at all, and
  the next change sent cleared it.** Google fills in a composed line for every
  address it holds, whether or not it also holds the separate parts, so an
  address given to Google as one line reads back with only that line set. It
  was read here into six empty boxes, stored as an address of nothing, and
  sent back that way, taking the address off the Google contact. The line is
  now kept, in the same place a one-line address from a card import or the
  contact editor is already kept, and sent back as a street address rather
  than left to Google to compose again, so it survives even without settling
  what Google does with an address update that carries no line of its own.

- **Fixed: a contact whose only postal address was recorded before this
  program kept a list of addresses sent no address to Google or Outlook, and
  the next change cleared whatever either held.** The email address and the
  phone number both already fell back to the single column recorded for a
  contact from before the list existed; the postal address had no such
  fallback. It does now, for both address books, and a blank address row left
  behind by the editor is no longer sent either.

- **Fixed: correcting anything about a contact synced from Outlook, a phone
  number, a note, anything, sent that contact's own name over a name Outlook
  keeps beside one of their addresses.** Outlook can hold a name beside each
  address a contact has there, separately from the contact's own name, a
  maiden name kept on an old address being the ordinary case. That name was
  never read in here, and every push to Outlook stamped the contact's own
  name onto every address, which overwrote it on the first change made to the
  contact after it synced. The name is now read in, kept through a Google
  sync and through an edit made in the contact editor, neither of which can
  otherwise see it, and sent back to Outlook unless nothing was ever recorded
  for that address, in which case the contact's own name is still sent so an
  address typed here reaches Outlook with some name on it.

- **Fixed: which of a Google contact's several addresses or email addresses
  showed as the main one was decided by the order Google happened to list
  them in, not by which one Google actually calls primary.** This program
  always shows a contact's first address or email as the main one, in the
  contact list, in the editor, and when a change is sent to Outlook. Google
  says which one it considers primary through a flag of its own, and that
  flag was read and thrown away. The one Google calls primary is now put
  first, so this program's own "first is main" rule agrees with Google's
  answer instead of contradicting it whenever Google's list happened to put
  a different one first. This program still cannot write that flag back:
  Google refuses a change that names it, so which address or email Google
  treats as primary can only be read here, never set.

- **Fixed: ticking off a Google task could put back a task deleted on another
  device.** Every change sent to Google Tasks carried a claim that the task had
  not been deleted, whether or not that is what your copy meant. Changes go up
  before the new ones come down, so the claim arrived while word of the
  deletion was still unread here, Google put the task back, and the read that
  followed found it alive and kept it. Your deletion was undone and nothing was
  said about it. Nothing this program sends now says anything about whether a
  Google task was deleted; deleting one is still its own separate request.

- **Fixed: copying a message into a task or an event made something that could
  never be sent anywhere.** Copying a message fills in a title and a body and
  asks nothing else, and the questions nobody was asked were stored as answered
  with nothing. A task like that was refused by Microsoft To Do on every sync,
  for as long as the task existed, and you were told only that a sync had a
  problem. Those questions now take the answer the form would have shown you if
  it had opened: a task is normal priority, an event is confirmed and busy.

  A priority that is not one of the three the task services understand is now
  stopped here, in a line naming the value and the words that would work,
  rather than being sent and refused. Nothing has been run against a real
  account: what is tested is what leaves this machine.

- **Fixed: opening Tasks or Notes did not say what had loaded.** Opening the
  calendar says how many calendars and how many reminders arrived. Opening
  Tasks said how many tasks but never how many lists, and opening Notes filled
  its sidebar without a word either way. Both now say what arrived, the same as
  the calendar does.

  Three other things that change the status bar stay silent on purpose, and the
  reason for each is now written down beside the code: turning offline mode on
  or off, the outbox queue count, and the module name in the status bar. The
  first two would repeat a sentence you have just heard. The third is sent both
  when you really switch module and when a finished task sync asks the panel to
  redraw, so speaking it would tell you that you had moved to Tasks when you
  had not.

- **Fixed: the tag, signature, filter and contact windows did everything in
  silence too.** Adding, changing and deleting an item put a word on a line of
  text above the buttons and said nothing, so by ear there was no confirmation
  that anything had happened and no way to tell which item had been deleted.
  All of them now speak, and deleting says the name of what went. Being told to
  select something first comes in ahead of the ordinary run of messages.

- **Fixed: the Account Manager did everything in silence.** Every answer that
  screen gives, twenty-three of them, was written on a line of text above the
  buttons and said nowhere. That is the screen you add your first account on,
  and it includes the answers you have to do something about: signing in
  failed, no credentials are set up for that provider, the account is added but
  not authorised. Working by ear you pressed a button and heard nothing back.
  All of them are now spoken as well as shown, and the ones you have to act on
  come in ahead of the ordinary run of messages.

  Two of those sentences pointed you at a page by its file name, which read out
  one character at a time. They now name the page the way the Help menu does.
  Being told to select an account, with no word about what for, now says what
  for.

  Known limitation. The hint under the email address box, the one that tells
  you to use an app password rather than your ordinary one, is still shown and
  not spoken. It is rewritten on every keystroke while you type an address, so
  speaking it would read a paragraph over you again and again. Attaching it to
  the password box, so a screen reader reads it when you get there, is the
  proper fix and has not been done yet.

- **Fixed: the same loss in an Outlook calendar, and a meeting marked tentative
  came back confirmed.** A meeting that repeats, made here in an Outlook
  calendar, arrived there as a single appointment for the same reason as at
  Google, and the read that followed took the repeat off the copy here. Making a
  meeting now says how often it comes round, which days of the week it lands on,
  and whether and when it stops.

  Outlook is told a shape rather than a rule, and the two ways of saying it are
  now checked against each other, so a repeat cannot be filed as a different one.
  A repeat Outlook has no way of saying is refused rather than rounded to
  something near it: a meeting on Tuesdays and Thursdays would otherwise have
  been at risk of being filed as a Tuesday meeting with every Thursday gone. When
  that happens the meeting still goes up, once, on the day it starts, and the
  sync tells you how many meetings went without their repeat, so the days that
  are only on this computer are not something you have to find for yourself.

  A meeting Outlook sends back as part of a series is also no longer written into
  your diary a second time on top of the series already there.

  Separately, the status of a meeting, confirmed or tentative or cancelled, is
  yours alone on an Outlook calendar. Outlook has no field for it, so every read
  used to say confirmed, and a meeting you had marked tentative quietly became
  confirmed. What you set is now kept.

  Known limitations. Turning a repeat on or off on a meeting Outlook already
  holds does not reach Outlook. Days you have cancelled out of a series cannot be
  sent to Outlook when the series is made, because Outlook only takes those one
  at a time on a series it already holds; the cancellation is kept here. A series
  made in Outlook still arrives here as separate meetings, one per day, because
  the only way Outlook offers to read a calendar bit by bit answers with days and
  never with the series. None of the Outlook calendar syncing has run against a
  real account.

- **Fixed: a meeting that repeats, made here in a Google calendar, arrived there
  as a single appointment and then stopped repeating here as well.** Make a
  weekly meeting in a Google calendar in this program and it was filed at Google
  as one appointment on one day. Nothing said so. The next read of that calendar
  then brought the single appointment back and took the repeat off the copy on
  this computer too, so the meeting was gone from every week but the first at
  both ends, with no way to tell and nothing to undo.

  Making a meeting now says how it repeats, and any days of it you had already
  cancelled go up with it, so cancelling a day is not undone by the meeting being
  created. Changing a meeting Google already holds still says nothing about how
  it repeats, which is deliberate: Google reads that as the whole truth about the
  series, and this program cannot say all of what a series can hold.

  A Google calendar is now read as the series itself rather than as a list of the
  days it falls on, which is how calendar servers were already read. A meeting
  that repeats is one entry in your diary and the days are drawn from its rule.

  Known limitations. Turning a repeat on or off on a meeting Google already holds
  does not reach Google; you have to delete the meeting and make it again. If
  your diary already holds a separate entry for each day of a Google series from
  an earlier version, those entries stay where they are until you delete them,
  because Google will not name them again. None of the Google calendar syncing
  has run against a real account.

- **Changed: sending a queue of mail now signs in to your mail server once
  instead of once per message.** Every message sent put a copy in your Sent
  folder, and each copy opened its own connection and closed it again. A queue of
  fifty messages was fifty sign-ins, and some providers turn that down partway
  through. One connection is opened before the queue starts and closed when it
  finishes. If your account keeps its sent mail on this computer, nothing signs
  in at all.

  A very long queue can still outlive that connection. If it does, the copies
  after it are saved on this computer instead and you are told, the same as when
  a server refuses one. Nothing goes missing quietly.

- **Fixed: the Outbox sat at the bottom of the folder list.** Mail waiting to go
  is the one folder there is something to do about, and it was the last thing in
  the tree: below Trash on an account that downloads its mail, and in among your
  own folders in alphabetical order on one that keeps it on a server. It now
  sits where the rest of the program already said it belongs, just under Drafts
  and above Sent. Nothing moves in your mailbox; this is only where the folder
  appears in the list.

  A folder whose kind was stored with spaces around it also used to sort as
  though it were an ordinary folder while reading as what it really is
  everywhere else. It now sorts as what it is. Folder names with accented
  letters may come back in a slightly different order than before, and it is now
  the same order the server's own folder list uses.

- **Fixed: a change or a deletion for a task could reach the wrong task, or
  nothing at all.** A task and the list holding it are named by identifiers the
  service picked, and those identifiers were put into the web address exactly as
  they came. One holding a slash, a plus, a question mark, a hash or a space
  breaks the address in two, so the request arrives pointing somewhere else, and
  a deletion sent to the wrong task cannot be taken back. The reading side was
  worse: a hash cut the address short, and what was cut off with it included the
  parts that ask for completed, hidden and deleted tasks. A read cut short comes
  back with less than the list holds, and less is what this program reads as
  "these were deleted somewhere else", so it would have taken tasks off this
  computer that nobody had deleted anywhere. Identifiers now go into a task
  address the same careful way contact and calendar identifiers already did.

  Known limitation, unchanged: none of the task syncing has run against a real
  account.

- **Fixed: the Calendar window answered out loud and then stopped.** Everything
  that window says back, a refusal, a note that nothing is selected, and what it
  is waiting to do when you close it, went onto a line of text under the title
  and nowhere else. Nothing raises a notification for that line, so a screen
  reader read it only if you went looking. The clearest case: press Delete on
  one day of a repeating event in a Google calendar, an Outlook calendar or one
  this program can only read, answer "just this one day", and the refusal
  arrived in silence. Every sentence that window gives back is now said as well
  as shown, and a refusal comes in above the ordinary run of status, because it
  is the answer to the key you just pressed.

  Known limitation: this has not been through a screen reader yet. What is
  tested is that every sentence goes out through the same call that shows it.

- **Fixed: what the Calendar window did was reported by an internal identifier,
  on a line nobody hears.** Creating an event, changing one, and taking one day
  off a repeating event each put a line on the status bar naming the event by
  the identifier it is stored under, and announced nothing. The Delete key
  on the calendar panel already said the same actions in words. The window now
  says what it did in the same words, names the event by its title, and says it
  once when the window closes rather than once per action, because several
  status sentences in a row leave only the last one heard.

- **Fixed: closing the Calendar window having done nothing said events were
  saved.** Open Calendar, press Close, and a screen reader announced "calendar
  events saved". It now says nothing, which is what every other manager already
  did when you cancel out of it.

- **Fixed: the Sync button in the Calendar window said a sync had started and
  started nothing.** It announced "Calendar sync requested" and that was the
  whole of it. It now starts the same sync the Calendar menu entry starts.

  Known limitation, unchanged: calendar syncing has never run against a real
  account.

- **Fixed: "just this one day" described a delete as though it kept the day.**
  Under that answer, the Calendar window and the Delete key both read out a
  sentence written for an edit: that the day would be taken off the series and
  kept as a separate appointment, so there would be two entries from then on. A
  delete keeps nothing. Deleting and changing now describe themselves in their
  own words, on both doors.

- **Fixed: "just this one day" promised a calendar server a change it would
  refuse.** For a repeating event whose time zone is spelt the way Outlook and
  Exchange spell it, changing one day is refused before anything is written,
  because the appointment kept for that day could never be created on a calendar
  server. The sentence read out under the answer still said both halves would go
  up on the next sync, and it was heard before the refusal. It now says the
  change cannot be done there yet, names the time zone that stops it, and says
  choosing it changes nothing. Taking one day off such a series is unaffected: a
  delete keeps no appointment, so there is nothing for the server to refuse.

- **Fixed: changing just one day of a repeating event could take that day off
  your calendar server and never put it back.** Changing one day is one action
  to you and two changes to the calendar server: a separate appointment is
  created for that day, and the day is taken off the series. Those two went up
  independently, and the one that takes the day away was allowed to succeed on
  its own. If the calendar server would not accept the new appointment, and for
  a series whose time zone came from Outlook or Exchange it never would, the day
  was removed from the series at the server and the replacement never arrived.
  The appointment was then on this computer and nowhere else: gone from your
  phone, gone from anything else reading the same calendar, and gone for good if
  this computer was ever set up again from scratch. Nothing said so, because the
  copy here still showed it.

  The two now go up as a pair. The new appointment goes first, and the day is
  only taken off the series once this computer has seen the calendar server
  accept it. If the new appointment is turned down, or the sync cannot tell
  whether it went, the series is left exactly as it was and the sync says why
  and that it will try again next time. If the new appointment lands and the
  change to the series is turned down, the appointment shows twice until the
  next sync, which you can see and put right. It can no longer show nowhere.

  Both halves still go up in one sync rather than needing two, and a program
  closed between the two remembers they belong together.

  Known limitation, unchanged: none of this has run against a real calendar
  server. What is tested is what is sent, what is stored, and one test server
  running on this machine.

- **Fixed: taking one day off a repeating event said it was done while the day
  was still on the list.** Pressing Delete on one day of a repeating event and
  answering "just this one day" said that the day was taken off and the other
  days were unchanged. That was true in what is stored, and the calendar list on
  screen still held the day until something else read it back, so the sentence
  and the list disagreed. Anybody working from the sentence had no way to know,
  and anybody working from the list would delete it again. The list is now read
  back before the sentence is said, the same way every other delete on that key
  already worked.

- **Fixed: a confirmed delete of a repeating event could end in silence.** If
  the event was taken away by a sync, or by another window, between the
  confirmation and the answer about which days it meant, the key did nothing and
  said nothing. Silence there is the one thing indistinguishable from a delete
  that worked, so the next press landed on whichever row had moved up. It now
  says the event is no longer there and that nothing has been changed.

- **Fixed: the Calendar window said an event had been changed or deleted the
  moment you asked for it.** Nothing that window is asked for happens until it
  closes, and on a Google calendar, an Outlook calendar, or a calendar this
  program can only read, changing or deleting one day of a repeating event is
  not carried out at all. So "That one day is taken off" was said, the window
  closed, and a refusal followed saying nothing had been changed. The window now
  says what it is waiting to do and when it will happen, and a calendar that
  will refuse the answer refuses it straight away, while you are still standing
  there. For an edit that refusal now comes before the editor opens rather than
  after the whole form has been filled in.

  That line of status text is now read out as well as shown. See the entry
  below.

- **Fixed: taking one day off a repeating event from the Calendar window
  reported it as a deletion.** The event is still there, with that one day taken
  out of it. The status line said it had been deleted.

- **Fixed: every message sent went out with no identifier of its own, so no
  mail program could thread it.** A message carries an identifier that every
  other program uses to work out which conversation it belongs to. Messages sent
  from here carried none. A recipient's mail program had nothing to join the
  message to, so a conversation started here arrived as loose mail, and each
  reply after it started again. The copy kept in Sent had nothing in that column
  either, so replying to something you had sent yourself began a new
  conversation rather than continuing the old one.

  Sent mail now carries one. It names the domain of the address the message is
  sent from, which is the same domain the recipient reads on the From line, and
  never the name of this computer. A message that failed and is tried again
  keeps the identifier it had the first time, so the recipient sees one message
  and not two. Read receipts carry one as well.

  Drafts saved to a Drafts folder keep the rule they already had, which is a
  deliberately different one: a draft is not a message anybody has received and
  its identifier says so.

  Replies were already sending the two headers that say which message they
  answer, and those are unchanged. What was missing was the identifier on the
  message being answered, when that message was one this program had sent.

- **Fixed: mail collected over POP could be removed from the server whatever
  Allow Changes said.** This only affected accounts set to clear the server:
  the ones where Leave mail on the server is off and a number of days is set.
  For those, mail old enough to go was removed from the provider whether or not
  mail changes were allowed, and starting the program with changes turned off at
  the command line did not stop it either. Removal over POP is permanent and
  there is no trash behind it, so where those settings were on, this could have
  been the last copy anywhere but this computer. Every other kind of change,
  sending, deleting and filing over IMAP, was already behind the setting.

  Removing a message from a POP server now asks the same question everything
  else asks, and says so when it is turned down: "Refused to remove a message
  from the mail server", followed by how to allow it.

  What changes for somebody using it. An account with those housekeeping
  settings on and mail changes off will now keep its mail on the server, and the
  status line at the end of the check says how many are still there and where to
  turn it on, instead of the mailbox quietly emptying. Mail still arrives here
  exactly as before, and the check itself is not reported as a failure: nothing
  went wrong, the clear-out is waiting on a setting.

  There is also now a check that fails the build if any part of this program
  grows a new way of reaching a server without somebody writing down whether it
  can change anything. That is what was missing: the list of places that can
  write was kept by hand and POP was never on it.

- **Fixed: a mail server that refused to list a folder was read as a folder
  with no mail in it, and the messages already downloaded were deleted to
  match.** Asking a server which messages a folder holds and being turned down
  came back as the same answer as asking and being told "none". The sync
  believed it, and every message it had stored for that folder went, along with
  the copy of each message kept for reading offline. Mail written on this
  computer, sent copies, drafts and anything collected from a POP account, was
  never touched. Folders themselves were never removed.

  Every way of reading from a mail server now tells a refusal apart from an
  empty answer: listing folders, listing subscriptions, searching a folder,
  fetching messages, fetching one message, reading which messages changed, and
  asking what the server can do. A refusal stops that folder, says which folder
  and repeats what the server said about it, and nothing is removed from this
  computer. A folder that really is empty still empties here, as it always did.

  There is a second check behind that one. If a server says a folder holds
  messages and then lists none of them, the two answers disagree and nothing is
  deleted on the strength of it. The cost is one round of tidying: a folder
  somebody emptied from another program while this one was reading it has its
  rows cleared on the next mail check instead of that one.

  A connection that goes away in the middle of an answer is also no longer read
  as a complete answer.

  Two things are worse than they were, and both are on purpose. A folder whose
  messages the server will not hand over now fails that folder outright instead
  of quietly syncing less of it, and the same for a folder whose flags it will
  not read. A loud incomplete answer is better than a quiet wrong one.

  Known limitations: the library this program uses to speak to mail servers is
  where this comes from. Its own helpers stop reading a server's answer at the
  last line without looking at whether that line said yes or no, so a refusal
  arrives as no data. Two of its parsers do check, and those two are still
  used. The rest of the commands are now sent and read here instead. The same
  library has two faults in the part that watches a folder for new mail that
  cannot be worked around from here: a server that refuses to start a watch
  comes back with no reason given, and a server that rejects the request with
  no explanation at all crashes inside the library. The first now gets a
  sentence of its own so nobody is told a watch failed with nothing after the
  colon; the second is still a crash. None of this has been reported to that
  library's authors yet, and none of it has run against a real mail account.


- **Changing a day of a repeating event asks which days you mean, and can now
  change just the one.** Opening a day of a repeating event, or deleting one,
  puts the question first: every day in the series, or just this one day. Every
  day in the series is already chosen when the question opens, and that is where
  the keyboard lands, so the first thing a screen reader reads out is the answer
  Continue will take. The two answers are a set of radio buttons under a heading
  that asks the question, and the arrow keys move between them. Under each
  answer is a sentence saying what that answer will do to the calendar this
  event is filed in. Enter does nothing but close the question, because both
  answers act on somebody's calendar and one of them acts on every day of it.

  Just this one day is carried out for a calendar held on a calendar server and
  for a calendar kept on this computer. That day is taken off the series and
  kept as a separate entry. It is refused, with the series left exactly as it
  was, for a Google calendar, an Outlook calendar and a calendar this program
  can only read, and the refusal says which calendar and why.

  A repeating event filed in a Google or Outlook calendar now says, in the
  question itself, that how often it repeats is known to this computer only and
  that the calendar holds it as a single appointment.

- **Fixed: correcting a repeating event moved the day it starts on, and the
  earlier days disappeared.** Every day of a repeating event is one stored
  entry, and the editor was filled from the day on the screen but saved against
  the day the series starts from. The difference was read as a date somebody had
  typed. So opening the fortieth Tuesday of a weekly meeting, correcting the
  spelling of its name and saving moved the series onto that Tuesday, and the
  first thirty-nine Tuesdays were gone. Opening a day and saving without typing
  anything did the same thing, and marked the whole record to be sent back to
  the provider. The day that was opened now reaches the save, so the series
  keeps its own start. Typing a new date on a day of a series moves every day of
  it by that much, rather than moving the series onto the day you happened to be
  standing on.

  Known limitations: just this one day is written as a day taken off the series
  plus a separate entry, not as one occurrence moved. So other calendar programs
  reading the same calendar show two things rather than one moved day, and
  putting the day back into the series is not something this can do. Deleting
  just one day works the same way, and the day cannot be brought back. None of
  this has run against a real calendar server, a real Google account or a real
  Outlook account: what is tested is the parsing, the sentences, what is stored
  on this computer, and one test server running on this machine. Nothing has
  been checked with a screen reader.

  Known limitation, new: on a calendar held on a calendar server, changing just
  one day is now refused when the repeating event carries a time zone written
  the way Outlook and Exchange write them, such as Eastern Standard Time. This
  program cannot yet tell a calendar server what such a zone means, so the
  separate appointment for that day could never be created there. Nothing is
  changed and the day stays part of the series, and the refusal says so. Every
  day in the series still works on those events, and just this one day still
  works on events whose zone has an ordinary name such as Europe/London.

  Rough edge that goes with it: the question about which days you mean still
  reads out, under just this one day, that both go to your calendar server on
  the next sync. On one of these events that is not true, and you hear it
  before the refusal.

- **A contact group does something now.** Until now a group could be made, named
  and listed, and that was all it could do. You can now rename one, put a
  contact in one, take a contact out of one, and, the point of the whole thing,
  write to one: choosing Write to this group opens a new message with everybody
  in the group already on the To line.

  Everything is on the context menu, opened with the `Applications` key or
  `Shift+F10`. Press it on the groups tree in the contacts sidebar for Write to
  this group, New group, Rename this group and Delete this group. Press it on a
  contact in the list for Put in a group and Take out of a group. No new
  keyboard shortcut was added. The entries are listed under Contact Groups in
  [the keyboard shortcuts guide](KEYBOARD_SHORTCUTS.md).

  The sidebar now has a Groups branch of its own, and each group reads as its
  name and how many people are in it, such as "Team A, 3 people", rather than
  "Team A (3)".

  A member with no email address is left off the To line rather than sent as an
  empty recipient, and the announcement says so: "Writing to Team A, 2 of 4
  people. The others have no email address." The address it uses is the one on
  the contact's main line, so somebody whose only address is one of their extra
  ones is counted as having none.

  Known limitations: nothing has been checked with a screen reader, including
  what the new Groups branch does to where focus lands when the sidebar is
  filled again. Nothing here has run against a real account, and nothing needs
  to, because none of it touches one.

- **Contact groups are kept on this computer, and the product now says so.** A
  group made here is not sent to Google or Outlook, and a group you already keep
  there does not appear here. That has always been true and nothing said it.
  The window that makes a group now says it before you type a name, the privacy
  page has a section on it, and the sentence said after a group is made no
  longer names a mail account.

- **A person's name survives a trip to an address book and back.** The two parts
  of a name, the given name and the family name, are now kept as they were given
  rather than worked out from the whole name each time. A family name that
  carries a space, such as "van der Berg", stays whole, and a middle name stays
  in the given name where somebody put it.

  What went wrong before: the whole name was split at its last space every time
  it was sent. "Grace Brewster Murray Hopper" went out as a given name of "Grace
  Brewster Murray", and a person whose address book held "Grace" and "van der
  Berg" got them joined into one line here and split back as "Grace van der" and
  "Berg". Each round trip could change the name again.

  The contact editor now has a Given name box (Alt+G) and a Family name box
  (Alt+M) on the Basic Info tab. For a contact whose parts were never recorded,
  they open filled with one guess at where the name divides, so you can see the
  guess and correct it. Nothing splits a name after that, ever.

  Known limitations: none of this has run against a real Google or Outlook
  account. Whether a screen reader announces the two new boxes correctly has not
  been checked with a screen reader. Clearing a family name here does not clear
  it at Outlook, which is how Outlook has always treated a field left out.

- **A birthday with no year is read out as a day and a month.** A birthday
  recorded without a year is stored as "--03-14", which is what a contact card
  writes and what an export needs, and a screen reader reads that one character
  at a time. The contact details now say "March 14th", or "14th March" where
  this computer writes the day first, and the reading aloud of a contact says the
  same. A birthday that does carry a year is read as a whole date and never as
  "2 days ago", because how long ago somebody was born is not the question.

  Known limitations: the Birthday box in the contact editor still shows the
  stored form, on purpose, because that box holds the value being edited and
  typing words into it would store the words. Month names are English whatever
  language this computer is set to, so on a non-English machine a birthday is
  now two English words in a row rather than one.

### Changed

- **Sending a message and changing a mailbox are now tried out against a mail
  server before you get them.** Until now nothing checked what these actually
  put on the wire. The only thing measured was the refusal: with Allow Changes
  off, nothing left the machine. What happened with it on had never run
  anywhere.

  A mail server that answers now runs inside the tests, and every one of these
  is sent to it and read back: sending a message, sending a read receipt,
  starring, marking read, copying, moving, deleting, saving a copy of a sent
  message, and replacing a filed draft. Three things that were only written
  down in a comment are now checked. Everybody a message is addressed to gets
  a line of their own at the server. A blind copy reaches the server and
  appears in neither the message every recipient reads nor the copy filed in
  Sent. A read receipt leaves exactly as it was written.

  This does not mean sending has been tried against a real account. It has
  not. What is proved is what goes out and what a server we wrote answers, and
  Gmail, Fastmail and Exchange each have their own opinions about all of it.
  The Allow Changes warning stays exactly as it is.

  Known limitations. A message sent from here goes out with no identifier of
  its own, and most mail carries one. The date is filled in and the identifier
  is not. Two things follow: the copy kept in Sent is filed without one, so
  replying to your own sent message starts a new conversation instead of
  continuing the old one; and where your provider adds an identifier as the
  message leaves, which the large ones do, the copy in your Sent folder and
  the copy your recipient received no longer agree about which message they
  are. Choosing what that identifier should say is a decision about what every
  recipient sees, so it is written down here rather than guessed at.

  Second known limitation, in the same area. On a mail server that cannot move
  a message in one command, moving or deleting one is three steps: copy it,
  mark the original, remove the original. The order is right, and no single
  step failing loses the message. But when the last step fails you are told
  only that the delete failed, and the message list keeps its row. The copy is
  in the Trash, the original is in the folder marked to go, and nothing says
  so. Getting that sentence right is its own piece of work.

- **What an imported card says now reaches your address books.** Import
  Contacts used to write what a card said into the record here and stop there.
  The next read from Google or Outlook wrote that address book's own copy back
  over it, so a card corrected a contact until the next sync undid the
  correction, and nothing said so. It was not even consistent: a card folded
  into somebody who already had a change waiting went out with that change, so
  one file could send one person's details to Google and leave the next
  person's here.

  Importing a card is something you did on purpose, the same as an edit in the
  contact editor, and an edit is sent. So an import is now treated as one.
  What a card changed is queued for every address book that holds the person,
  and a card for somebody nobody holds yet is offered to your address book the
  way a contact typed here already is.

  What that costs, said plainly. Importing a file of old contacts puts those
  people in your real Google or Outlook address book. The import says so at
  the time rather than leaving you to find out from the next sync: "Imported
  40 contacts. 40 contacts are waiting to be sent to your address book."
  Whether they may go is one of the things Allow Changes decides, and the sync
  says what it is holding back.

  A card that changed nobody sends nothing. Reading the same file again, or
  reading a backup of what is already here, is not a change and is not queued.
  That holds for a contact stored before Wixen Mail kept lists of addresses,
  whose one address and one number read back from a card as a list of exactly
  those; read as a change, importing a backup would have sent every contact an
  older version had stored.

  Known limitation: none of this has run against a real Google or Outlook
  account.

- **One name for the setting that decides what may be sent: Allow Changes.**
  A sync tells you to turn on Allow Changes. The settings section was headed
  Allowed Changes, and the two boxes in it started "Let Wixen Mail". Three
  wordings for one thing, so somebody who heard the sentence and went looking
  for it had to work out whether they had found the right place.

  The section is now headed Allow Changes, and the boxes read "Allow Wixen
  Mail to change my tasks, contacts and calendar" and "Allow Wixen Mail to
  send and delete mail". The choices on the first-run screen use the same
  word, and so does the testing page. The heading and the sentence come from
  one place in the code now, so they cannot drift apart again.

  Nothing about what is allowed has changed. These are the words only.

- **A phone number or an address with one label keeps it.** A label was only
  ever kept when a contact had two or more of something, so a person with one
  number labelled Work had that word thrown away on the way in. It was worse
  than losing it: opening such a contact in the editor invented a label, calling
  a lone number "Mobile" and a lone address "Personal", and saving sent that
  invented label back to the address book. And a label you chose here for your
  single number was deleted from this computer on the very next check, because
  the address book sent one number back and one number meant no list.

  This was a deliberate choice when it was made and it has been reversed. A
  contact from Outlook, which sends no labels of its own, now stores "Other" for
  its one address rather than nothing, which is what stops the editor inventing
  "Personal" for it.

  Known limitations: none of this has run against a real account. A number an
  address book gave no label at all is now stored as "Other" and sent back as
  "other", so the first change to such a contact writes a label where there was
  none. It settles after that and does not keep changing.

- **A message you send is kept somewhere, even when the server will not keep
  it.** Sending put a copy in the account's Sent folder on the server and wrote
  nothing on this computer. When that failed the failure went to a log file, the
  message had already gone, and the copy then existed nowhere at all. There were
  five ways to reach that: an account that has never checked for mail and so has
  no Sent folder yet, an account set up with sending but no usable receiving
  port, a connection that drops between sending and filing, a sign-in that
  expires in the same gap, and a server that refuses the copy because the
  mailbox is full or the message is too large.

  The copy still goes to the server first, because that is what puts it on every
  device. If the server will not take it, a copy is saved in Sent on this
  computer instead, with its text, and you are told what the server said. If
  there is nowhere at all to put it, you are told that too, and told to check
  for mail once so the account learns where its Sent folder is.

  Known limitation: none of this has run against a live mail server.

- **A setting to keep your own copy of sent mail on this computer.** On the
  Compose tab of Settings: "Keep a copy of sent mail on this computer, even when
  the server saves one". Off unless you turn it on, and off in settings files
  written before it existed. With it on, the message is in Sent the moment it
  goes, without waiting for the next check for mail. The cost is that Sent then
  lists each message twice once the server's own copy comes down, which is what
  the setting does rather than a fault. The box says so, and so does the
  description a screen reader reads after the label.

  Known limitation: whether a screen reader reads that description has not been
  checked with a screen reader.

- **A copy saved on this computer is no longer deleted by the next check for
  mail.** This is what made the change above possible rather than a way of
  losing mail more slowly. Checking for mail compares what the server lists
  against what is stored, and anything stored that the server does not list is
  removed, along with its text. A copy saved here was never on the server, so it
  read as a message the server had dropped. Rows this program writes itself now
  carry a mark, and neither the ordinary removal pass nor the one that runs when
  a server renumbers a mailbox touches a marked row. A saved copy is also left
  out of the comparison entirely, so the count of what a check removed no longer
  includes messages nothing removed, and the server is not asked about a message
  it has never had.

- **A reply you sent still sits in the conversation it answers.** The copy saved
  on this computer recorded nothing about what the message replied to, and did
  not record where the sender asked replies to go. So a reply filed here started
  a fresh conversation in your own Sent folder, and replying to it went to your
  own address. It now carries the same chain of ancestors a downloaded message
  carries, through the same code, so the two cannot disagree. A message that has
  no date of its own is filed under the time it was sent, rather than with a
  blank date that sorts it to an end nobody reads.

  Known limitations, both unchanged from how mail collected over POP has always
  been filed: attachments are recorded as present but their contents are not
  saved, so a saved copy says it has attachments and cannot list them. And an
  account on a provider that files its own copy of everything it sends, which
  Gmail does, already ends up with two copies in Sent. Nothing here checks for
  that, because skipping the copy on a guess about a provider risks the copy
  existing nowhere, which is the failure this release closes. With the new
  setting on, such an account would have three.

- **Delete works on a POP account.** Pressing Delete on mail collected over POP
  refused, and the reason it gave was about a mail server setting that had
  nothing to do with it. Delete now moves the message to that account's Trash
  folder on this computer, and Delete again from the Trash, or Shift+Delete
  anywhere, takes it out of every folder here.

  Nothing is removed from the POP server by this. Mail stays there until the
  account's own "Leave mail on the server" setting takes it, which is unchanged
  and still leaves everything by default. The wording says "on this computer"
  every time for that reason.

  The account dialog has a new box, "Let me delete mail on this computer",
  ticked unless you untick it. Somebody who has told the account to clear the
  POP server after downloading has this computer as the only copy, and this is
  how they say Delete must not be the thing that loses it. Unticking it makes
  Delete say so, by name, instead of blaming a server.

  Known limitations: none of this has run against a live POP server.

  Deleting does not erase. The message leaves every folder, count and search,
  and nothing here brings it back, but the message and its text stay in this
  computer's mail database, which is not encrypted. Two things need them to
  stay. The number the POP server knows the message by is how the next check
  knows not to download it again, and the text was downloaded once with no
  server to fetch it back from. Nothing clears either yet, and the sentence
  after a delete says "Deleted from this computer" without saying any of this.
  If you delete mail because somebody else will use the machine, that is not
  what this does.

- **A message deleted on a POP account stays deleted.** Checking for mail asked
  one folder what had already been downloaded, so anything moved to the Trash
  looked like mail that had never arrived and came straight back on the next
  check. It now asks across the whole account, including messages that have been
  deleted. The same fix means the "remove from the server after so many days"
  setting keeps counting for mail that has left the inbox, where before that
  mail silently never left the server at all.

- **Every account has an Outbox you can open.** The folders that live on this
  computer were only created part-way through checking for mail over POP, so an
  IMAP account never got an Outbox at all and a POP account only got its folders
  after the first check that worked. They are now made for every account when it
  is read on the way in. What the Outbox shows is the send queue itself, which
  is still the one source of truth: a message waiting appears there, says what a
  failed attempt reported, leaves when it goes, and Delete in that folder takes
  it out of the queue.

- **The open Outbox is read again when sending finishes.** Somebody watching it
  saw rows for mail that had already gone until they left the folder and came
  back.

- **A POP account's Junk folder can be opened.** It has existed in the database
  since local folders shipped and has never been reachable: the rule that keeps
  a server's spam folder from being downloaded was also being applied to folders
  on this computer, which have nothing to download.

- **Mail collected over POP is read for signs of an impersonation.** Only the
  message's headers were looked at, so a link whose words and address disagree,
  an address made to look like somebody else's, or pressure to act at once
  produced a warning on an IMAP account and silence on a POP one, with nothing
  saying which account you were on. The same reading now runs on both, from the
  message the download already has in hand.

  Links are also now checked against Google's lists on POP mail, if you have
  switched that on. It is off unless you ask for it and unchanged in what it
  sends.

  This will produce some false positives. The reading can never call something
  a phishing attempt on its own: the strongest word it uses is "suspicious",
  which is the existing guard against a warning becoming the thing people click
  past.

- **A setting for that reading: "Read each message on this computer and mark
  suspicious ones".** In Settings, then Advanced. On unless you turn it off,
  because it sends nothing anywhere and it is what mail arriving over IMAP has
  had all along. It sits beside the Google Safe Browsing box and is deliberately
  a separate switch: that one can put four bytes of a link on the wire, and
  sharing a switch would mean agreeing to it to get a check that touches
  nothing. `docs/privacy.md` says what each of them does.

### Fixed

- **A meeting from a calendar server whose time zone name itself contains a
  colon showed the wrong start and end time, or a broken time zone name.**
  Microsoft Exchange and Office 365 write some time zone names this way, for
  example `(UTC-05:00) Eastern Time (US & Canada)`. The calendar standard
  requires such a name to be quoted, precisely because it holds a colon, and
  this program was reading only as far as the first colon in the line,
  quoted or not. Such a meeting now reads its start, its end and its own
  time zone correctly, and saving a change to it now writes the time zone
  name back out quoted, the way it arrived.

- **An event whose calendar server says how long it runs instead of giving a
  separate end time showed no end time at all.** Radicale, one of the
  calendar servers this program connects to, is one that writes events this
  way. Such an event now shows the correct end time, worked out from when it
  starts and how long the server said it lasts. Saving a change to such an
  event also no longer leaves the server's own line saying how long it runs
  sitting beside the new end time, which the calendar standard does not
  allow together on one event.

- **When a row had gone, or a command went wrong, you were told almost
  nothing.** Eight places said "That row is no longer there", which does not
  say what row or whether anything happened, and half of them left out the
  part saying nothing had been changed. All of them now name what has gone
  and say the rest is untouched, in the same words wherever you are.

  Separately, every command in Contacts, the calendar, Tasks, Notes and
  Reminders failed with one sentence, "That did not work", followed by
  whatever the storage said. By ear a failed delete and a failed move were
  the same words, the row was never named, and there was nothing in it to
  act on. It now says what was being done, to which row, why it did not
  happen, that nothing has been changed, and what to try.

- **The message saying one day of a repeating event could not be kept on its
  own described something that had never happened.** It opened "That one day
  was not kept", which reads as though the change had gone to your calendar
  server and been turned down. Nothing had been sent: the answer is worked out
  before the editor even opens, and again before anything is written. It now
  says the day cannot be kept, and it ends by telling you what to do about it,
  which is to change the event's time zone to one this program knows and then
  open the day again.

- **A draft your mail server would not take could leave no copy on the server
  at all, and nothing said so.** Saving a draft again replaced the copy in the
  server's Drafts folder, and it did that by taking the old copy away first and
  offering the new one second. When the server then turned the new one down,
  there was no copy of the draft on the server, so it was gone from your phone
  and from anything else reading that account. Both failures went to a line in a
  log file. Nothing on the screen changed and nothing was read out.

  Now the copies already there are found first, the new one is saved second, and
  only then are the older ones taken away. At no point in that order is the
  server left without a copy. When something goes wrong you are told which of
  four things happened: the draft is on this computer and the server has the
  older copy, the draft is on this computer and nowhere else, the draft was
  saved but an older copy may still be beside it, or it all worked. Only the
  last one stays quiet, because saving happens once a minute while you write.

  The third of those is what you hear on a mail server that can only mark the
  old copy and leave it in the folder. Such a server is now told apart from one
  that really took the old copy away, by counting the copies that went rather
  than the copies that were asked about, so the two drafts your other devices
  show are at least explained. They are still both there and this program will
  not remove the older one: the only removal such a server offers takes every
  message in the folder that anything has marked for removal.

  A draft that could not be put in the Drafts folder on this computer, which is
  what a POP account uses, is now said out loud too. It was also only logged.

  Your work was never at risk in any of this. The draft is written to this
  computer before the server is asked, so what was lost was the copy your other
  devices see.

- **A delete or a move the server only half carried out was announced as one
  that never happened.** On a mail server that cannot move a message in one
  command, moving one is three steps: copy it, mark the original, remove the
  original. The copy goes first so that nothing is ever lost. If the server then
  refused to mark or to remove the original, you were told the message was not
  moved and not deleted, and the list kept showing it where it had always been.
  The server had it in two folders at that point, and on another device you
  would have found both.

  Now each of those endings has words of its own. You are told the copy is in the
  Trash, or in the folder you chose, and that the message is still in this folder
  as well, and why. Where the original was left untouched the sentence says that
  trying again would make a second copy, because nothing anywhere removes
  duplicates. Where the server marked the original, the row is marked here too,
  so the list and the server say the same thing.

  A failure now means one thing only: nothing on the server changed. The message
  is exactly where it was and trying again is safe.

- **Delete destroyed the message on any account whose Trash folder this program
  did not recognise.** Delete is meant to move a message to the Trash, where it
  can be got back. Working out where the Trash is gave one answer for "somebody
  asked for this to be gone for good" and for "none of this account's folders
  looks like a Trash to me", and that answer was to mark the message and remove
  it from the server with no copy kept anywhere. The status line then said
  "Deleted", which was true. A server that names its trash in another language,
  a provider that calls it something of its own, or an account whose folders had
  never been read was enough to get there.

  Now nothing is deleted in either of those cases. The message stays exactly
  where it was, and a message says so and says what to do next: move it to the
  folder this account keeps deleted mail in, or use Delete Permanently to take
  it off the server for good. An account that has never checked for mail is told
  to check once first. Delete Permanently is unchanged and still works on an
  account nothing is known about, because that command means it.

  Naming your own Trash folder per account is not possible yet, so an account
  whose Trash this program does not recognise has no ordinary Delete until that
  exists. None of this has run against a real mail server. What is tested is the
  decision, the sentences, and the order the handler does things in.

- **A change your mail server turned down was reported to you as done.** Two of
  the commands this program sends to a mail server, setting a flag on a message
  and removing one message, were sent in a way that never read the server's
  answer. A server saying no came back looking exactly like a change that
  worked.

  Every flag this program sets went through one of them, so starring a message
  or marking it read could be refused and reported as done, and you would find
  out days later from another device. The other is the command that removes a
  message, so on an account whose mail goes straight out rather than to a
  Trash folder you could be told "Deleted" over a message still sitting in the
  folder. Both now report a refusal as a refusal.

- **The privacy page said nothing here sends your contacts, your calendar or
  your tasks to anybody. They are sent.** It opened by saying that nothing in
  this program sends your messages, your contacts, your calendar or your links
  to anybody. That was wrong about three of the four: with the answer a new
  installation starts on, changes to your contacts, your calendar and your
  tasks are sent to the provider you signed in to. Sending mail to a provider
  is the one thing a new installation refuses. A table five lines further down
  the same page said so, and the sentence above it said the opposite.

  The page now says what goes where, in about the same few lines it took
  before. It also names the other thing that leaves your computer without you
  asking for it: showing a message that points at a picture asks that address
  for the picture. The page explained that further down and the opening summary
  did not mention it.

  The check that holds every other claim of this kind to what the code does now
  reads this one too. It could not before. It looked for a sentence carrying a
  word that puts it at installation time, such as "by default", and a sentence
  saying that nothing is sent to anybody carries no such word, because it has
  already claimed every setting there is.

  Other absolutes went the same way. The privacy page and the installing guide
  both said nothing is left behind after an uninstall. The uninstall can fail
  to remove something, and writes a note in your temporary folder saying what
  it could not remove, so both pages now say that instead. The architecture
  notes said nothing sensitive is written to the database, on a page that also
  says the cached mail is not encrypted; that sentence now says credentials,
  which is what it meant.

- **A task you delete stays deleted when the list it was in disappears.** When a
  task list went away at your provider, every deletion still waiting for that
  list was thrown away with it. If the provider had moved one of those tasks
  into another list rather than deleting it, the next sync read the task back
  and put it on the screen again, with nothing left to say you had ever deleted
  it. Deleting a task and finding it back reads as the delete having quietly
  failed, and a row that returns says nothing when it does, so it is easy to
  miss altogether.

  Whether a deletion can be forgotten is now decided in one place, beside the
  record it clears, instead of separately by each part of the sync. A deletion
  of a task your provider named is kept until that provider has taken it. A
  deletion of a task made here that was never sent anywhere is still cleared, so
  nothing is remembered for ever with no reason to be.

  Known limitation: a deletion whose list has gone can never be sent, because
  there is no longer anywhere to send it. This computer stops asking, so it no
  longer reports a problem after every sync about something you cannot act on.
  If your provider still holds that task in some other list, it keeps its copy
  while this computer shows nothing.

- **Two pages stopped saying that everything is held back until you say so.**
  The comparison page promised that nothing writes to a server before you
  permit it, and the front of the testing page said sending was switched off
  from the first run. Both were true when they were written, and neither is
  true now: a new installation allows changes to tasks, contacts and the
  calendar to go to your provider, and only mail is held for you. The table
  on the testing page has said the true answer all along, nine lines below a
  sentence saying otherwise. Both pages now say which half starts on, which
  half starts off, and why the two answers are split.

  The check that compares prose with the code's answer used to read only
  prose that names Allow Changes, and neither page does, which is how these
  two wordings walked past it and became the tenth and eleventh of their
  family. It now also reads a sentence that says something is written and
  names where to, whatever the sentence calls the setting, if it calls it
  anything. It reads what stands before an "until" as the answer a fresh
  copy gives. And it tells the word for a program starting from the word for
  a value starting. Both wordings are pinned in its corpus beside the true
  sentences that have to stay quiet.

- **The status page and the README answer "does this work yet" truthfully
  again.** Both said mail cannot be received, that IMAP and POP3 do no
  network work, that OAuth sending is missing, that a draft cannot be saved,
  that a calendar cannot be added by its address or written back to, and
  that a repeating event is shown once. Every one of those became false as
  the work landed, and the pages kept saying it. Both also named versions
  long gone and test counts far below the real ones. They now describe what
  is built, date-stamp every measurement they quote, and name no version at
  all, and a new check refuses a version on either page that the code does
  not ship.

- **Events sent to a calendar server now carry the time zone rules their
  document relies on.** The calendar format requires it: every time written
  in a named zone must be able to find that zone's rules in the same
  document. Documents written here named the zone and defined it nowhere,
  which a strict server refuses outright and a lenient one quietly guesses
  at. A new event now carries the rules for every zone its document names,
  and a change to an event whose server copy left the rules out adds them.
  A zone that moves its clocks on no yearly pattern has its change days
  listed one by one instead of forced into a pattern that would misplace
  meetings. An event naming a time zone this program has no rules for is
  held back with a message naming the zone, rather than sent as a document
  the server could misread; it stays waiting until the event's time zone is
  changed to a known one.

- **A cancelled day of a repeating meeting keeps its exact moment on the way
  to and from a calendar server.** A series can mark its cancelled days two
  ways at once: as an instant in universal time, or as a clock face in the
  meeting's own time zone. Sent to the server, all of them went out on one
  line under one label, chosen by looking only at the last value, so a
  cancelled day could claim universal time and a zone at the same time, which
  a careful server refuses outright, or lose its zone and sit an hour wrong
  for half the year. Each kind now goes on its own line, labelled as the
  value itself says. Reading is fixed the same way: a cancellation that
  arrives naming a different zone from its own meeting used to have the zone
  thrown away, which quietly renamed the moment; it is now converted into the
  meeting's zone, so the day that goes quiet is the day that was cancelled.
  The meeting's own times are never converted: a nine o'clock meeting stays
  at nine o'clock when the clocks change.

- **A meeting cancelled in a time zone this program cannot read now keeps that
  zone.** Outlook and Exchange write time zone names of their own, such as
  "Eastern Standard Time", which are not the names this program looks times up
  by. A day cancelled in one of those had the zone thrown away, and the next
  time the meeting was sent back to the server the bare clock face was dressed
  in the meeting's own zone: four to five hours from the moment the server had
  stated, so the wrong day went quiet and the cancelled one came back. The
  cancelled day now keeps the zone it arrived in and goes back out saying the
  same thing, and the server's own description of that zone travels with it.
  The same is true of a cancelled time that falls in the hour the clocks skip,
  which names no moment at all: it is kept exactly as written, in the zone it
  was written in, and noted in the log rather than passed over in silence.

  Two honest limits. A meeting synced before this fix already lost the zone,
  and nothing can work out what it was; reading the calendar from the server
  again repairs it. And a new event whose cancelled day names a zone this
  program has no rules for is now held back with a message naming that zone,
  the same as an event whose own time zone is unknown, instead of being sent
  at the wrong hour.

- **A calendar event you deleted could come back; now it stays deleted whatever
  kind of calendar held it.** The record saying "this was deleted here" was
  thrown away whenever the deletion had nowhere to be sent: an event in no
  calendar, in a calendar the account may only read, in a calendar made on
  this computer, or in a subscribed feed. The provider or the feed still knew
  the event, so the next sync wrote it back onto the screen as though the
  deletion had quietly failed. The record is now kept for as long as anything
  could hand the event back, and the feed refresh, the one read that never
  asked what was deleted here, asks now. An event at a calendar server whose
  stored address is missing is covered the same way: the deletion cannot be
  sent, but it no longer comes back.

  Known limitations: a record kept this way is kept for good, because nothing
  ever takes the deletion off this computer's hands. That is the price of the
  event staying deleted. And deleting an event out of a calendar the account
  may only read deletes it on this computer only: the provider still holds it,
  and this program cannot ask for it to go.

- **A task change held back by Allow Changes is no longer called a problem.**
  With Allow Changes off, a task you had edited or deleted here was reported
  as an error after every sync, forever: "1 problem" on the status line, with
  nothing saying what would fix it, about a change that was simply waiting on
  the setting. The sync now counts it as waiting and says so in a sentence
  that names the setting: "1 change is waiting here: turn on Allow Changes in
  Settings to send it". Turning the setting on sends the change on the next
  sync, with nothing typed again. Calendar and contact changes already said
  this; task changes now say the same sentence.

- **The working-day hours apply as soon as they are saved.** The calendar's
  note about an event before or after the working day was built from the
  hours read when the application started, so changing them in Settings
  looked like a setting that did nothing until the next start. The rows now
  use the hours that were just saved.

- **A date without a time is spoken as a date again, not as hours since
  midnight.** Under the relative date style, the one the product ships with, a
  task due on a day was announced "Due: 12 hours ago" at noon of that very day,
  as though it were already overdue, and the same counting reached events,
  reminders, notes and the reminder alert. A date with no time on it is now
  always said as its date, the way a birthday already was. Values that do name
  a time keep the relative wording: "3 hours ago" about this morning's meeting
  is that style doing its job.

  Two neighbours of the same mistake went with it. The calendar's note about
  an event outside the working day now judges the same hour the cell speaks,
  on this computer's clock, and a start value that is not a real time earns no
  note rather than being read as an hour of the day. And a reminder set for a
  day is no longer announced as overdue while that day is still going; it
  becomes overdue once the day has passed.

- **Enter no longer answers yes to a deletion you have not finished hearing.**
  The question asked before deleting a contact, a task, a note, an event or a
  reminder, and the one asked before deleting a list, a calendar or a notebook
  with everything in it, both arrived with Yes waiting on Enter. That is what a
  Windows message box does unless it is told otherwise. Press Enter partway
  through the question and the thing was gone, and none of it can be undone.

  Both questions now open with No waiting on Enter, so answering without
  hearing the end of it does nothing. Yes is still one Tab or one letter away.

  Anybody can be caught by this and it costs more if you are listening rather
  than reading: hearing a question takes longer, so there is more of it left
  when your finger moves, and Enter is how you answer everything when you work
  by keyboard, so it is already on its way.

  The composer's question about a message the spell checker has doubts about
  still answers Enter with yes, and that stays deliberate. Somebody who meant
  to send and heard the warning should not have to go looking for a button, a
  message can be written again, and the whole question is in the words.

  Known limitation: this has not been through a screen reader. What was
  measured is the style the code hands the message box.

- **Settings no longer offers three answers nothing kept.** Each was on the
  screen with a value a screen reader read out, and nothing anywhere saved that
  value or read it back, so what it said was a setting somebody could not have.

  "Default format", offering HTML or Plain Text, is gone from the Compose tab.
  There is no such choice to make: one editor writes every message and nothing
  asks which was wanted.

  "Enable threaded view by default" is gone from the Reading tab. Threaded view
  is not built. Its View menu item is there and disabled and says so, and a
  default for something that cannot be switched on is a default for nothing.

  "Load remote images in messages" is gone from the Reading tab, and a sentence
  saying what happens stands where it was. It sat unticked, which reads as a
  promise that pictures are not fetched, and nothing honoured it either way.
  What happens: a picture a message points at rather than carries is left in
  the message, and the preview pane and the conversation window show a message
  in a browser, which fetches the picture and so tells the sender the message
  was opened. The reading window, which is where a message opens when you press
  Enter on it, shows text and fetches nothing.

  Known limitation: there is still no way to stop that fetch. Blocking it is a
  feature nobody has built, and the Reading tab now says so rather than
  implying it is done. [What Wixen Mail sends, and where](privacy.md) now has
  a section on it. What is written here was read out of the code, not
  measured on the wire.

- **The signature setting on the Compose tab does something now.**
  "Automatically insert signature on new messages" was built ticked, handed
  back by nothing and saved by nothing. A screen reader announced it as on, it
  could be ticked and unticked, and every message got the account's signature
  either way, so what somebody was told and what happened were two different
  things.

  It is a real setting now, and its label says what it covers: "Start every
  message with my signature". A reply, a forward and a message written to a
  contact all open with the signature as well, so the old label was narrower
  than what was happening. It arrives ticked, and a settings file written
  before it existed reads as ticked, which is the behaviour every message has
  had. Unticked, a message opens empty. The signature stays on the account,
  the signature manager still edits it, and it can be put in by hand.

  Known limitation: this has not been through a screen reader. What was
  measured is the text the code produces and that the window reads the setting.

- **Importing an address book that splits a person across cards no longer makes
  two contacts of them.** Several address books export one card per address
  rather than one card carrying the whole list. What says two records are one
  person here is an address they share, and two such cards share none, so unless
  you already hold that person at both of them, both cards were written down.
  Every person their address book had split arrived here twice, and both halves
  were queued to go to your real address book.

  Two cards in one file are now one person when they are the same card apart
  from the addresses on them, which is what a per-address export writes. Anything
  the two cards disagree about keeps them apart, so two people who happen to
  share a name are still two people. A card with no name on it is never joined to
  anything. This applies to cards in the file being read, never to a contact you
  already hold: an address book is entitled to hold two people with one name, and
  folding a card into one of them would put a stranger's address on somebody
  real. A card that matched somebody you already hold is still a card in the
  file, so whether the two join does not turn on which of them matched her.

  The count now says how many people the file wrote down rather than how many
  cards did the writing. Two cards for one person used to be announced as
  "Imported 2 contacts. 2 contacts are waiting to be sent to your address book"
  with one contact in front of you.

  Known limitation: two people with the same name, and nothing recorded for
  either but one address each, are read as one person holding both addresses.
  Nothing is deleted and mail to either address still reaches somebody, but it is
  the wrong answer. The joined card is queued like any other import, so with
  Allow Changes on it is then sent to your real address book. The narrower rule
  that avoids it, asking the cards to agree on something besides the name, would
  leave the commonest shape of export there is, a name and an address,
  duplicating everybody.

  How this was measured: through the import, with the cards a per-address export
  writes, with two people who share a name, with a card that matches somebody
  already stored, and with cards carrying no name. Nothing has run against a real
  address book.

- **A whole-day event sent to a calendar server is written as a day.** The
  document writer turned a whole-day event's stored value into a date by taking
  the dashes out of it. That works for an event made here, which stores a bare
  date, and not for one that came from Google or Outlook, which stores the day
  in one place and midnight in another. The writer read the second, so a
  birthday went out as `DTSTART;VALUE=DATE:20260727T00:00:00Z`, which is not a
  date. A server that checks what it is sent refuses the whole change.

  Nothing carries such an event to a calendar server today, because moving an
  event that Google or Outlook already holds into another calendar is refused
  before it starts. This is fixed rather than left to that refusal, so that
  changing what moving an event is allowed to do does not quietly bring it back.

  How this was measured: through the document writer, with the day held in each
  of the four shapes this program stores one in. Nothing has run against a real
  calendar server.

- **An event whose zone name is blank no longer goes to two places at two
  different times.** A stored event carries a zone name beside its clock face. A
  name that is empty, or nothing but spaces, names no zone, and four different
  pieces of this program each decided for themselves what to do about that. They
  gave three different answers.

  What that did. Sent to Google, an event whose zone name was a single space
  went out as a clock face in a zone called " ". Sent to Outlook, the same event
  went out as an hour on this computer turned into universal time. Those are
  different moments, so one event was in two calendars at two different times,
  five and a half hours apart for somebody in Kolkata. Sent to a calendar
  server, the same event produced `DTSTART;TZID=:20260305T090000`, which is not
  a calendar document at all: a server that checks what it is sent refuses the
  whole change, and one that does not stores a meeting in a zone whose name is
  nothing.

  There is now one answer, in one place, and all four ask it. A name of nothing
  but spaces means no name, which sends the hour the event really means. A name
  with spaces around it is still the zone it names, so " Europe/London " is
  Europe/London rather than a refusal.

  How this was measured: through each of the four writers, with the name stored
  as empty, as one space, as several and as a tab, and by comparing the moment
  the two providers were given rather than only the shape each was sent. Nothing
  has run against a real account or a real calendar server.

- **A cancelled day of a repeating meeting is no longer shown.** Servers write a
  cancellation in one of two ways: as a day and time in the meeting's own zone,
  or as the same instant written in universal time. Only the first was honoured.
  The second was compared as text, eight characters at a time, so a meeting at
  one in the morning in Kolkata cancelled as half past seven the evening before
  in Greenwich was compared with the wrong date and stayed on the calendar. It
  works the other way too: an evening meeting in New York, cancelled as the
  following morning in Greenwich, kept the cancellation and lost the meeting.

  A cancellation that names an instant is now turned into a day using the
  meeting's own offset, so both spellings of one cancellation mean the same
  thing. This affects meetings from a calendar server and from a subscribed
  calendar feed, which are the two sources that keep the cancellation as the
  server wrote it.

  Known limitation: a meeting stored as a clock face with a zone name beside it,
  rather than one carrying its own offset, still has its cancellations compared
  as text. Turning an instant into a day there needs the zone's summer-time
  rules, which this program does not carry.

  How this was measured: through the code that works out which days a repeating
  meeting falls on, with the meeting and the cancellation written the way a
  server writes them. Nothing has run against a real calendar server.

- **An event, a task or a contact you delete no longer comes back on the next
  sync.** The note saying you had deleted something was thrown away the moment
  Google, Outlook or your calendar server accepted the deletion. The read that
  runs straight afterwards in the same sync then found the provider still naming
  the thing, found nothing stored here under that name, and wrote it back down.
  It was on the screen again under the provider's own identifier, with nothing
  left to say it had ever been deleted, which reads as the deletion having
  quietly failed.

  This was live for calendar events at Google, at Outlook and at a calendar
  server, and for tasks at Google and at Outlook. Contacts were fixed in the
  previous round of changes, but only for the sync that did the deleting: run
  the same sync again with the address book still naming her and she came back.
  All three now follow one rule, written down in one place.

  The rule is that something deleted on this computer is remembered until the
  provider has taken the deletion, and for seven days after that, and no read
  writes down anything the memory names. A deletion nobody has taken yet is not
  a memory, it is work still owed, and it is kept for as long as that stays
  true.

  What finally lets a memory go is the clock and nothing else, so the record
  drains rather than growing for ever. Seven days outlasts a provider's list
  answering from a copy written just before the deletion landed, and covers the
  machine being shut for a weekend before the next sync runs.

  Known limitations. A provider that takes a deletion and then goes on listing
  the thing for more than a week will hand it back, and nothing here can tell
  that apart from somebody creating it again elsewhere. Deleting an event from a
  subscribed calendar feed is a different case and is not covered: a feed is
  published somewhere else and only read here, so nothing can delete anything in
  it, and the event returns on the next refresh.

  How this was measured: the whole way round in each sync, from deleting the
  item, through the provider answering the deletion with success, to the
  provider's next list still naming it, in the sync that did the deleting and in
  the sync after it. None of this has run against a real account.

- **A date stored by Outlook, or by this program's own event editor, is read out
  as a date again.** An event whose time is stored as `2026-07-27T09:00:00` was
  handed back unchanged, so the words the reading asked to have said were that
  string. It now reads "July 27, 2026 at 9:00 AM".

  Two separate things put values of that shape into the cache. Microsoft Graph
  writes the time on every event as `2026-07-27T09:00:00.0000000`, and the code
  that turns a stored value into words has never known that shape, so every
  event from an Outlook calendar has read out this way for as long as Outlook
  calendars have worked. The event editor here started writing
  `2026-07-27T09:00:00` in the previous round of changes, so events made and
  moved on this computer joined them.

  Underneath, the part that reads a stored time and the part that writes one
  each kept their own list of the shapes a stored moment can take, and the two
  lists disagreed by exactly those two shapes. There is now one list, in one
  place, and everything that reads a stored time uses it.

  The same change fixes a reminder that never went off. The reminder code kept a
  third list, of two shapes, and neither had a `T` in it, so a reminder whose
  time came from Outlook or from the event editor was read as no time at all and
  was passed over silently on every check.

  A whole-day date still reads as a day with no clock reading on it, and a
  stored value that is none of the known shapes is still said as it stands
  rather than dropped or guessed at.

  How this was measured: the whole way round, from moving a time in the event
  editor, through the value that lands in the stored column, to the words the
  row hands to the announcement when you press Space on it.

  Not verified: no screen reader run. What was checked is the text asked to be
  said, not how any particular screen reader says it.

- **Importing two cards for one person no longer makes two contacts and loses an
  address at your address book.** Several address books export one card per
  address rather than one card carrying the whole list. Reading a file like
  that, the first card replaced the addresses stored for that person with the
  one address it named, so the second card matched nobody and was written down
  as a new contact. Both rows were then marked to be sent, so a real address
  book gained a duplicate and lost one of her addresses.

  An imported card now adds addresses and takes none away. It still wins
  everywhere else it says something: a company, a job title, a photo or a phone
  number written on a card replaces what is stored, and everything the card is
  silent about is kept as it was. A card that names one address on two lines,
  which is what a program that merged two records writes, adds it once.

  What that costs: importing a card can no longer remove an address. Removing
  one is done in the contact editor, where it is one person deciding about one
  contact rather than a file deciding about everybody in it.

  Still true, and not fixed here: one file holding two cards for somebody this
  computer has never seen still makes two contacts. Nothing in those two cards
  says they are the same person except the name, and two people can share a
  name.

- **The first-run screen now reads out the answer it is going to use.** The
  screen that asks what Wixen Mail may change put focus on "Read my mail, change
  nothing" and ticked the next answer down, which sends changes to your tasks,
  contacts and calendar up to your provider. So the answer announced on opening
  was the cautious one and the answer Continue would act on was the next one
  down: pressing Enter on what had just been read out switched on writing to a
  real address book, calendar and tasks, and nothing said so.

  Focus now lands on the answer that is ticked, so what is announced is what
  Continue will do. Which answer the screen starts on has not changed: it is
  still the second of the three, which leaves mail alone and sends changes to
  tasks, contacts and the calendar to your provider. The words on the screen
  have been corrected where they described the old behaviour.

  Not verified: this has not been through a screen reader. What changed is which
  control the window puts focus on.

- **A contact you delete no longer comes back in the same sync.** Deleting a
  contact here sends the deletion out to the address books that hold her. The
  read that follows in the same sync could write her straight back down, under a
  new identifier, with nothing left to say she had ever been deleted. That reads
  as the delete having quietly failed, and it is worse than not syncing at all.

  The sync passed over anybody whose deletion had not gone out yet, and stopped
  passing over her at the exact moment it had: sending the deletion clears the
  note saying one is owed, so the check disarmed itself for the one person it
  was there to protect. It now passes over everybody the sync deleted, whether
  the address book has taken the deletion or not.

  Not verified: whether Google or Outlook really name somebody in a read taken
  moments after they took her deletion. Nothing here has run against a real
  account, and the rule holds either way, because nothing a sync deleted should
  be written back down by that same sync.

- **A deletion your address book never received is no longer read out as one it
  did.** Delete somebody on a phone, then delete the same person here before the
  sync catches up, and the address book answers that it has no such contact.
  That is the deletion having already happened, so the product stops asking,
  which is right. Counting it as a deletion that was sent is not: the summary
  said "1 deletion sent to your address book" about an address book that had
  just said it never had her, and that sentence is the only thing telling you
  whether your deletion travelled.

  A sync now says "1 already gone from your address book" for those, apart from
  the count of deletions that really went out. A sync where both happen says
  both.

- **An event made here reaches Outlook at the hour you set it.** The Outlook
  writer read a time this program had stored with no zone beside it as though
  it were already universal time. An event made here at nine in the morning
  went to Outlook as nine in Greenwich, so for anybody not on that clock it
  landed at the wrong hour: five in the morning on the American east coast,
  half past two in the afternoon in Kolkata. The Google writer has always read
  the same value as a time on this computer, so one event reached the two
  providers at two different hours.

  Both are now given the same moment, each still in the shape it reads: Outlook
  a clock face with a zone name and no offset, Google an offset. The test named
  for the two agreeing checked the shape each provider was sent and never
  compared the hours, so it passed throughout. It compares them now.

  Whole-day events are unchanged. Outlook is told a whole day starts at
  midnight and refuses one that does not, so its date stays where it is.

  Events already sent at the wrong hour stay there until you edit them again.

- **Correcting an event's spelling no longer moves it to a different time of
  day.** Changing anything about an event that came from Google, Outlook or a
  calendar server, even just its title, rebuilt its start and end from the two
  boxes in the event editor. Those boxes hold a date and a clock and have
  nowhere to put a time zone, so the zone the provider sent was dropped. The
  event then went back with its clock face read as a time on this computer: a
  nine o'clock meeting in Kolkata reached Google as nine o'clock here, nine and
  a half hours out, and nothing said so. Sent to a calendar server the same
  edit wrote a time in no zone at all, which every client reads in its own.

  A date or time box nobody typed in now keeps the moment exactly as the
  provider sent it, down to the seconds. A box somebody did type in is read in
  the zone the event is in rather than the zone this computer is in, so moving
  a meeting to half past ten means half past ten where the meeting is.

  This covers events carrying an offset such as `+05:30`, events written in
  universal time, events named in a zone such as `Europe/London`, whole-day
  events, repeating events, and events made here, which have no zone to keep
  and are still read as times on this computer.

  Which events were affected: those whose provider sent a time with an offset
  or a trailing `Z` and no zone name beside it, which is most timed events from
  Google. An event whose stored copy names its zone was never moved by this,
  because the name was kept and the name is what both providers read.

  Known limitation: an event carrying an offset and naming no zone, moved to a
  date on the other side of a daylight saving change, keeps the offset it had
  and lands an hour out. Nothing stored on such an event says what its zone's
  rules are. Where the event does name a zone, the name decides and this does
  not happen.

  Events already sent at the wrong time cannot be put right from here. The hour
  of day survived, so the event still reads as the time you set it, but the
  provider now holds that hour in this computer's zone and nothing here
  remembers which zone it should have been. Putting one right means opening it
  in Google Calendar, Outlook or your calendar server and setting its zone
  there.

- **The changelog said this program changes nothing at your provider until
  you say so.** A new installation allows changes to tasks, contacts and the
  calendar at Google or Microsoft. It does not allow mail to be sent, moved
  or deleted. Two entries further down claimed the opposite, and eight
  comments in the source said the same thing in their own words, so somebody
  reading either could have pointed this at a real address book believing
  Allow Changes was holding everything on this computer. What the program
  does has not changed. What changed is that it no longer says it is safer
  than it is.

- **One person is one contact, whichever of their addresses is written down.**
  A contact card, and an address book's own copy of somebody, were matched to
  the person already here by their main address alone and letter for letter.
  Two ways that made a second row for one person:

  - The card, or the address book, wrote to one of her other addresses.
    Somebody with a personal address here and a work address as well came back
    as two people, and only one of the two was still joined to Google and
    Outlook. The other was then offered to your address book as a new person.
  - The address was spelled with capitals in it. `Alice@Example.com` and
    `alice@example.com` were two people.

  Any address a contact holds is now that contact's, on both sides of the
  question, and addresses are compared without regard to case. The domain half
  of an address means the same however it is written, by definition, and no
  mail system anybody uses treats the half in front of the @ as case sensitive
  either. Two people who share no address are still two people.

- **Adding the same mailbox twice is refused however its address is typed.**
  Adding an account already set up was refused, but only when the address was
  typed exactly as before. "Test@Example.com" alongside "test@example.com"
  made a second account for one mailbox, and everything is filed per account,
  so the second one had its own mail, its own contacts and its own calendar
  and neither showed what the other held.

- **Searching your contacts finds somebody by any address or number she has.**
  The search box above the contact list looked at the first address and the
  first number only. The address you have for somebody is as likely to be her
  work one as her personal one, so typing the address you have found nobody
  and the list read out as empty. It now looks at every address and every
  number on the contact, still without regard to case.

- **Import Contacts and Export Contacts work on the account you are looking
  at.** Both named a fixed word rather than the account. Everything else in
  contacts works on the account you are looking at: the list is drawn from it,
  a sync runs for it, and a contact belongs to it. So with any account signed
  in, a file of cards was read into a part of the database nothing reads. The
  count said the contacts had arrived, the list did not change, and no sync
  ever saw them. Export had the matching half and wrote out a file with
  nothing in it whatever the list was showing.

  With no account chosen, both now say "Choose an account first. Contacts
  belong to the account whose address book holds them" instead of appearing to
  work. The contacts list is also filled again as soon as an import finishes,
  so what arrived is on the screen rather than only in the announcement.

- **The calendar sync now says when a change can never be sent.** A change could
  be left waiting with nothing anywhere able to send it, and nothing told you.
  The row waited for ever and every sync looked straight past it. Three ways in:
  the event is in a calendar you can only read, it is in a calendar made on this
  computer that no account holds, or it is in no calendar at all, which is where
  every event made from the calendar window starts.

  Each of those now gets a sentence in the sync summary, one for the calendar
  rather than one for each change, and it says the words are kept here and
  nothing is written over them, so nothing is lost, and what to do to have the
  change saved. It is said on every sync, because nothing resolves it on its
  own and saying it once means the person who was away that time never hears it.

  A calendar feed already had a sentence like this and it only counted events the
  feed itself names, so an event you moved into the feed's calendar, or made in
  it, was the one change most likely to be waiting and the one never mentioned.
  It is counted by asking the calendar what is waiting in it now.

- **A calendar you can only read is no longer offered as somewhere to move an
  event.** Moving an event offered every calendar on the account, including a
  calendar feed you subscribed to and one your calendar server lets you read but
  not write. Choosing one filed the event there, marked it as waiting to be sent
  and said "Dentist moved to Term dates". Nothing was ever sent: every sync from
  then on found a calendar it can only read and left the change where it was.

  Those calendars are now left out of the list you choose from, and a move into
  one is refused where the move is written as well, so it cannot happen by any
  route. The refusal has the same shape as the one you already get for an item
  your provider holds: "Term dates is a calendar this program can only read, and
  an event moved into it could never be sent. Nothing has been moved. A calendar
  you can change can hold it."

- **Saving an event you did not change no longer sends the whole event back to
  your provider.** Opening an event, pressing Save and typing nothing marked the
  event as changed. The next sync wrote the whole record back to Google or
  Outlook, including everything on the event this program keeps no copy of,
  which it overwrote with what it does not know.

  An event nobody changed is now left exactly as it is, and nothing is sent. An
  event you did change is sent as before, and a change you made and have not
  sent yet still goes, so opening it and pressing Save cannot lose it.

  One case still counts as a change when nothing was typed: a repeating event
  opened on one of its later days. The editor is filled with the day you were
  standing on and the stored event starts from another day, so the two differ.
  That is what happened for every event before this change, so nothing is worse
  than it was.

- **A calendar or task list you made here is no longer promised back from your
  provider.** The question asked before deleting one said "It will come back at
  the next sync" whenever the open account was a Google or Outlook one. That is
  right for a calendar or list your provider sent, and wrong for one you made
  here and filed under that account: nothing sends it anywhere, so nothing puts
  it back, and you were left waiting for a sync that will never mention it.

  The question now asks about the one you are deleting rather than about its
  kind. A calendar says where it came from, and a task list says it in the
  identifier it was stored under.

  Nothing else changed about deleting a container. Deleting a calendar, a task
  list, a note folder or a contact group still removes only the copy on this
  computer, and the question still says so for the two kinds a provider holds.
  Note folders and contact groups are never sent anywhere, so nothing is
  promised back for them and nothing is claimed about a provider.

- **A contact you delete is now deleted in your address book too.** Deleting a
  contact took it off this computer and told nobody. The next sync read your
  address book, found the person still there and wrote them down again, so a
  contact you had deleted came back after the product had said "deleted". A new
  installation already allows changes to contacts, so this happened to anybody
  who deleted one.

  Deleting a contact now leaves a note for each address book that knew them, and
  the next sync sends the deletion to each of those address books under the name
  that address book uses for the person. A note is cleared only once that
  address book has taken the deletion, so a deletion your provider refuses, or
  one that Allow Changes is holding back, waits and goes later instead of being
  lost. Until it goes, the sync no longer puts the person back.

  Somebody in two address books is one person. The deletion goes to both, and
  the sync counts them once: "Contacts sync: 0 created, 0 updated, 0 deleted, 1
  deletion sent to your address book". That clause is new, and the count behind it is
  new: there was one with the same name before, nothing could ever set it, and
  it was taken out rather than left able to report deletions that had not
  happened.

  A contact who only ever lived here leaves no note, because there is nowhere to
  send one. An address book answering that it has no such contact, which is what
  it says about somebody you had already deleted from a phone, counts as the
  deletion having happened. Where the address book is the one saying the person
  is gone, nothing is sent back to it.

  Known limitation: none of this has run against a real account. Deleting a
  calendar, a task list, a note folder or a contact group is still local only,
  and the question asked before it says so.

- **Somebody one address book deletes and the other still has stays here, and
  is counted once.** Google saying it had deleted somebody took the whole
  contact off this computer, and that included Outlook's name for her and any
  change Outlook had not been sent yet. The read from Outlook later in the same
  sync then wrote her down again as a new person, so one contact was read out as
  "1 created, 0 updated, 1 deleted" with one row left over. On the ordinary sync
  where Outlook has nothing new to say she was not written down again at all, so
  somebody Outlook still holds disappeared because Google had let her go.

  An address book saying it has deleted somebody now speaks for itself and for
  nothing else. It comes off the contact, and the rest stays: the row, its photo
  and the card it was imported from, the other address book's name for her, and
  the change that address book is still owed. The same sync says "0 created, 1
  updated, 0 deleted", which is one person and one thing that happened to her.

  Where no other address book holds the person, the row still goes, and the sync
  still says so when work went with it.

  A change that was waiting only for the address book that let her go stops
  waiting, because there is nobody left to send it to.

  Known limitation: where a contact came from is not rewritten, so somebody who
  came from Google and is now only in Outlook still counts as a Google contact.
  While "Send a change to a contact to every address book that has that
  contact" is off, a change to her goes nowhere, and the sync says so on every
  run: "1 change is not going to your other address book."

- **A change your address book has moved past is sent again instead of being
  thrown away.** Google and Outlook both hand out a version marker with their
  copy of a contact, and both turn down a change built against an older one.
  That is the ordinary case on a real account rather than a rare one: a phone or
  a webmail tab moves the other copy between one sync and the next.

  What happened before. A new installation already allows changes to contacts,
  so the sync sent the change carrying the marker from before, the address book
  turned it down, and the read that followed in the same sync wrote its copy
  over your edit. Nothing was sent and the edit was gone.

  Where somebody had turned Allow Changes off and then on again, the same fault
  read worse. The first sync said "1 change is waiting here: turn on Allow
  Changes in Settings to send it". Turning it on and syncing again lost the
  edit to both address books at once: "0 created, 1 updated, 0 deleted, 1 of
  your change replaced by the address book, 2 errors". The instruction had been
  about work the sync that followed it destroyed.

  A change turned down for that reason is not a change that failed. Wixen Mail
  now asks the address book what it holds for that one contact, puts the change
  on the marker that comes back, and sends it again, so your edit reaches the
  address book on the sync you asked for.

  What that costs, said plainly. The address book had changed that contact as
  well, and what it changed in the fields shown here is written over. The sync
  says so: "A contact you had changed was changed in your address book as well,
  and what you have here was sent over it." The copy that lost is still on the
  phone or the web page that made it, and the edit made here is nowhere else,
  which is why that is the one that wins. Fields the address book holds and
  Wixen Mail never sends are left alone.

  Known limitations: none of this has run against a real Google or Outlook
  account. The two answers now read as "your copy is out of date", 400 with
  FAILED_PRECONDITION from Google and 412 from Outlook, come from each
  provider's documentation rather than from a live sync, and a refusal worded
  some other way still ends the old way, with the address book's copy winning
  and the sync saying your change was replaced. A change that failed to go
  because the network dropped ends the old way too. If the address book will not
  hand its own copy over, nothing is sent, the sync says so in one error line,
  and your edit stays here to be sent next time rather than being replaced.

- **Importing a contact card no longer takes that person off your address
  books.** Import Contacts reads a .vcf file, or a folder of them, and folds
  each card into the contact it matches by email address. For somebody who is
  also in Google or Outlook, the record it wrote named no address book at all,
  so the contact became one this computer alone knows: every later edit to that
  person stopped reaching Google and Outlook, and a change already waiting to be
  sent was forgotten. The import also wrote over a saved photo, a note, a
  company and a job title with nothing whenever the card carried none of them.

  The rule now is the one the calendar sync already follows. A card is something
  somebody wrote down on purpose, so it wins wherever it says something, and
  every field it is silent about keeps what is stored. A card with no photo on
  it is not a card asking for the photo to be taken away. Which address books
  hold the person, whether a change is still waiting to be sent, when that
  address book was last read, and where the contact came from are none of them
  on a card, so an import no longer answers any of them.

  Where a card does list things, its list is what gets stored. A card listing
  one email address for somebody who had two here leaves them with the one on
  the card, because the card is saying which addresses that person has, and
  that is what makes re-importing a corrected card able to correct anything.

  What an import changes is now sent to the address books that hold the
  person. That is new in this release and is described under "What an
  imported card says now reaches your address books".

- **Editing a contact no longer loses a postal address written by an older
  version.** This one is about old data rather than anything that can be created
  today. A contact stored before Wixen Mail kept lists of addresses has its
  postal address in a single field, and the contact editor showed the email
  address and the phone number from those older single fields but not the postal
  address. So the editor opened with no address in it, and saving anything at
  all about that contact, a corrected phone number included, wrote the address
  away as empty. The editor now shows it the way it shows the older email
  address and phone number, on one line and labelled Home, and saving keeps it.

- **Editing an event keeps every alert on it, not just the first.** The event
  editor has one alert box, and it is filled from the first alert on the event.
  Saving rebuilt the alerts from that box alone, so an event with a popup
  fifteen minutes before and an email the day before came back with the popup
  only. The row was also marked as waiting to be sent, so the missing alert went
  up to Google or Outlook on the next sync. The alerts now come from the event
  being replaced. The box sets the lead time of the first alert and leaves the
  rest alone, including how that alert reaches you, which the editor has no
  control for. Clearing the box to zero takes off the alert it was showing and
  keeps the others.

- **Moving a task or an event no longer says "moved" when nothing was sent.**
  Move to another list or calendar wrote the new list or calendar here and did
  not mark the item as waiting to be sent, so nothing ever told Google or
  Outlook and the status line still said it had moved. An item made on this
  computer is now queued, and the next sync creates it in the list or calendar
  you chose. An item that came from your account is refused with the reason
  instead, and the refusal comes before the chooser opens rather than after you
  have picked somewhere. Nothing is written for a refused move.

  Known limitation: no move is sent to Google or Outlook as a move. Moving an
  item they hold means deleting it where it is and creating it again where it is
  going, and that is not built, which is why those are refused. A move of an
  item made on this computer reaches them only as that item being created in the
  list or calendar it ended up in.

- **Nothing offers a setting per account any more, because there is not one.**
  The first-run screen and the testing page both said you could set what Wixen
  Mail is allowed to change per account, and the line after a sync told you to
  "turn on Allow Changes for this account". No screen writes an answer for one
  account. There is one answer for the whole application, the settings screen
  writes that one, and it covers every account you have signed in.

  The shape the testing page recommended was not one the program could take
  either. It said to leave your real mail read only and allow everything on an
  account you do not mind breaking. An answer for one account can only ever take
  permissions away from the application-wide one, never add them, so that is the
  wrong way round.

  A sync now says "1 change is waiting here: turn on Allow Changes in Settings to
  send it", which is where the control really is. To use a real account with
  nothing at risk, start Wixen Mail with `--read-only`, which the testing page
  now points at instead.

  Known limitation: the setting an account can carry is still read and still
  honoured, so a per-account answer would work if anything wrote one. Nothing
  does, and no screen offers it.

- **The status line says "1 message" rather than "1 messages".** The line at the
  bottom of the window, which is also read out on every switch into a module,
  put the number in front of a plural word whatever the number was. A mailbox
  holding one message said "1 messages, 0 unread", an account with one folder
  said "1 folders on the server", and the same went for folders, calendars,
  calendar events, reminders, task lists, tasks, notes and contacts as each list
  arrived. This is heard far more often than any of the sync lines that had the
  same fault, because it is said on every switch between modules.

  They all ask the routine the sync summaries already ask, so there is one place
  to get this right rather than one per module. They are worked out where they
  can be checked rather than inside the window code, which is why nothing could
  reach them to check them before.

  Known limitation: none of these has been heard with a screen reader.

- **A contacts sync no longer counts the same person twice.** The line after a
  sync says how many contacts were created, changed and deleted. A person kept
  in both Google and Outlook arrives from whichever address book is read first
  and has the other one's copy folded in straight after, and both were counted.
  So a first sync of three people both address books hold said "Contacts sync: 3
  created, 3 updated, 0 deleted" and stored three contacts. At the two hundred
  shared contacts somebody really keeps, that reads as four hundred.

  The same fault counted one person as two when one address book moved its copy
  while the other deleted the contact: "1 updated, 1 deleted", about somebody who
  is not there any more.

  Those three counts are one person each now. Which of the three somebody is, is
  what happened to them by the end of the sync: gone if they were removed, new if
  they were not here when it started, changed otherwise. The same first sync says
  "Contacts sync: 3 created, 0 updated, 0 deleted".

  What is said after those three counts is deliberately about the same people,
  because each part says something the three cannot: what went out rather than
  what came down, whose work was lost, what is waiting on which setting, and
  whether a deletion took work with it. So the person counted as changed can be
  the same one whose edit the address book replaced, and both are said.

- **A contact change a setting holds back is still there when you turn the
  setting on.** A contacts sync sends your changes first and then reads the
  address book. Where a setting stopped the sending, the read that followed in
  the same sync wrote the address book's copy over your change and stopped it
  waiting. So the line after the sync told you to turn a setting on to send a
  change that the same sync had just thrown away, and turning it on sent
  nothing.

  Both settings did it. With Allow Changes off, one edit to one contact both
  address books hold was read out as "Contacts sync: 0 created,
  1 updated, 0 deleted, 1 of your change replaced by the address book. 1 change
  is waiting here: turn on Allow Changes in Settings to send it." Both
  halves of that were about the same contact, and by the end of the sync the
  change was gone. With "Send a change to a contact to every address book that
  has that contact" off as well, a second sentence named that setting and it did
  nothing either.

  The same sync now says "Contacts sync: 0 created, 0 updated, 0 deleted. 1
  change is waiting here: turn on Allow Changes in Settings to send it."
  Your change is still on the contact, and turning the setting on really does
  send it.

  What that costs, said plainly: while the setting is off, that one contact stops
  taking updates from the address book, because taking them would be writing over
  your change. The sync says so on every run for as long as it lasts. And your
  change is held against the version marker it was made against, so if the
  address book has moved its own copy on in the meantime, it can turn the change
  down once you do allow it. That is the ordinary tie, the address book wins it,
  and the line after the sync says your change was replaced.

- **A setting stops being blamed for holding back a change that has already
  gone.** When "Send a change to a contact to every address book that has that
  contact" is off and your change had already been replaced by the address book
  it came from, what was left waiting for the other address book was that address
  book's own copy rather than anything you wrote. Every later sync went on saying
  "1 change is not going to your other address book", for ever. It is now said
  while what is waiting is your work, which is the same rule the "replaced by the
  address book" line already followed.

- **Editing one contact no longer rewrites every contact in the account.** The
  contact manager hands back its whole list whenever anything in it changes, and
  every row in that list was written back as though you had edited all of them.

  The part that happens today, whatever your settings say: each rewritten
  contact was rebuilt out of what the contact editor holds, and the editor holds
  no photo and no imported card. So correcting one phone number erased the saved
  photo and the original card of every other contact in the account, and left
  each of them looking like a contact typed on this computer rather than one
  taken from an address book.

  The part that waits on a setting: every contact in the account was also marked
  as having a change waiting for each address book that knows it. A new
  installation allows changes to contacts, so the next sync pushed your whole
  Google and Outlook address book back to those providers, with the photos
  already gone. With Allow Changes turned off, nothing was sent.

  A contact you did not change is now left exactly as it is, which means nothing
  is written for it at all. A contact you did change is still marked as waiting
  and still owed to every address book that has it, and it now keeps its own
  photo and card as well.

- **A message rule you add is saved now.** Adding a rule in the rule manager
  did nothing at all. The manager tried to change the rule first and only
  created it if that failed, and changing a rule that is not there yet is not a
  failure, so nothing was ever created. Editing a rule that already existed
  always worked. The same mistake was found and fixed on labels and on
  signatures before this, and the rule manager was left behind both times.

- **"1 errors" and "1 changes are waiting here" are said properly now.** The
  line read out after a sync put the number in front of a plural word whatever
  the number was, so one thing going wrong was "1 errors", one change held back
  by a setting was "1 changes are waiting here: turn on Allow Changes for this
  account to send them", and a folder holding a single message was "1 of 1
  messages downloaded". The calendar sync, the contacts sync, the tasks sync
  and the mail sync each had their own version of it.

  They ask one routine now, so there is one place to get this right rather than
  five. The sentence about changes waiting is also written once and shared by
  the calendar and contacts syncs, which is how one of the two copies came to
  be corrected on its own.

- **A hole closed in how a sync summary is punctuated, which nobody meets
  today.** The rule is that each clause says what it is and never how it joins
  to its neighbours, so the summary decides the spacing and the full stops in
  one place. It took the stop and the spaces off the end of a clause and never
  off the front, so a clause written with a space already there would have
  carried two: "0 deleted.  Term dates: something". On the status line that is
  a typo, and what a speech synthesiser makes of it is up to the synthesiser.

  Nothing in the program writes a clause that way. Every path that could was
  checked and each one either trims what it writes or cannot reach that clause
  at all, so this is a guarantee that was not kept rather than a fault anybody
  has heard. It is fixed because the next clause somebody writes should not
  have to know.

- **A change an address book replaced is now said once, not on every sync after
  it.** Where your copy and the address book's copy have both moved, the address
  book wins and the line after the sync says your change was replaced, so a lost
  edit is not silently gone. While your other address book was still owed that
  contact, every later sync where the first one moved again said it over again,
  for ever, about an edit that was lost once.

  What was waiting after the first time was not your edit any more, it was the
  copy that survived, on its way to the other address book. A sync can tell
  those apart from what it already keeps, so the second one is now an ordinary
  update and says nothing. Nothing new is stored to do it.

- **A change kept from your other address book by a setting now says so.** When
  "Send a change to a contact to every address book that has that contact" is
  off, a change goes only to the address book the contact came from. That is
  what the setting is for, but the change kept from the other address book was
  counted nowhere and said nowhere: the sync reported a clean run, the change
  stayed waiting on the contact for ever, and nobody could tell that from an
  edit that had gone everywhere.

  The line after a sync now says "1 change is not going to your other address
  book: turn on sending a change to every address book that has the contact." It
  names that setting and not Allow Changes, because turning Allow Changes on
  sends none of these.

- **One contact you changed once is now said once, not once per address book.**
  The line after a contacts sync counts people, and a person kept in both Google
  and Outlook is one person. Until now each address book counted its own copy of
  her, so every number about that one person was double: one edit to one contact
  was read out as "2 updated" and as "2 of your changes replaced by the address
  book", and the same edit was described twice as lost.

  One edit to a contact both address books hold, where both of them turn the
  change down and then replace it with their own copy, now says "Contacts sync:
  0 created, 1 updated, 0 deleted, 1 of your change replaced by the address
  book, 2 errors". The errors really are two. Every other number counts people;
  these count what went wrong, and two address books refusing the same change is
  two things to look at.

  Two smaller faults in the same sentence go with it. A contact one address book
  changed while the other left its copy alone was counted as updated and as
  unchanged, so one person was read out as two. And one contact waiting on a
  setting was read out as "1 changes are waiting here ... to send them".

- **A contacts sync no longer reports contacts as changed when nothing changed,
  and no longer counts deletions it never made.** Two of the numbers on the line
  after a contacts sync were saying more than had happened.

  The first was the count of contacts updated. A sync reads only what changed
  since the last one, but there are two ordinary times it reads the whole
  address book instead: the first sync of an account, and any sync where the
  address book says the marker from last time is too old. Every contact that
  came back on such a read was written down again and counted as updated, so a
  first sync of two hundred contacts said "200 updated", and so did a re-read
  weeks later when nothing at all had changed. Hearing that your address book
  changed overnight is the kind of thing somebody goes looking through their
  contacts over.

  A contact where neither copy has moved since the last sync is now left alone
  and counted apart, the way the task sync already counted one: "Contacts sync:
  0 created, 0 updated, 0 deleted, 200 unchanged". The count is only said when
  there is something to say.

  The second was the count of contacts deleted, which added together two
  numbers: contacts removed from this computer because the address book no
  longer had them, and contacts deleted at an address book. The second number
  could never be anything but nought, because nothing here deletes a contact at
  an address book, and yet it was added into the total that gets read out. It is
  gone, so the number said is the number of contacts really removed.

  Known limitation, and it is the reason that second count existed: **deleting a
  contact here does not delete it in Google or Outlook.** It goes from this
  computer and the address book keeps its copy, so the next sync brings it back.
  Nothing says so at the time; the message after a delete is the contact's name
  and the word deleted. Making a delete travel means keeping a note of what was
  deleted while offline, which is a change to how contacts are stored rather
  than a line of code, so it is not done here. If you want a contact gone from
  your address book, delete it there.

- **A contact file laid out with indentation is read again, and an import that
  added nothing now says why.** The card standard reads every line starting with
  a space or a tab as the rest of the line before it, and this program does the
  same. A file somebody laid out by hand, or ran through a formatter, uses those
  same spaces to show what sits inside what. Read as joins, the whole card ran
  together into a single line with no email address anywhere on it, the contact
  was dropped, and the import said "Imported 0 contacts". A file of two hundred
  contacts written that way imported as nothing at all.

  A card is now read as laid out by hand wherever the two can be told apart,
  which is a line with a space in front of it directly after the line that opens
  or closes the card. Those lines carry the word `VCARD`: it is short, it holds
  no spaces, and no program breaks one across two lines. The spaces come off
  every line in such a card instead of being read as joins. Where the two cannot
  be told apart, a line starting with a space is still the rest of the line
  before it, because that is what the standard asks for. The calendar reader
  draws the line in the same place, for the same reason.

  What it costs, which is worth saying plainly: a card that is both laid out by
  hand and has a long line broken across two now reads that one line short. Once
  the layout is off there is nothing left to tell a join from an indent. The
  contact itself still reads, where before the card had nothing in it at all.

  The silence was the worse half. A card this program cannot use is passed over,
  which is right, because one bad card must not cost you the other hundred and
  ninety-nine. What came back was a count of how many arrived and nothing else,
  so a folder with no card files in it, a folder whose cards were all turned
  away, and a file this program could not read all said "Imported 0 contacts".

  An import now says what it left out as well as what it took. "No contacts were
  imported. 3 cards named no email address and were left out, because a contact
  here needs an address to write to." A contact that was read and could not be
  saved on this computer is said too, instead of going only to the log. The
  counts are added up across every file in a folder and said once.

  Known limitations: none of this has been heard with a screen reader. A card
  that names no email address is still turned away, which is the same decision
  as before and is why an address book kept without email addresses imports as
  nothing; what has changed is that it now says so. A file this program cannot
  open at all, because of its permissions or because it is not text, is still
  skipped without a word.

- **The sentence said after a sync no longer stutters or trails off.** When a
  sync had more than one thing to tell you, the parts were pushed on to the end
  of each other and their punctuation collided. A contacts sync with changes
  waiting, a contact deleted with your change still in it, and something that
  failed, all at once, said "turn on Allow Changes for this account to send
  them.. 2 contacts you had changed were deleted in your address book, and your
  changes went with them., 2 errors". On screen that is a typo. Read aloud it is
  not: a screen reader stops at every full stop, so you heard a stutter, then
  the sentence, then a fragment hanging off the end of it.

  The parts of the sentence are now collected and punctuated in one place. The
  counts go in the opening list, each whole sentence follows with one stop at
  each end, and a count worked out last no longer lands behind a full stop. The
  same fault was in the calendar sync, which said "to send them., 1 errors" and
  ran two sentences together at "them.. Term dates", and in the task sync, where
  the count of problems was hung on the end of "Sign in to this account again to
  send task changes" with a comma, so it was heard as part of the instruction.
  All three now read as sentences. The line after a mail check is a list of
  counts with no sentence in it, so it never had this; it is now built the same
  way, and is worked out where it can be checked rather than inside the window,
  so the first sentence somebody adds to it cannot start the fault off again.

  Known limitation: none of these sentences has been heard with a screen reader.

- **A contact you changed is no longer quietly replaced by your address book's
  older copy.** Change a contact here, and the next contacts sync wrote Google's
  or Outlook's copy of that contact straight over your change: the name, the
  email addresses, the phone numbers, the company, the job title, the notes and
  every other field either address book holds. Your edit went, and the status
  line called it an ordinary update. With Allow Changes turned off, this
  happened to every contact you edited, every time, because the change could
  never be sent and so was never anything but the newer copy.

  A sync now works out whether your address book has actually touched its own
  copy since it was last read, using the version marker the address book gives
  out with a contact.

  - If your address book has not touched it, your change stays and is sent when
    it can be.
  - If both of you changed the contact, the address book's copy wins, and the
    status line says so: "1 of your change replaced by the address book". The
    address book wins because its copy is the one your phone and the web page
    already agree on, and because it refuses a change that does not carry its
    current version marker, so a copy kept here would be stuck being refused for
    ever with nothing to break the deadlock.

  Two related losses are fixed with it. A contact you typed here that your
  address book turned out to already hold, matched by its email address, was
  replaced the same way and left marked as having something to send, with
  nothing left to send it; it is now replaced openly, counted, and no longer
  left waiting for ever. And a contact deleted in your address book while you
  had an unsent change on it now says what happened rather than counting as one
  more deletion: "A contact you had changed was deleted in your address book,
  and your change went with it."

  Known limitations: none of this has run against a real Google or Outlook
  account. Whether a screen reader announces the new sentences well has not been
  checked with a screen reader. Where an address book gives no version marker at
  all, a sync cannot tell whether it changed its copy, so it treats it as
  changed and your edit loses; it is counted and said rather than silent, and
  both address books this program talks to do give markers. Losing a change is
  still losing a change: nothing keeps a copy of what was replaced, so the only
  way back is to make the change again.

- **A change that reached one of your address books is no longer thrown away by
  the other.** A person can be in both Google and Outlook, and a change you make
  reaches both. Between one sync and the next it can have reached one of them
  and still be owed to the other, which is where you are left when one of the
  two pushes fails, and where you always are if you have set changes to go only
  to the address book a contact came from.

  In that state, syncing the address book that already had your change wrote its
  copy of the contact over everything stored here. If that address book had
  changed the contact since, the edit the other one was still owed went with it,
  nothing counted it, and the status line said "1 updated".

  A sync now asks whether the contact stored here holds work that has not
  reached everywhere it belongs, rather than asking one address book about its
  own copy. Your change stays until every address book that knows the contact
  has it. If the address book you are syncing has moved its own copy in the
  meantime, its copy still wins, but now the loss is counted and said the same
  way as any other: "1 of your change replaced by the address book".

  The same mistake made deletions silent. Deleting a contact here removes it and
  everything waiting to be sent with it, whichever address book was still owed
  it, so a contact deleted in Google while Outlook was owed your change went
  without a word. It is now counted and said.

  Known limitations: none of this has run against a real account. What survives
  is still the address book's copy, not yours; nothing keeps what was replaced,
  so the only way back is to make the change again. When the address book's copy
  wins, that copy is then sent on to the other address book, which is what keeps
  the two agreeing, and the status line counts it as one of your changes sent.
  It is not one of yours, and the count says more than it should.

- **A change sent to Outlook now says which copy it was built on.** Google has
  always been told the version of the contact a change was made against, and
  refuses the change if its own copy has moved on since. Outlook was told
  nothing, so two devices editing the same Outlook contact overwrote each other
  and neither was told. This was also given as the reason the address book wins
  a tie, in the changelog and in the code, and for Outlook it was not true.

  Changes to Outlook contacts now carry the version marker Outlook last gave, as
  an `If-Match` header, which is where Outlook looks for it. A change built on a
  copy that has moved on is refused rather than overwriting what is there, the
  sync says the change could not be sent, and the read that follows brings back
  the copy that is there and counts it as replacing yours.

  Known limitations: this has not run against a real Outlook account, so what
  Outlook actually answers to a marker it does not recognise has not been seen.
  A contact stored before this program kept markers has none to send, and a
  change to it still goes out unconditionally and can overwrite a change made
  somewhere else. Calendar events are unchanged and send no marker.

- **A contact exported to a file and imported back is now the contact it was.**
  Exporting your contacts to a `.vcf` file and importing that file, which is how
  you move an address book to another machine or keep a copy of one, used to
  lose all of the following. They are fixed:

  - The given name and the family name were written nowhere at all, so a
    contact came back with a whole name and neither part. The card now carries
    both, in the field the format gives them. Nothing is split out of the whole
    name to fill them in, so "van der Berg" stays "van der Berg".
  - A department was lost. With a company recorded it was written nowhere; with
    none it went out in a way that came back as a company called ";Research".
    Company and department now travel together, the way the format intends, and
    come back in their own boxes.
  - How somebody is related to you was written out and read by nothing.
  - A field you named yourself was written with its name forced into capitals
    and its spaces turned into dashes, and read by nothing. "Blood type" now
    comes back as "Blood type".
  - An address with a semicolon in it, such as "12 High Street; Flat 2", came
    back with the flat number in the town box and shoved the town, the county
    and the postcode each one box along, dropping the country off the end.
  - The address shown on a contact's main line came back as the raw line from
    the file, punctuation and all, which a screen reader reads out as a run of
    semicolons. It now reads the same after an import as it does after an edit.
  - The label on a phone number, an email address or a postal address was cut
    short, and one kind of label damaged the value beside it. A label is
    whatever you picked in the contact editor or whatever your address book
    called its own custom type, so "Work, main", "Home; the flat" and
    "Grandma's house" are ordinary ones. A comma cut the label off at the
    comma. A semicolon threw away everything after it. A colon did that and put
    the rest of the label on the front of the phone number, so the number came
    back wrong as well as the label: "Ada: personal" turned "+44 7700 900999"
    into "PERSONAL:+44 7700 900999". Even "Work Fax", which is in the dropdown,
    came back "Work fax". They all come back now exactly as they were, and so
    does a label run together with a comma and no space after it, such as
    "Work,main", which was the last shape a comma still cut off.
  - A space next to a line break in the file was eaten. The format breaks a
    long line in two and marks the second half with a space, and reading it
    back took off every space there rather than the one the break put in. A
    county of "Tyne and Wear" came back "Tyneand Wear" whenever the break
    landed beside that space. This was also an import fault: a card written by
    any other program that broke a line in the same place arrived with two
    words run together.

  What still does not survive a round trip:

  - A contact with no email address, such as one you keep only a phone number
    for, is written to the file and then refused on the way back in. The import
    turns away any card that names no address it could write to, so that a file
    from anywhere cannot fill your address book with rows nobody can reach, and
    that rule is why this contact is turned away too. Which of the two rules
    should give way is a decision nobody has made yet. If you keep contacts
    with no email address, a `.vcf` file is not a complete backup of your
    address book. The import no longer does this without a word: it says how
    many cards it left out and why.
  - Whether a contact is a favourite. A contact card has no property for it and
    this program does not invent one, because a property only this program
    understands would help it talk to itself and nothing else.
  - A label made only of the words the card standards define, run together with
    a comma and no space, such as "Work,Home". It comes back as "Work". Written
    that way, a card cannot tell it from a list of two standard labels, which
    is what `TYPE="voice,home"` means in a card written elsewhere, and reading
    a real list correctly is worth more. Put a space after the comma, or use
    any word the standards do not define, and the label survives.
  - Custom fields written by a build from before this release. Their names were
    already flattened into capitals in the file, and the new reader does not
    read them. Everything else in such a file reads normally.

  Also worth knowing, and neither of these is a loss. Exported cards now carry
  the structured name property that the vCard 3.0 standard requires and this
  program never wrote, so a card exported from here is valid where it used to be
  malformed. And a label with punctuation or a space in it is now written in
  quotes, which is what the standard asks for, so other programs read it as one
  label rather than as the end of the line.

  None of the export or import work has been checked with a screen reader, and
  none of it has run against a real account, which it does not need to: a `.vcf`
  file is read and written here and nowhere else.

- **An appointment whose title a calendar server wrote with a stray space no
  longer reaches the calendar twice.** The calendar standard puts no space
  between a property name and the punctuation after it, so a server holding
  `SUMMARY :Quarterly review` is holding a line it should never have written.
  This program used to read past such a line and then leave it in place when it
  sent your change, so the server ended up with two titles on one appointment
  and which one you saw was up to whichever calendar program you looked in. The
  same went for the start date, where two of them is an appointment on two days.

  Both halves of the program now read such a line the same way. It is read
  rather than refused, so the title shows here instead of the appointment coming
  through with no title at all, and it is replaced rather than left beside your
  change when the change goes out. The zone a start date is written in is read
  the same way too, so a nine o'clock London meeting no longer arrives with no
  zone on it.

  Known limitation: this covers space around the name and nothing else. A line
  mangled some other way is still copied through untouched, and this program has
  no way to notice, because the check that reads a change back out asks the same
  reader that missed the line.

- **An appointment further ahead than a sync asks about is no longer deleted
  from this computer.** When this program reads a calendar from a calendar
  server it asks for six months back to a year forward, because asking for
  everything is slow on a calendar with years of history in it. The pass that
  followed then removed anything the server had not just mentioned, and an
  appointment eighteen months out is not mentioned for the simple reason that
  nobody asked about it.

  So an appointment you made a long way ahead was sent to the server on one
  sync and deleted from this computer on the next, with nothing said. The server
  still had it, so it came back the next time you looked further ahead than a
  year, or when the window caught up with it. Until then it was gone from every
  list, every search and every reminder here.

  A sync now only removes an appointment when it falls inside the stretch of
  time it actually asked about. An appointment it cannot place, because the date
  stored for it cannot be read, is kept rather than guessed at.

  Known limitation: an appointment that repeats and whose first occurrence is
  older than six months is now kept even when the server really has dropped it,
  because what is stored here is where the series starts and not the days the
  server worked out. A stale entry on a calendar is the safer of the two
  mistakes, and it clears the next time the series changes.

- **A change waiting to be sent to Google Calendar or Outlook Calendar is no
  longer written over by the read that follows.** A sync sends what you changed
  and then reads the calendar back. If the sending part could not go, and the
  usual reason is that Allow Changes is off, then the reading part wrote the
  provider's copy straight over your edit and marked the event as no longer
  waiting for anything. Your words were gone and nothing was left to try again.

  The calendar-server side of the program has always left a waiting change
  alone. Google and Outlook now do the same: while a change is waiting, the
  event here is yours, and the provider's copy is taken only once the change has
  been sent or you have discarded it.

- **An edit to an event in a calendar you subscribe to is no longer wiped by the
  next refresh.** Some calendars can only be read. A subscribed feed is
  published by somebody else, and a calendar server can mark one of its own
  calendars read-only. A change you make to an event in either can never be
  sent, and nothing in this program ever tried to.

  For a subscribed feed the refresh then wrote the feed's copy straight over
  your change. You edited an appointment, it looked saved, and at the next
  refresh it said what the feed said. Nothing told you, and the words you typed
  were gone. That is the same shape as the bug below about editing an event
  written in small letters, and the same reason it is the worst shape a bug can
  take here: only you knew the words had ever existed.

  Your change is now kept, the feed's copy is not written over it, and the pass
  that clears out events the feed has stopped carrying leaves it alone too. The
  event as it arrives from the feed is left alone until the change is gone. That
  matches how a calendar you can write to has always behaved, where a change
  waiting to be sent is the newer copy.

  That last part was missing when this entry was first written, and without it
  the rest did not hold. The row was safe from being written over and not from
  being deleted, so a feed that stopped carrying the event you had edited took
  your words away with it and said nothing. On a calendar a server marks
  read-only it was worse, because the sync said the opposite out loud: "nothing
  is written over it, so nothing is lost", in the same pass that removed the row
  it was talking about. The same thing happened on a calendar you can write to
  whenever Allow Changes was off: the summary said the change was waiting for
  you to turn the setting on, and the row holding that change had already
  gone.

  Being unable to save is one thing and losing the words with no word about it
  is another, so the sync now says which calendar it is and what it means:
  "Term dates: 1 change made here cannot be sent, because this is a calendar
  this program can only read." It is read out and shown, not only written to the
  log, and it is said on every sync rather than once, because nothing resolves
  it on its own. One sentence per calendar, not one per event.

  The same sentence is now said for a calendar a server marks read-only, where
  the change was already being kept safely and nothing had ever mentioned it.

  **What it does not do.** Adding the event again to a calendar you can change
  is the only way to have it saved, which is what the sentence says. Moving the
  event to another calendar is on the menu and does not work for this: the moved
  row keeps the identifier and address it was stored under, so the next sync
  either sends the change back to the calendar that would not take it or reports
  that it does not know where the event lives. That is a separate gap and it is
  not fixed here.

- **A note that mentions the end of an event no longer destroys the event's
  repeat rule.** If you typed something like "Say END:VEVENT when you are done"
  into an event's notes box, or if the calendar you subscribe to carried a note
  that did, the next time this program read that event it stopped reading at
  those words. Everything written after the note was invisible to it: the repeat
  rule, the days the series had been called off, and the rest of the note
  itself.

  On a calendar you can write to, the damage went further. The next edit to the
  event, even something as small as a change of title, sent the server a copy
  with no repeat rule, no cancelled days and no note. The server accepted it, so
  the sync reported no errors and the event stopped waiting to be sent, and the
  repeat rule was gone from the server for good with nothing left to try again.
  On a subscribed calendar nothing was written, but the event still showed as
  happening once instead of every week and nothing said why.

  The cause was that two pieces of code answered the question "where does this
  event begin and end", and they answered it differently. Now one routine
  answers it and everything that reads or writes a calendar asks that routine.
  An appointment that carries a reminder also reads correctly now: an
  appointment with no note of its own used to come back wearing the words its
  reminder says.

- **A calendar server's wrong answer can no longer overwrite somebody else's
  appointment.** Before sending a change, this program asks the server for the
  document it holds and edits that. It did not check that the document it got
  back was for the event being changed. A server answering with the wrong
  resource, or an address that had gone stale, meant this program wrote your
  appointment over a stranger's and both syncs counted it a success. It now
  refuses to send anything unless the document really holds the event, says so,
  and leaves the change waiting so the next sync tries again.

  Known limitation: if a server really does hold a different identifier at that
  address, and keeps holding it, the change will be refused at every sync and
  will never go out. You would see the message each time rather than losing the
  edit quietly, which is the trade made here, but nothing yet gives up and tells
  you the address is wrong. None of this has run against a real calendar server.

- **A contact card written in small letters imports now.** The card format says
  a property name means the same however it is written, and plenty of software
  writes them in small letters. This program read them only in capitals, so a
  file exported by that software imported as nothing at all and the import
  reported that it had run. Cards are now read whatever case they are written
  in. A property name is also matched in full rather than as a prefix, so a
  `TELEPHONE` line no longer fills in a phone number, and a photo carried in the
  card is recognised in both of the ways clients write it rather than only one.

  Known limitation, and it is about which details a card carries rather than how
  they are written. Reading a card and writing one still disagree about three of
  them. The two parts of a name are neither written into a card nor read from
  one, so a card holding "Grace" and "van der Berg" separately imports with both
  boxes empty, and exporting a contact who has them writes only the whole name.
  A relationship is written into a card and never read back. A department is
  written only for a contact with no company, and it is read back as the company
  with a stray semicolon in front of it. Export a contact, import it again, and
  those are what is different. None of it changed here.

  Superseded in an unreleased change: all three travel both ways now. The two
  parts of a name, how somebody is related to you and a department are each
  written into a card and read back out of one. The entry about a contact
  exported to a file and imported back, further up, is the whole of it.

- **The sign-in pages your browser shows are readable pages now.** When you sign
  in to Gmail or Outlook, the provider sends your browser back to a page this
  program serves, and that page is the one part of signing in you meet as a web
  page rather than as a window. There were four of them and none was a whole
  document: no language, so a screen reader read English sentences in whatever
  voice it was left in, which on a computer set to another language makes them
  unintelligible; no title, so the tab announced its address; no heading, so
  there was nothing to jump to; and no stated encoding, so an accent could
  arrive as nonsense. Each one now says it is in English, carries a title naming
  Wixen Mail, opens with a heading, and states its encoding.

  What the pages say has been rewritten in plain language and now tells you what
  to do next. The one shown when a sign-in is refused no longer repeats the
  provider's own wording at you, and the one shown when the reply does not match
  no longer says "CSRF state does not match", which named the problem in
  language nobody outside this trade uses and said nothing about what to do.

- **Text arriving from outside is no longer put into the sign-in page as
  markup.** The page shown when a sign-in is refused used to write two values
  out of the address it was sent, the error and its description, straight into
  the page without escaping them. Anything able to send your browser to that
  address while the sign-in was open, which is a two minute window, could
  therefore put its own markup into a page you were reading, in the middle of
  signing in to your mail. Those two values are no longer on the page at all.
  Wixen Mail still shows what came back through your browser, in its own window
  as text, so you can still see why a sign-in failed. It is now shown as a
  single line and cut off if it is very long, so one reply cannot fill the
  window or read aloud for a minute. A provider can refuse a second way as well,
  in its answer to the request this program makes for your access token, and
  both ways now go through the same limit.

  Known limitation, and it is not fixed: while it waits, the sign-in listens on
  every network connection this computer has rather than only on the computer
  itself. Narrowing that is a one word change and would be the right one, but it
  cannot be made without trying it against a real browser and a real mail
  provider, and no part of signing in has ever been run against either. The
  reason it has not been narrowed is written beside the code.

- **A mail provider that refuses a sign-in can no longer read a page of text at
  you.** There are two ways a provider can refuse. One is on the page it sends
  your browser back to, and what that one says has been cut to a single short
  line since the change above. The other is the request this program makes for
  your access token, and that one was not cut at all: whatever the provider put
  in its explanation went whole into the account screen's status line, which a
  screen reader reads out. A five thousand character explanation was five
  thousand characters read aloud, with no way to stop it short of leaving the
  screen. Both ways now go through the same limit, so there is one rule rather
  than two.

- **A long line from a calendar server is no longer read half of.** The calendar
  standard makes a server break any line longer than 75 characters and carry the
  rest on the next one. Reading an event, this counted the carried-on part as
  something it did not recognise and passed over it, so every property long
  enough to be broken up was read as far as the break and no further.

  What that cost, in the order it matters. A repeating event that has had days
  cancelled out of it names them on one line, and five cancelled datetimes is
  already past the limit, so the last of them was thrown away: a meeting you had
  cancelled was announced again on the day you cancelled it for. A long title
  came back cut mid-word, which is the title read out. A repeat rule ending on a
  given date could lose that date, and a series that was meant to stop then
  never stopped. A long note or a long place came back cut in the same way, and
  a start time could lose its own digits.

  The identifier a server gives an event is often long enough to be broken up
  too, and that one behaves differently from everything above. Half an
  identifier is not the name the server knows the event by, but nothing went
  wrong on the day it was stored: the server sends the same document at every
  sync, the same half was read out of it every time, and the event went on
  matching itself. The cost falls on the first sync after this change, when the
  whole identifier arrives and no longer matches the half stored beside it. What
  that would have cost, and what stops it, is at the end of this entry.

  The same applies to a published calendar you subscribed to, which is written
  by somebody else's software and broken up the same way.

  Lines are now put back together before the event is read, which is what the
  half of this program that sends a change has always done before reading the
  document it is changing. Guests, alarms and everything else this program
  passes through untouched were never read here and are unaffected.

  A value read short is put right by the next sync of that calendar, with
  nothing to ask for and nothing to press. The server keeps the document and
  every sync reads the whole of it afresh.

  **The identifier needed more than that**, because putting it right changes
  what the event is called. An arriving event is matched to the copy stored here
  by its identifier, so a copy stored under half of one matches nothing when the
  whole one arrives: it would be stored a second time and the copy already here
  removed as one the server had dropped, taking the category, the guests and the
  alerts typed on this computer with it. A calendar server has a second name for
  an event, the address it lives at, and that address does not change. An
  arriving event is now matched on either, so the copy already here keeps its
  place and everything typed onto it stays. The address is exact rather than a
  guess at how much of an identifier was kept, and where two stored events share
  an address, which should never happen, neither is matched that way.

  A calendar you subscribed to has no such address, because a feed names its
  events and nothing else. So an event in a feed whose identifier was cut short
  is stored afresh on the first refresh after this change, and anything typed
  onto it here, such as a category or an alert, goes with the copy it replaces.
  Everything the feed itself carries arrives whole, because a feed is read whole
  every time.

- **An event a calendar server sends without an address of its own no longer
  takes the calendar's address for it.** A server says where each event lives,
  and when it did not say, this read the answer as the address of the calendar
  itself. A change to such an event would then have been addressed to the whole
  calendar rather than to the event, and every event arriving that way read as
  living in the same place, which is one of the two names an arriving event is
  matched to a stored one by. Nothing is guessed at now: the event is stored
  with no address, the sync says plainly that where it lives is not known here
  rather than sending the change anywhere, and the next read of that calendar
  fills the address in. A server answering this way is a broken one and none has
  been seen doing it.

- **A repeating event first dated a very long time ago no longer stops halfway
  through the calendar.** An event that repeats every day and carries a first
  date more than about a hundred years back was shown for the first few months
  of the calendar and then simply stopped, with no gap, no warning and nothing
  to look wrong. Scroll forward and the appointment had quietly ceased to exist.
  Older still, and it never appeared at all.

  The cause was that the days were worked out by counting forward from the
  series' own first date, one repeat at a time, with a cap on how far that
  counting would go. A daily series starting in 1917 used the whole cap getting
  to today and ran out part way through the year ahead.

  The days are now worked out from the part of the calendar you are looking at.
  A daily event dated 1917 shows on all 546 days of the calendar instead of 151,
  and one dated 1900 on all 546 instead of none. The days a series falls on are
  exactly the days it fell on before for every series that already worked: only
  the ones that were being cut short have changed. Opening the calendar is also
  quicker where old repeating events are stored, because the years in between
  are no longer walked through one day at a time.

  A series set to stop after so many occurrences is the awkward one, because
  showing it correctly means knowing how many have already gone by. For a daily
  or a weekly series that number is now worked out by arithmetic, so it starts
  from the part of the calendar you are looking at like any other. A daily
  series dated 1900 set to stop after 50,000 times showed on none of the 546
  days and now shows on all of them, and one set to stop after 46,187 times
  still stops on the 15th of June 2026, the day it was always going to stop on.

  A monthly or a yearly series that stops after so many occurrences is still
  counted from its own first date, one month or one year at a time, because
  there is nothing to multiply: only seven months of the year have a 31st, and
  only one year in four has a 29th of February. That counting reaches back 3,300
  years for a monthly series and 40,000 years for a yearly one, which is further
  than any date a calendar can hold, so no such series is cut short by it.

- **A day you cancelled out of a repeating Google event stays cancelled.** Three
  separate faults dropped the days you had called off. Each one had the same
  result: the occurrence came back, and a meeting you had cancelled was
  announced again on the day you cancelled it for.

  When a repeating event has a time zone on it, which most do, Google writes the
  zone into the line naming the days called off, and that shape was not
  recognised at all. An event is also allowed to name its called-off days on
  more than one line, and only the first line was kept, so every day you
  cancelled after the first came back. Lastly, a zone whose name has a digit in
  it, such as `Etc/GMT+5`, put that digit on the front of the date, which then
  read as no date at all and was passed over without a word.

  All three are fixed. The days a Google event calls off are now read by the
  same code that reads them from a calendar server, so the two arrive stored the
  same way, and the part that works out which days a series falls on no longer
  depends on which of them stored it.

  The last of the three is put right as the calendar is read, so events already
  stored need nothing. The first two are not: what was thrown away was thrown
  away before the event was saved, and it comes back only when Google sends that
  event again, which it does when the event next changes or when the whole
  calendar is read afresh. So a series nobody has touched since keeps showing
  the day you cancelled, and there is no way to ask for a fresh read from the
  product today.

- **A calendar written in small letters is now read.** The calendar standard
  says that a property name such as `SUMMARY`, and the `BEGIN:VEVENT` that opens
  an event, mean the same whichever case they are written in. Only capitals were
  matched.

  An event from a server that writes them in small letters never reached the
  calendar at all. Nothing arrived half read: the identifier and the start time
  are the two things an event cannot be read without, so the whole appointment
  was dropped as it arrived, with nothing said about why. A subscribed feed
  written the same way was split into no events and showed as an empty calendar.

  Property names and the lines that open and close an event are now matched in
  either case, in a calendar from a server and in a subscribed feed alike.

  That alone did not make such a calendar usable, and three entries below are
  the rest of it. The zone a meeting is in was matched in capitals only, so a
  calendar in small letters was read right down to the appointment and came back
  with no zone on it. The two letters that shape a timestamp were matched in
  capitals only, so the start was not something the calendar could show on a day.
  And the half of this program that writes a change back was still looking for
  capitals, so an edit to such an event was thrown away without a word. The four
  together are what makes a calendar written in small letters read, and stay
  read when you change something on it.

  Almost every server and every publisher writes these in capitals, so this may
  never have reached anybody. Nothing that was read is lost for good: the server
  keeps the document and every sync reads the whole of it afresh. An edit thrown
  away is a different matter, and the entry about the writer says what that cost.

- **A change to an event written in small letters now reaches the calendar
  server.** Making such an event readable, above, also made it editable, and the
  half of this program that writes a change back still looked for the lines
  opening and closing an event in capitals only. So it found no event to change.
  It copied every line through untouched and sent the server its own old words
  back, the server accepted them, the sync reported success, and the change
  stopped waiting to be sent, so nothing ever tried again.

  What that looked like: you edited an appointment, it looked saved, and the
  next time the calendar was read it said what it said before. Nothing told you.
  It is the worst shape a bug can take here, because the words you typed were
  gone and only you knew they had ever existed.

  Both halves now read a name the same way whatever case it is written in. This
  was only ever true between two commits of the same unreleased round of work,
  so no build that has gone to anybody could do it, and no calendar anybody
  keeps has lost anything to it.

  Two smaller things went with it. An event whose stored version number was
  written in small letters had that number left in the document beside the new
  one, so another calendar program picking the higher of the two could believe
  the copy your change replaced. And an alarm inside such an event was no longer
  recognised as a block of its own, so saving the appointment took away the
  words the alert shows, along with the appointment's own note. The alert then
  had nothing of its own to say, and a calendar program with nothing to say
  falls back to the appointment's title.

  The time the alert goes off is not affected, and was not affected before
  either. An alarm is timed by how long before the appointment it fires, this
  program never writes that, so it comes through whatever else happens.

- **Editing a repeating event no longer wrecks the occurrences you had moved.**
  When you move or retitle a single occurrence of a repeating event, a calendar
  server keeps the series and each changed occurrence together in one document.
  Saving any change to that event wrote this program's version of it into every
  occurrence in the document, not just the series, and it wrote them all into
  the same place. So the series came out with two titles, two start times and
  two repeat rules, which is not a valid event and which some servers will
  refuse outright, and every occurrence you had moved lost its own title and its
  own time and was left as nothing but a marker saying which week it replaced.

  Only the series is changed now, and every occurrence changed on its own goes
  back with the values the server had, in a document written out afresh. This
  one has nothing to do with letter case. It was found while fixing the entries above and it was there
  before them.

  What was lost is not recoverable from here, but it was almost certainly never
  lost: nothing in this program has run against a real calendar server, so the
  only way to have met it is to have tried a real account yourself.

- **A change that did not get made is no longer sent and counted as saved.** The
  bug above was able to lose an edit because of what happened after it: a
  document with the change not in it was still sent, the server accepted it,
  and the change stopped waiting to be sent, so nothing ever tried again.

  The document that would go to the server is now read back before it goes, with
  the same routine that reads a calendar arriving, and the change has to come
  out of it: every line the change is made of, once each, among the event's own
  lines rather than an alarm's, under the same identifier. If it does not come
  back out, nothing is sent, the change stays waiting so the next sync tries
  again, and the sync counts a failure where it used to count a success.

  Which of the four things went wrong is written to the log file, and the screen
  shows only the count of errors, so you learn that something failed and not
  what. That is how every calendar sync failure has always been reported here
  and nothing about it changed. If a change of yours keeps waiting sync after
  sync, the log is where the reason is.

  What that covers, and what it does not, because the difference is the whole
  value of it. It covers the change being written somewhere no reader will look,
  the change being written into somebody else's appointment, a document that
  opens an event and never ends it, a line lost or doubled between building the
  document and writing it out, and a property this program starts writing
  without also taking the server's old copy of it out first, which would
  otherwise leave two start dates on one appointment.

  It does not cover a wrong answer from the one routine that decides where an
  event begins and ends. Both halves ask that routine, so a wrong answer looks
  right to both of them and this check cannot see it. What closes that class is
  there being one routine rather than two, which is the first entry in this
  section, about a note that mentions the end of an event, and not this check.
  Nor does it cover a line neither half recognises as a property at all: a
  title the server wrote in a shape this program does not read is not taken out
  and not counted, so the document goes out carrying two of them.

- **The timezone on a meeting is read whether the server writes it in capitals
  or not, and quote marks around it are no longer part of it.** The zone a time
  is named in arrives as `DTSTART;TZID=Europe/London:...`, and that was matched
  in capitals only. A calendar written in small letters throughout was read
  right down to the appointment and then came back with no zone on it, so a nine
  o'clock London meeting showed at nine o'clock in whatever zone this computer
  is set to.

  It did not stop at reading. This program writes a zone only when it has one,
  so the next change you saved took the zone off the server's copy as well, and
  every other program reading that calendar then had the meeting at the wrong
  time too.

  The standard also lets a server put quote marks round the zone name, and some
  do. Kept, the quote marks are part of the name, which matches no zone in the
  world, so the meeting was again read in this computer's own zone and the quote
  marks were written back into the document on the next save.

- **A start time written with small letters is a time again.** The two letters
  that shape a calendar timestamp, the `T` between the date and the clock and
  the `Z` that says the time is in UTC, mean the same in either case. Matched as
  capitals only, `20260305t090000z` was handed on to the calendar exactly as it
  arrived, so the appointment had nothing the calendar could read as a day and
  showed on no day at all. A time stored that way by an earlier build also lost
  the `Z` on its way back out to the server, which turns a nine o'clock UTC
  meeting into nine o'clock in no zone at all, and that is nine o'clock wherever
  the reader happens to be.

- **A comma in an appointment's title no longer arrives with a backslash in
  front of it.** A calendar document puts a backslash before a comma, a
  semicolon and a backslash, and writes a line break as the two characters `\n`,
  so that words somebody typed cannot be read as the document's own punctuation.
  This program has always written them that way and never took them off again on
  the way in. An appointment called "Lunch, then a walk" therefore arrived from a
  calendar server as `Lunch\, then a walk`, and that is what the list showed and
  what a screen reader read out. A note written on another program as two lines
  arrived as one line with a visible `\n` in the middle of it.

  It got worse rather than staying still. The half of this program that writes a
  change escapes whatever it is given, so every save wrote the backslashes again
  and they doubled each time, in your calendar and at the server.

  The title, the notes and the place are now read as the words they are, and the
  same words go back to the server. Not always the same characters, and it is
  worth being exact about which ones move. A line break can be written two ways
  and goes back written the one this program writes. A comma or a semicolon a
  server left bare comes back with the backslash the standard asks for. Every
  calendar program reads those the same way either way, so nothing you typed
  changes.

  One of these did lose a character, and no longer does. A backslash in front of
  anything the standard does not name, such as `\q`, used to be thrown away on
  the way in, so a title the server held as `Ten\q twenty` was read here as "Tenq
  twenty" and the next save wrote "Tenq twenty" back with the backslash gone for
  good. Both characters are kept now. The cost is that such a title now reads out
  with a backslash in it, which is what the document says is there.

  The identifier an event is known by is left exactly as it arrives: it is the
  name the server calls the event by, not words anybody typed, and it is matched
  character for character.

  A calendar you subscribed to is read by the same code and was showing the same
  backslashes.

- **A calendar file laid out with indentation is read again.** The change above
  that puts a broken line back together reads every line starting with a space
  or a tab as the rest of the line before it, which is what the calendar
  standard says such a line is. A file somebody laid out by hand, or ran through
  a formatter, uses those same spaces to show what sits inside what. Read as
  joins, the whole file ran together into a single line with no identifier
  anywhere on it, so the event was dropped and nothing was said about it. A
  calendar you subscribed to that was written that way showed as a calendar with
  nothing on it.

  A file is now read as laid out by hand wherever the two can be told apart,
  which is a line with a space in front of it directly after a line that opens
  or closes a block. Those lines carry a block name: it is short, it holds no
  spaces, and no calendar program breaks one across two lines. The spaces are
  taken off every line in such a file instead of being read as joins. Where the
  two cannot be told apart, a line starting with a space is still read as the
  rest of the line before it, because that is what the standard asks for.

  What it costs, which is worth saying plainly: a file that is both laid out by
  hand and has a long line broken across two now reads that one line short. Once
  the layout is taken off there is nothing left to tell a join from an indent.

  There is a sharper edge to that, and it is not fixed. The second half of a
  broken line then stands on its own, and where it happens to begin with a
  property name and a colon it is read as that property. A note carrying on with
  "Location: the car park" gives the appointment that place instead of the one
  written on its own line further down. It needs a file that is indented and
  broken and broken at exactly that word, so it is unlikely, and the alternative
  is guessing at which lines read like properties, which is the guesswork this
  rule exists to avoid.

  The event itself still reads, where before the file had no event in it at all.

  The same layout also ran an event's closing line together with the calendar's
  own closing line in a document being sent back to a server, so neither closed
  properly. That is fixed by the same change.

- **A calendar nothing could be read from no longer looks like a calendar with
  nothing on it.** An event this program could not make sense of was passed
  over, which is right, because one bad entry must not cost you the other two
  hundred. When every entry was passed over the answer was the same empty list,
  and an empty calendar is exactly what somebody then goes looking for a broken
  account over.

  A calendar sync, or a refresh of a calendar you subscribed to, that could read
  none of what arrived now counts that as a problem, so the sync no longer
  reports plain success over an empty calendar. A calendar that really has
  nothing on it is still reported as the ordinary thing it is. Nothing already
  stored is deleted in either case.

  A feed that failed while being read was also being reported as a feed that
  failed to arrive, which sends you looking at your network for something that
  is not there. Those two now read differently.

  Two known limitations, both real. What the window says and what is announced
  is still only that the sync had a problem, and how many; the sentence saying
  what the problem was, and how many entries could not be read, goes to the log
  and no further. And this tells apart "nothing could be read" from "there was
  nothing on it", but not the case in between, where most entries read and a few
  did not. Those few are still passed over without a word.

- **An event repeating on a weekday no month has is now shown once, and says
  so.** A monthly repeat can name which weekday of the month it falls on, such
  as the second Thursday. No month has a sixth Monday, but a rule could still
  ask for one, and an event asking for one appeared on no day at all: the
  calendar simply did not show it. With some of the numbers a calendar server or
  a provider can send, it was worse, and the whole calendar list stopped being
  built. Such an event is now shown once, on its own start day, with the same
  sentence used for every other repeat that cannot be worked out.

- **A repeating event with an impossible gap between occurrences no longer lands
  on days it is not on.** A calendar server or a provider can send a repeat rule
  saying something like "every 4294967295 weeks". The gap from one occurrence to
  the next was counted in a size too small to hold a number that big, so it came
  out as some other, much smaller gap and the event appeared on days nothing had
  asked for. Worse for a yearly rule, where the wrong gap was no longer a whole
  number of years, so the event moved to a different month and stayed there. The
  gap is now counted at full size. A rule whose next occurrence falls off the end
  of the calendar shows its first day and stops.

- **Deleting a contact group no longer offers to delete the people in it.** The
  question said "Delete the contact group "Team A" and the 3 contacts in it?
  This cannot be undone." It never touched the contacts. It now says the people
  in it stay in your address book.

- **Deleting a contact group no longer promises it will come back.** The same
  question added "It will come back at the next sync", which was said for
  anything deleted while a mail account was open. Nothing has ever sent a
  contact group to a provider, so the sync was never going to mention it. The
  sentence is now added only where there really is a copy at the provider, which
  is a calendar or a task list.

- **A contact group no longer disappears when you open a different account.** A
  group was filed under whichever account was set as the default, and the
  sidebar only ever read the account being looked at, so a group could be made
  and vanish. Groups are now kept on this computer and are shown whichever
  account is open. Groups made before this change are still found, so a group
  somebody thought they had lost will reappear.

- **Deleting a message no longer destroys text that exists nowhere else.**
  Deleting marks the message and drops its saved text, on the reasoning that it
  can be downloaded again if the message is brought back. That is true of mail
  on an IMAP server and untrue of everything else: mail collected over POP was
  downloaded once, and a copy of a sent message saved here was never on a server
  at all. Both are now kept. A filter rule carrying a delete action reaches this
  today, so it was not hypothetical.

  The same rule is applied to the routine that drops old message text to keep
  the cache within a size limit. Nothing calls that routine, so no size limit is
  actually being applied, and this release does not start applying one: turning
  it on as it was written would have deleted the only copy of every POP message
  and every saved sent message on the machine.

- **A refused delete no longer says the change was undone.** The row only leaves
  the list once the server has agreed, so when a delete failed nothing had been
  put back. Marking read and flagging do change the row first, and those still
  say so.

- **A reply now lands in the conversation it answers.** Every reply this program
  sent started a new conversation in the recipient's client, because the two
  headers that say which message is being answered were never written. They are
  now, both of them: the message being answered, and the whole chain of messages
  before it. For somebody working through a mailbox by ear, this is the
  difference between one thread and a scattering of unrelated messages.

  This is carried all the way through. It survives Save Draft, so a reply put
  aside half-written and reopened tomorrow still goes out inside its thread.

  Known limitation: no reply from here has been read by a real mail client, so
  whether the threading is right at the far end is untested. What can be checked
  is that the headers are built correctly and reach the outgoing message, and
  that is checked.

- **You can now give the name recipients see.** Mail went out as a bare address
  where every other program sends a name. The account dialog now has a box for
  it, labelled "The name people see when your mail arrives", separate from the
  Account Name box, which is the label you gave the account and is usually
  something like "Work". Leaving it empty sends a bare address, which is what
  every message sent until now carried, so nothing changes until you type
  something.

  The same name goes on the copy filed in your Drafts folder, so what you come
  back to on another device is the message you were writing.

  Known limitation: not tried against a real mail server.

- **Reply to Sender Only is on the Message menu, so its key works.** The command
  existed, with a toolbar button and three lines in the shortcuts document
  saying `Alt+Shift+R`, and nothing bound the key: Windows dispatches menu
  accelerators, and there was no menu item. The only way to reach it was the
  mouse, which for the people this program is for means it did not exist.

- **A read receipt is now filed against the message it is about.** It named the
  subject and reached the right person, and arrived as loose mail rather than
  inside the thread, because the message list did not carry the original's
  identifier. It does now.

### Changed

- **Reply All now also reaches the person who wrote the message.** Before, on a
  mailing list, a reply to all went to the list and everybody copied and never
  to the author, because the author had asked for replies to go to the list.
  This is a decision rather than a fix: some lists treat a personal copy as
  rude, and reaching the person you are answering was judged the lesser cost.
  They are added to Cc, and only when they are not already among the recipients
  and are not you. Reply to Sender Only is unchanged and still reaches one
  person.

- **What a reply is about to do is said more usefully.** A reply to one person
  now names them, "Reply to sender only, Ada Lovelace", instead of counting to
  one. A reply to several still gives the count rather than reading out a list.

### Fixed

- **The shortcuts document said `Ctrl+Shift+G` gets older messages, and nothing
  was bound to it.** The key is `Shift+F9`, which is what the menu has always
  had. Worse, the sentence spoken after a sync said `Ctrl+Shift+G` too, so the
  program was telling people to press a key that did nothing. Both corrected.

### Added

- **A change you make to a calendar you added by its address is now sent back to
  that server.** Until now it was not. You could edit an appointment in one of
  those calendars, the edit would look like it worked, and the next sync would
  quietly put the server's version back. Adding an event, changing one and
  deleting one all reach the server now.

  Nothing goes anywhere unless Allow Changes is on for that account. With it
  off, the change waits on this computer and the sync says how many are waiting,
  and no request of any kind is made.

  **What happens to the parts of an event this program does not understand.**
  A calendar server holds a whole document per event, and a change replaces the
  whole thing, so this could easily have wiped out everything it does not model.
  It does not: the document the server holds is read first, the handful of
  things this program knows about are replaced inside it, and every other value
  goes back with the same words it arrived with. Guests keep their invitations
  and their answers, alarms keep their own times, and anything another calendar
  program put there is kept.

  The values go back unchanged. The document itself is written out afresh
  rather than copied, so it is the same value rewritten and not the same bytes:
  a line the server had broken across two is put back together to read it and
  broken again to send it, and the break can land in a different place. That
  breaking is new. Before it, a guest with a long name and an address on the
  same line went back as one line of 101 characters where the standard allows
  75, and a server that checks what it is sent can refuse the whole change over
  it.

  What this program replaces is the title, the notes, the place, the start and
  end, the repeat rule, the days the series calls off, and the status. Emptying
  one of those here clears it on the server too, because that is what emptying
  it means. Those it does replace go back written the one way this program
  writes them, even where you changed something else on the event and not them.
  That is the same words and not always the same characters; the entry above
  about a comma in a title says exactly which characters can move.

  **If somebody else changed the same event first**, from a phone or another
  program, the change is refused rather than written over theirs, and the sync
  reports it. Opening the event and saving again will send it. A deletion is
  deliberately not checked this way: you asked for the event to go, and a check
  there would make the deletion fail on every sync from then on with nothing you
  could do about it.

  A server that does not name versions is a server this check cannot be made
  against. The change still only alters what that server had a moment earlier,
  so nothing else is lost, but a change made from another device in the same few
  seconds could be overwritten.

  **None of this has been tried against a real calendar server**, so treat a
  calendar you care about with caution. The window for adding one says so.

- **A birthday sent to a calendar server now still happens every year.** The
  repeat rule was written only for events with a time on them, so any whole-day
  series, which is what a birthday and an anniversary are, would have gone out
  as a single day.

- **A category you typed on an event now goes to Outlook with it, and one set in
  Outlook now comes back.** Outlook has categories and shows them by name and
  colour, and this was filling one in and then not sending it, so an event
  arrived filed under nothing. It now goes both ways, so refiling an event in
  Outlook refiles it here on the next sync.

  Outlook's copy is the one that wins, the same as for the title, the time and
  everywhere else the two disagree. That is a change from before: a category
  taken off in Outlook now comes off here as well, instead of the copy on this
  computer being put back. Google Calendar has no category of its own, so
  nothing changes there and a category you typed on a Google event still stays
  where you put it.

  Not confirmed against a live account, so whether Outlook accepts the list as
  sent is still unknown.

- **A repeating event now shows on every day it falls on.** A weekly meeting
  used to appear once, on the day it was first set up, and nothing told you why
  the other weeks were empty. Every day of a series is now a row of its own, in
  date order with everything else, and each one says how often the event comes
  round and when it stops: "every week", "every two weeks", "every week on
  Tuesday and Thursday", "every week, until 2026-09-30".

  A day the series has called off is no longer shown, so a meeting that was
  cancelled is not one you turn up to.

  The words are the same ones the New Item form offers when you set a repeat, so
  a series is described one way wherever you meet it. Where a rule uses
  something the form cannot express, it is described in plain words instead.

  Repeats that this cannot work out are not guessed at. The event is shown once
  and says so, out loud, without you having to ask for the full reading: that is
  the one case where there is nothing else on the screen to tell you.

- **The reading of an event now says what kind of day it is.** A birthday, a
  holiday and a dentist appointment sound different from each other. Categories
  could be set before and nothing ever read them back.

### Changed

- **Changing a repeating event asks which days you mean.** All the days of a
  series come from one stored event, so changing the fortieth Tuesday would have
  rewritten all fifty-two and told you the event was updated. You are now asked
  whether you mean just that one day or every day in the series, before anything
  is written and before the editor opens. The same question is asked before a
  delete.

  **Changing one day on its own is refused for now**, with a sentence saying so
  and saying that nothing has been changed. It is refused rather than quietly
  widened to the whole series, because widening it destroys the other days'
  details and cannot be undone. Choosing every day in the series works.

- **The count of events in the calendar will look larger.** It now counts days
  rather than stored events, so an account with one daily meeting counts that
  meeting once for each day it falls on.

### Fixed

- **A comma in an appointment's title no longer splits it in two, and a line
  break in the notes can no longer add anything to the event.** Text you typed
  went into the document for a calendar server exactly as typed, and four
  characters mean something there. A note with a line break in it could write
  whatever the next line said straight into the appointment as a property of its
  own.

- **A change waiting to be sent is no longer written over by the next sync.**
  When a change could not go, because Allow Changes was off or the server was
  unreachable, the read that followed replaced your words with the server's. The
  change then went on waiting, but what it was waiting to send was the server's
  own copy back to it.

- **A repeat rule was being thrown away on the way to a calendar server.** The
  document built for a calendar server carried no repeat rule, no called-off
  days and no time zone, so the first change ever sent to one would have turned
  a weekly meeting into a single appointment and moved the times of anything not
  kept in UTC. No calendar was harmed, because nothing sent one until the change
  above in this same release; the document was made correct first.

  A time typed here was also being written in a shape no calendar server reads
  ("20260306 0900", with a space and no seconds), which a server that checks
  what it is sent refuses outright.

- **A repeat rule from Google was being read as the wrong thing.** Google sends
  the rule, the called-off days and any extra days as separate lines of one
  list, and whichever line came first was stored as the rule. A list of
  called-off days stored as a rule repeats nothing at all.

**Known limitations for repeating events**, and none of this has run against a
live account or a live calendar server:

- A series you make here still reaches Google or Outlook as a single
  appointment. The repeat rule is deliberately never included in a change sent
  to a provider, because sending an empty one is an instruction to stop
  repeating and would flatten the series at their end. So the days show
  correctly here and not there.
- A day of a series that was moved or renamed at the server is shown at its
  original time and under its original name, and you are not told. Only whole
  days that were called off are handled.
- A repeating event from Google or Outlook does not say that it repeats. Those
  two send the days themselves rather than the series, so the rule never arrives
  and there is nothing to read out. The days are all there.
- Days are worked out for 180 days back and 365 days forward, the same stretch
  the syncs ask for. A series is not shown outside that.
- Rules using positions within a month, particular months of the year, or
  anything finer than a day are not worked out. Those events are shown once and
  say so.

- **You can now add a calendar by its address.** Tools, then Add a Calendar by
  Address. Two kinds work. A calendar held on a calendar server, such as
  Fastmail, Nextcloud or a server your workplace runs, which you sign in to: it
  asks the server what calendars it has and you choose one, so you never have to
  know the path to it. And a calendar feed, the kind a school or a football club
  publishes as a file, which needs no sign-in.

  Until now nothing in the application could set either address, so the code
  that keeps those calendars up to date, and the code that erases their sign-ins
  when you uninstall, could never run. Both are reachable for the first time.

  The sign-in goes to the Windows credential store, where account passwords
  already go, and never into the database. That means copying or backing up your
  profile does not carry a calendar password with it, and uninstalling clears it
  from one place.

  A sign-in is only sent over a secure connection. An address that is not secure
  is refused with a sentence saying why and what to do instead, because a
  password sent over an ordinary connection is readable by anyone on the
  network. A feed carries no sign-in, so an ordinary address is accepted for
  one.

  **The window stays awake while the server is asked.** Asking a calendar
  server what it has can take up to thirty seconds, and the window used to sit
  frozen for all of it: nothing on screen moved, nothing answered a key, and a
  screen reader said nothing at all, so there was no way to tell whether it was
  working, had finished, or had died. The asking now happens away from the
  window. A small window says that it is looking and how long it may take, and
  offers Stop looking, which leaves everything exactly as it was. When the
  answer comes back you are told how many calendars were found before the list
  appears, because a list filling up in silence tells you one row and nothing
  about how many there are. A server that never answers now says so and says
  what to try.

  **Known limitations:** none of this has been tried against a real calendar
  server. A calendar you add this way is read, not written: events you change in
  it stay on this computer and are not sent back, which the window says before
  you add anything. Some servers write their answers with different shorthand
  from the four this program reads, and those are reported as having no
  calendars when they may have several. A calendar is filed under the account
  that was open when you added it, and only that account's calendars are brought
  up to date while it is the one on screen.

### Fixed

- **A change to an Outlook contact is now addressed at the right contact.**
  Outlook's own name for a contact was dropped into the address as it came, so
  one containing a slash or a question mark, which its names can, pointed the
  change at some other contact or at none. A deletion sent to the wrong contact
  cannot be taken back. Event and calendar names were already handled this way
  and contact names now are too. Not seen happening: no account has run against
  a live Outlook address book yet.

- **A contact's phone numbers, email addresses and postal addresses can no
  longer disappear as a set.** Each is kept as a list in one field, and a list
  that was missing any part of any entry could not be read back at all, which
  counted as having none. Because a change sent to Google says which parts of a
  contact it may alter, and all three are named, an unreadable list would have
  told Google to remove every number, every address, or every email that person
  has. A missing part is now simply blank. Nothing in the program writes a list
  in that shape today, so this closes a hole rather than fixing something seen
  happening.

- **Two buttons in the calendar sidebar now do what they say.** New and Delete
  both said to use File, then New, then Calendar, and that menu item does not
  exist: the New menu offers a message, an event, a reminder, a task, a note, a
  contact and an account. Both buttons now make and remove a calendar
  themselves, the way the same two buttons in Contacts already did.

- **A change you make to an appointment now reaches your calendar.** Adding,
  editing or deleting an event in a Google or Outlook calendar used to stay on
  this computer, and the next sync quietly put the old version back. The change
  is now sent up the next time that account syncs, and the status line says how
  many changes went.

  It only happens for an account whose owner has turned on Allow Changes, in
  Settings. With that setting off nothing leaves the computer, the change waits
  rather than being lost, and the status line says so and names the setting
  instead of reporting an error you cannot act on.

  A change is sent as a change rather than as a replacement, so moving a weekly
  meeting does not turn it into a single appointment and changing the room does
  not uninvite anybody.

- **Emptying a field now empties it at your provider.** Deleting the notes, the
  place or the title of an event left the old text in your calendar, because an
  empty value and an untouched one went up as the same thing. They are now told
  apart, so clearing something here clears it there.

- **An alert you set now reaches Google.** The alert was stored without saying
  how you should be alerted, and Google drops an alert that does not say, so
  every reminder set in this program was silently discarded on the way. Outlook
  was never affected.

- **A whole-day event is a whole day at both providers.** A one-day event was
  sent starting and ending on the same date, which both providers read as no
  time at all, so it was refused or drawn as nothing. It is now sent ending the
  following day, which is how both of them write a whole day.

  **Known limitations:** none of this has met a live Google or Outlook calendar.
  A calendar from a calendar server, such as Fastmail or Nextcloud, still does
  not accept changes: a change there replaces the whole appointment, and this
  program does not yet keep enough of the server's own copy to put back what it
  would otherwise destroy, so an edit to one of those stays on this computer.
  An event marked tentative or out of office reaches Google as busy, because
  Google has only two words where this program has four.

- **One account can hold two calendars with the same name.** A Work calendar of
  your own and a Work calendar shared to you, on the same account, used to be
  one calendar too many: the second was refused with a database message nobody
  could act on, and it was never saved. Calendars are now told apart by the
  calendar, not by what it is called, so both are kept and both are yours to
  rename.

  Nothing about the calendars you already have changes. Every one of them keeps
  its own row, its own name and its own settings, and none of them is given a
  new identity, so a CalDAV sign-in still belongs to the calendar it was saved
  for.

- **An event stays in the calendar it belongs to.** The same event in two
  calendars, which is what you get from subscribing to one holiday feed twice or
  from a meeting that is in a shared calendar and in your own, was stored once
  rather than twice, and it moved to whichever calendar was refreshed last. One
  of the two calendars was always missing it, and which one changed every time
  anything synced.

- **Events that belonged to no calendar are now filed under one.** Events stored
  before this program had calendars to put them in appeared only in the combined
  view, and no single calendar's list could show them. Each is now filed under
  the calendar its own server syncs into, chosen the same way every time. An
  account with no calendar of that kind is left alone rather than having one
  guessed for it.

  **Known limitations:** where an account already has two calendars from one
  server, an event that belonged to none of them goes to one of the two and it
  may not be the one it came from. You can move it. None of the calendar syncing
  has run against a live Google, Microsoft or CalDAV account.

- **The same person in two address books is now one contact that both know.**
  Where an account is signed in to both Google and Microsoft and a person is in
  both, there was room here for one address book only. Each sync used to take
  the contact off the other and report it as new, and it never settled. The
  previous answer, released earlier, was to let whichever address book reached
  the contact first keep it and to say the other copy had been skipped, which
  meant the order the syncs happened to run in decided what you saw.

  A contact now carries a list of the address books that know it, and each one
  says what it calls the contact. A sync matches on its own name for the person
  and leaves what the other address book said alone, so Alice in Gmail and Alice
  in Outlook are one Alice with two entries behind her. Nothing is reported as
  skipped any more, because nothing is.

  Where the two disagree about a detail, whichever synced last still wins. That
  has not changed.

- **A contact with no email address is an ordinary contact, and an account can
  hold as many as it has.** Somebody with only a phone number is normal in an
  address book. This used to tell two contacts apart by their email address, so
  every contact without one collapsed into a single row: the second one saved
  took the first one's place, and both were counted as new. Contacts are now
  told apart by the contact, so a name and a phone number is enough.

  Two things follow from the same change. Changing a contact's main email
  address now saves, where before the contact manager refused it with a message
  about a failed constraint. And a group with a member who has no email address
  now leaves that member off the To line rather than putting an empty address on
  it.

- **Editing a contact no longer cuts every contact in the account off from the
  address book it came from.** Saving one edit rewrote every contact in the
  account as one nobody had synced. The next sync then treated the whole address
  book as new.

- **A change you make to a contact now reaches every address book that has that
  contact.** Correcting a phone number used to stay on this computer, and the
  next sync put the old number back. The correction is now sent out the next
  time that account syncs, to Gmail and to Outlook if both have that person, and
  each is sent its own name for them. The status line says how many changes
  went.

  Which address books get it is yours to decide, in Settings, on the Language
  tab under Contacts: "Send a change to a contact to every address book that has
  that contact". It is on to begin with. Turned off, a change goes only to the
  address book the contact came from and the others keep what they had.

  As with the calendar, it only happens for an account whose owner has turned on
  Allow Changes. With that setting off nothing leaves the computer, the change
  waits rather than being lost, and the status line names the setting instead of
  reporting an error you cannot act on. Turning on the new setting does not
  switch any of that on.

  If one address book takes the change and another refuses it, only the one that
  refused is asked again next time, so a Gmail failure cannot send your edit to
  Outlook twice.

- **The Allowed Changes settings are on the screen again.** The section holding
  "Let Wixen Mail change my tasks, contacts and calendar", "Let Wixen Mail send
  and delete mail" and the sentence saying none of it has been tried against a
  real account was built and never placed on the panel, so the warning had
  nowhere to appear and the two boxes were not where they said they were. This
  needs a look on screen and with a screen reader to confirm it now reads in the
  right order.

- **A contact sent to Outlook now arrives with everything you typed.** Only the
  first email address and the first phone number went, no postal address went at
  all, and a website could not go because there was nowhere to put it. All of
  them now go: every address, every number in the place Outlook keeps that kind
  of number, both postal addresses, and the website. A contact sent to Google
  now carries its postal address too, which was the one thing missing there.

- **Editing a contact no longer relabels it as one you made here.** Correcting a
  detail on a contact that came from Gmail used to record it as having come from
  nowhere. Nothing visible depended on that until now; with changes going out,
  it decided where the change went.

  **Known limitations:** none of this has met a live Google or Outlook address
  book, and the version marker Google wants with a change is stored and sent but
  has never been checked against a real one. A change sent to Outlook can add
  and correct but cannot empty: clearing a contact's nickname here leaves the
  old nickname at Outlook, which is the direction that loses less. A contact you
  type in here still reaches one address book, whichever syncs first, rather
  than all of them; that is unfinished. When a change cannot be sent, the
  sentence saying why goes to the log file and the screen shows only a count, so
  you learn that something failed and not what. A contact the provider no longer
  lists is still kept here after a full re-read, unchanged: deciding to delete
  somebody's contacts because a read came back short needs to know the read was
  complete, which is separate work.

- **A task deleted somewhere else now goes from here too, however long ago it
  was deleted.** Google says a task has gone by sending it back marked as
  deleted, and it only does that for a while. A task you deleted on your phone
  while this program was shut for longer than that was never removed here. It
  stayed on the list forever, with no way to reach it from Google. Google tasks
  are now compared against what Google sent back, the same way Outlook tasks
  already were, so a task that is no longer there is no longer here.

  A removal is only decided from a complete answer. If the list of task lists
  stopped at the limit on how much one sync reads, if a list could not be read
  or saved, if a list held more tasks than one sync will take, or if a task came
  back with nothing to identify it by, then nothing is removed and the sync says
  what it could not see. A task Google names as deleted still goes in all of
  those cases, because that is Google naming the task rather than a gap in what
  came back.

  Tasks you made here are never removed by a sync. They have never been sent
  anywhere, so a provider saying nothing about them means nothing.

- **Outlook tasks are no longer removed on a partial answer.** There were two
  ways a sync could delete a task that was still there. When the list of task
  lists stopped at the limit there were lists nobody read, and a task that had
  moved into one of them looked deleted. When Outlook sent back a task with
  nothing to identify it by, the task it stood for looked absent. Both now stop
  the sync removing anything, and the second is said rather than passed over.

- **One deletion is counted once.** Where a provider sent the same task list
  twice in one answer, a single task removed from it was reported as two
  removals.

  **Known limitations:** none of this has run against a live Google or Outlook
  account. On the first sync after this update, tasks that were deleted
  elsewhere long ago and left behind here all go at once, so an account that has
  been running for a while may see several disappear together. What a sync
  removed is a number on the status line; the reasons a sync could not see
  everything reach the status line as a count of problems, and the words for
  them are in the log file.

- **F1 opens help for whatever you are looking at, and the Help menu lists
  every page.** The guides were written and they ship beside the program, and
  there was no way in: no contents, no F1, and the only button that opened one
  was on the first-run screen, which you see once. F1 used to open the About
  box, which is a version number and a licence and not what anybody presses F1
  for.

  Landing on the right page matters more by ear than by eye. Somebody who can
  see a page skims it for the part they need; somebody listening reads it in
  order, so the difference between the right page and a list of pages is
  minutes.

  **Not done:** none of this is published on the website yet. The pages ship
  with the program and open from it.

- **One list for every account's inbox.** "All Inboxes" is the first thing in
  the folder tree. Anybody with more than one account works out of one list
  rather than several, and switching accounts to find out whether anything
  arrived is worse by ear than by eye: it is a walk through a tree rather than a
  glance at a sidebar.

  Each row carries the account it came from, so flagging, labelling or deleting
  one from that list reaches the right server. That was worth fixing on its own:
  every action on a message used whichever account happened to be open, which
  was right only while looking at that account's own folder.

- **Ctrl and a number puts a label on a message.** Labels could be made, named,
  coloured, edited and deleted, and none of that ever reached a message: the
  table, the join table and the manager were all there and nothing put one on
  anything. It is the fastest thing there is for working through an inbox by
  ear, because it decides one thing about a message without opening it or
  leaving the row. The same key takes the label off again, `Ctrl+0` removes them
  all and says how many there were, and a number with no label on it says so
  rather than doing nothing. Reading a message in full says its labels, because
  a colour on a row is not something everybody can read.

  An account starts with the five Thunderbird uses, in the same order.
  Thunderbird puts these on the bare number keys; this uses Ctrl and the number,
  because a bare digit in a list is also a character and a list that jumps to
  what you type cannot tell "label this work" from somebody spelling their way
  to a message about invoice 4021.

  Labels travel as IMAP keywords, so one put on here is on the message in
  Thunderbird too, and one set on a phone arrives here on the next check. A
  label made by hand carries a keyword built from the letters of its name,
  since a keyword cannot hold a space. Renaming a label keeps the keyword it was
  already sent under, because changing it would leave the old one on every
  message on the server with nothing here recognising it. Sending is a change to
  the mailbox, so it waits until changes to mail are allowed.

- **Rules run on arriving mail.** They could be written, named, ordered and
  stored, and nothing had ever evaluated one: the engine, the editor and the
  table all existed and no arriving message was ever handed to them. Rules now
  run on mail as it comes down, once, on the messages that just arrived rather
  than on everything held, and the sync says how many were sorted. A rule that
  moves or deletes is left undone while changing mail is switched off, and the
  count of those is said too, because a rule that files invoices and does not is
  one somebody believes is working.

  Where a message matches several rules that disagree, the answer is settled
  before anything is written: the later rule wins, and deleting drops the rest
  rather than moving a message into a folder on its way to the trash.

  **Known limitation:** a rule that moves a message to a folder is named in the
  log and not carried out. Moving needs a write to the server, and doing half of
  it in the cache alone would show a message in a folder it is not in until the
  next sync put it back.

- **Signatures go on messages.** They could be written, named, marked as the
  default and stored, and none of that ever reached a message. One now goes on
  a new message, a reply and a forward, above the quoted original where you are
  already typing, so it can be read and changed before sending rather than
  appearing on the way out. A draft keeps whatever it was saved with rather than
  collecting another every time it is reopened.

- **A signature can be skipped when reading.** It is separated by the standard
  line of two dashes, which is what every other client uses to know where a
  message ends, and the reading surface marks it as "Signature" so you can jump
  to it or stop before it. Five lines of job title and disclaimer arrive on
  every message in a thread and are the same every time.

- **[How Wixen Mail compares](comparison.md)**, a written comparison against
  Thunderbird and Outlook: what they do that this does not, what this does that
  they do not, and what a 2025 accessibility study of Thunderbird found its
  blind testers ran into.

- **Notes and descriptions can be written in Markdown.** A note with a heading
  and a list in it used to come back as one flat run of text, so the shape
  somebody put there to make it findable was the first thing lost. Now a
  heading is read as "heading level 1" and a list item as "bullet" or
  "numbered item", which is the only way speech can say either. Nothing has to
  be turned on and nothing new has to be learned: what is stored is exactly
  what was typed, and text that is not Markdown is read exactly as it was
  written rather than reported as an error. The long field in each of the four
  item forms says so when you reach it.

- **A note can be heard in full.** Space on a note in the list read the same
  one-line preview the column already showed, so a note's contents could not be
  heard without opening it. It reads the whole note now, and the preview column
  shows the first line's words instead of its Markdown markers.

- **An event's description is read at all.** It holds the agenda and the
  dial-in number, and nothing read it out.

- **Files can be sent with a message.** The Attach File button has been on the
  compose window from the beginning with nothing behind it: no handler, no
  list, no column in the outbox queue, and no part in the message that went
  out. Alt+A or the button opens a file picker, the file is named and measured
  out loud when it goes on, Delete in the attachments list takes one off, and
  the file is read at the moment of sending, so what arrives is the version
  that existed when you pressed Send rather than when you picked it. The size
  is checked against what 25 MB becomes after encoding rather than what it is
  on disk, so a message too large to send says so before the upload instead of
  arriving back as a bounce an hour later.

- **The compose window can be worked from the keyboard.** Every control whose
  label underlines a letter now answers to Alt and that letter from inside the
  message as well as outside it: Alt+T for the recipients, Alt+O for the
  formatting menu, Alt+A to attach a file. Tab leaves the message body in both
  directions. Before this the message body kept every key it was given, so the
  only way out of it was Escape, which throws the message away.

- **The toolbar says what it is.** F8 goes to it, and each button announces its
  group, its name, the key that does the same thing without the toolbar, and
  where it sits in the group. Bold, Italic and Underline used to announce as
  "B", "I" and "U", which is what their labels are.

- **Dates read the same way everywhere, on the clock this machine keeps.** The
  message list read a date properly and every other list printed what was in
  the column, so a task due "2026-07-30" was read out as a run of digits. Two
  new choices, both taken from the machine unless you set them: whether the
  month is a word or a number, and whether the clock runs to twelve or
  twenty-four. All four date settings now have somewhere to be set; date style
  and date order have been in the settings file since they were written with
  nothing in the application that could change them.

- **Space reads the message rather than the row twice.** Shift+Space on the
  message list now reads the message itself, with its headings spoken as
  headings, without opening it. A row that is part of a conversation says how
  many messages it holds.

### Fixed

- **The folder list says which folder you are on.** Choosing which folders stay
  up to date gives you a list of check boxes, and each row said whether it was
  ticked but none of them said it was the one under the cursor. A screen reader
  working out the current item from what the rows report found no current item
  anywhere in the list, so on a server with sixty mailboxes there was no
  reliable way to know where you were, and pressing Space was a guess about
  which folder you had just changed. The row the cursor is on now reports
  itself as the current one, and it keeps its tick while doing it.

  The window also used to open with no row chosen at all, so there was nothing
  to be current. It now opens on the first folder. Nothing is ticked or
  unticked by that; it only decides where the cursor starts.

  Still to be confirmed with a screen reader. What is settled is that the rows
  answer with the right facts when Windows asks them. Whether NVDA then says
  "selected" on each arrow key is a separate question, because saying a row
  aloud depends on the list raising an event as well as on the row's answer.

- **A message is no longer announced as English whatever language it is in.**
  Every message you opened claimed to be English, so a German message on a
  German machine was read with English pronunciation rules from the first word
  to the last. That is worse than saying nothing, because a reader told nothing
  carries on in the voice you chose. The message now carries the language
  Windows is set to.

  Read the honest limit with it: nothing here knows what language a message was
  written in. No message arriving carries that information, and a sender's own
  marking on the whole document is removed when the message is made safe to
  display. So a message in a third language is still announced with your
  pronunciation rules. A marking a sender put on part of a message, a French
  quotation inside an English one, does survive and is used.

  When Windows will not say what language it is set to, the message says nothing
  about language rather than claiming English. That is a deliberate gap against
  WCAG 3.1.1, written up in `docs/accessibility.md`. Still to be confirmed with
  a screen reader: whether a reader acts on the language also depends on its
  automatic language switching being on and on a voice for that language being
  installed, and neither is ours to set.

- **A label with a real ampersand in it keeps the word after it.** Turning a
  visible label into a spoken name deleted every ampersand, but wxWidgets writes
  one real ampersand as two, so "Go Back && Edit" would have been announced as
  "Go Back  Edit": the word gone and a double space where a listener hears an
  odd pause. Nothing was routed through it in a way anybody would have heard
  yet, and the two buttons on the send confirmation now are, so this fixes it
  before it shipped. A label ending in more than one colon also had the whole
  run stripped; only the first is a visual convention, so only the first goes.

  Still to be confirmed with a screen reader: naming those two buttons is a call
  into Windows that cannot be read back, and the same kind of call once named
  sixteen controls that nobody ever heard.

- **A reminder going off now has its own sound instead of borrowing the one for
  new mail.** A reminder coming due played the exact tone that means a message
  arrived, so the sound sent people to an empty inbox while the reminder sat
  behind whatever they were working in. The two are different facts and they
  now sound different: the reminder tone is higher than anything else in the
  set and more than twice as long as either arrival sound.

  It also had no settings of its own. Because it borrowed the new mail sound,
  it followed the new mail settings, so anyone who had switched that sound off
  had silently switched reminders off too, with nothing named "reminder"
  anywhere to look at and work out why. Reminders are now their own event and
  are switched on and off on their own.

  The window itself has not changed. It still opens, still says what is due and
  when, and is still not switchable, because the window is the thing the sound
  is only announcing.

  **Still to be confirmed by listening:** the two tones are far apart as
  numbers. Whether an ear separates them, at the volume and on the speakers
  somebody actually has, and whether the reminder tone is comfortable rather
  than piercing, is a listening pass and nothing here has settled it.

- **Pressing Space twice on a message now tells you more than pressing it
  once.** The second press is meant to add the recipients, when it arrived,
  whether it is unread or flagged, and the labels on it. On any message whose
  text had not been downloaded yet, which is most of an unread inbox, it
  repeated the first press word for word. The key read as broken, or as a
  message with nothing more in it.

  Labels are the part that had nowhere else to go. There is no Labels column in
  the message list and no label line in the reading window, so a label you put
  on a message with Ctrl and a number was announced once and then could not be
  found again without leaving the row. The second press now says it, on a
  message that has been downloaded and on one that has not.

  Along with that, a message with something attached used to be read as "1
  attachments", whatever it carried: the wrong number and the wrong plural in
  the same three words. A message list row knows that something is attached and
  not what, so it now says "Has attachment", the same words the Attachment
  column uses. A reading that was given the parts counts them, and then the
  count is the real one.

  **Still to be confirmed with a screen reader:** all of this is about the words
  the application asks to have said. Whether NVDA speaks them, in a useful order
  and at a useful length, and whether the longer second press is welcome or is
  now too much to sit through, only a listening pass answers.

- **The date at the top of a message is written out instead of read as digits.**
  Opening a message, or hearing one read out with Space, started with the date
  exactly as it was stored: a run of digits, dashes, a T, colons and a time zone
  offset. It now follows the same date settings as every other date in the
  application, so it reads as "July 26, 2026 at 2:30 PM", or numerically, or on
  a twenty four hour clock, according to what you chose.

  This covers the header block above a single message. The headings inside a
  conversation still carry the stored date, and that is a separate fix.

  **Still to be confirmed with a screen reader.**

- **The first thing the application says now waits for a window that can carry
  it.** Wixen Mail opens by saying it is ready and saying where the cursor has
  landed. Both were said before the control that carries announcements existed,
  so both went out on a path that reports success and delivers nothing here. The
  opening lines of every session reached nobody.

  Anything said before there is somewhere to put it is now kept, and handed over
  as soon as there is. Up to eight lines are kept, oldest dropped first, so a
  start that never gets a window cannot grow without limit or release a wall of
  speech later.

  **Still to be confirmed with a screen reader:** this fixes lines being thrown
  away, and that much is certain. Whether NVDA speaks them at that point in
  startup, before the window is on screen, is a separate question that only a
  listening pass answers.

- **New mail is announced once instead of twice.** Every arrival was announced
  as "New mail, New mail", because the detail passed alongside the event
  repeated the event's own wording. It read as a fault in the program and took
  twice the braille cells. A detail that only repeats what the event already
  says is now dropped, wherever it comes from, so no other announcement can pick
  up the same stutter.

- **A mail check that worked no longer announces that the connection was lost.**
  Every ordinary check ended by dropping its connection, and that was announced
  as a loss, at a priority the pace limit does not apply to. Several times an
  hour, the one announcement that should mean "your mail has stopped arriving"
  was spent on mail arriving normally. A busy mailbox rang it over and over.

  The connection is now reported only when something has actually gone wrong,
  once rather than on every retry, and it says so once more when the connection
  comes back. A check that finishes normally says nothing about the connection,
  and the status line still reports the check itself.

  Two smaller changes come with it. A sync that fails now sets the connection to
  the failure and its reason, where it used to set the same value a successful
  check ends with, so the status field reads "Error" and what went wrong rather
  than "Disconnected". And the announcement no longer repeats that reason,
  because a failed send already says "Message not sent" and names the fault, and
  a failed sync already says "Error" and names it.

  **Still to be confirmed with a screen reader:** what is announced, and how
  often, has been checked in tests. How it sounds in use over a long session is
  a listening question.

- **A task list you delete on your phone now goes from here too.** A list that
  stopped coming back from Google Tasks or Microsoft To Do stayed on this
  computer forever, with its tasks in it and no way to reach any of it from your
  provider. A sync now removes it, along with the tasks the provider had put in
  it.

  Anything you made yourself is kept. A task you wrote here and have not sent
  yet moves to another of your lists rather than being deleted, and the sync
  says how many were moved so it is not a surprise. Where there is no other list
  to move it to, the list stays where it is and the sync says why. A list you
  made on this computer is never removed by a sync, because your provider never
  had it to stop sending.

  A sync that could not see the whole picture removes nothing at all: one list
  that could not be read, or a response that arrived incomplete, and the removal
  waits for the next sync. So a list you delete elsewhere can take one extra sync
  to go from here.

  **Known limitations:** a list holding more than ten thousand tasks still comes
  back cut short, and the tasks past that number are still missed on the way
  down. The sync now knows the read was cut short, so it removes nothing on the
  strength of it and says so as a problem, but it does not go back for the rest.
  None of this has run against a live Google or Microsoft account.

- **A very long Microsoft task list no longer loses everything past its first
  ten thousand tasks.** A read that stopped at that limit looked exactly like a
  list that had ended, so every task past the cap counted as one the provider no
  longer had and was deleted from this computer. A read that was cut short now
  says so, and nothing is removed on the strength of it.

- **A sync that cannot read its own database says so instead of reporting a
  clean run.** Where the changes or the deletions waiting to be sent could not
  be read, the failure looked identical to having nothing to send: the sync
  reported no problems and sent nothing, on every sync, for as long as the
  fault lasted. It is now counted as a problem on the status line.

- **An event brought down from Google or Outlook belongs to a calendar you can
  open.** Every one of them was filed under no calendar at all, so picking a
  calendar in the list showed nothing and the only place they turned up was the
  combined view. Each account now gets one calendar per service, called Google
  Calendar or Outlook Calendar, and the events go into it. Events already on
  this computer are filed the next time the sync sees them.

  Two things went in alongside it that nobody can see yet, and this is the
  honest reason they went in first. Sending a calendar change is not built. The
  code that would have sent one was written to send the whole event every time,
  with every field it had nothing to say about left blank, and both services
  read a blank field as an instruction rather than as silence. So the first
  change to any event would have removed every guest from it and turned a
  repeating series into a single appointment. A change now names only what it
  changes, and the request Google is sent asks it to merge rather than to
  replace.

  The time an event starts is also now written the way Outlook reads one: a
  clock face with the zone named separately, rather than a time carrying its own
  offset next to the name of a different zone, which would have put a noon
  meeting at five in the afternoon. An event whose stored time cannot be read at
  all is refused, and says which value was the problem, rather than being sent
  at a guessed hour.

  None of this has met a live server. What is proved is what goes out, not what
  either service does with it.

- **The dates and markers sent to Google and Outlook survive the trip.** The
  first calendar sync on an account asks for a window of six months back to a
  year forward, and it wrote the two ends of that window straight into the web
  address it asked. Both end in `+00:00`, and a plus sign in that position of a
  web address means a space, so both services were being asked about a time
  neither could read. The same fault applied to the markers each service hands
  back to say where the last sync finished: one containing an `&` split into two
  and asked a different question than the one intended. Every value this program
  puts into an address is now encoded first.

  This has never been seen happening, because no part of this program has yet
  run against a real Google or Outlook account. What is proved is that the
  program now sends the value it means to send. Whether each service is happy
  with it is still for the first live run to say.

- **Uninstalling takes the sign-in token of an account whose address was changed
  after it signed in.** Which token belongs to an account was worked out from
  the address the account has now, and the token was filed under the address it
  had when it signed in. So somebody who authorised a Google account and later
  corrected the address to their own domain was left with a token in the Windows
  credential store after the program was gone, and the uninstall reported that
  everything had been removed. Every account now gives up an entry for every
  provider the program can sign in to. Asking for one that was never there costs
  nothing, which is already how the master key is handled.

- **A category typed onto a Google or Outlook calendar event survives the next
  sync.** Neither category was being read back from the service, so every sync
  wrote a blank one over whatever had been typed here and the category was gone
  by the time anybody looked. The calendar an event was filed under went the same way. Both
  are kept from the copy already stored now, which is what the CalDAV sync
  already did. The people invited and the alerts set are still taken from the
  service, because unlike a CalDAV server both of these do send them.

  **Still not right:** an event arriving from either service is stored with no
  calendar at all the first time, because neither sync makes a calendar to file
  it under. Hiding a calendar cannot hide those events, and the per-calendar
  list never shows them.

- **An event with no alert is no longer given a fifteen minute one on its way
  to Outlook.** The alert was switched on for every event sent, with a lead
  time invented for any event that had none, so somebody who deliberately took
  the alert off would have been interrupted anyway. The Google side of the same
  pair already handed an event with no alert back to the calendar's own
  default; Outlook does that now too. An alert that could not be read is left to
  the calendar default rather than sent to Google as "this event never alerts",
  which switched the default off as well.

  **Where this lands:** nowhere yet. Nothing in the running program sends a
  calendar event to either service, so the calendar is read-only in practice.
  These are corrections to the shape a change would take on the day something
  sends one.

- **A contact sent to Google keeps its birthday, its website and its notes.**
  A contact created here and sent to the Google address book arrived with its
  name, its addresses, its phone numbers, its company and its nickname, and
  with those three fields silently missing. Nothing said so, at either end. A
  contact sent to Outlook already carried its birthday and its notes, so the
  two address books disagreed about what a contact is.

  A birthday recorded without a year goes as one, because Google is the one
  address book that can hold a birthday that way. A birthday somebody typed in
  words is still left out rather than sent, which is what the Outlook side does
  too: losing one field beats an address book refusing the whole contact.

  **Still not carried either way:** a postal address. Both address books hold
  one, this application reads neither, and a contact sent to either one goes
  without it.

  Superseded in an unreleased change, and only half of it. A postal address now
  goes out to both, and Outlook's two come back with a contact this computer
  reads for the first time, under Home and Work. Only that first time: the list
  stored here holds the addresses from every address book at once, so no sync
  writes it whole, and an address changed at Outlook afterwards does not come
  down. One held at Google is still not read at all: it arrives in the reply and
  nothing stores it, so a postal address typed in Gmail does not appear here.

- **One word for a finished task, and only the words that tell you something.**
  Pressing Space on a task said "done", pressing it again said "Completed", and
  the column beside it said "Done": one state with three names. It is "Done"
  everywhere now, and asking for the whole record on an unfinished one says
  "Not done" rather than saying nothing, since silence cannot be told apart
  from a reading that failed. A starred contact is read as "Favorite", the way
  the menu, the folder tree and the detail pane already spell it.

  Every task and reminder also said its priority, and every event its status,
  when nearly all of them hold the ordinary value. Working down thirty tasks
  meant hearing "Priority: normal" thirty times for nothing. A priority is now
  said when it is not the ordinary one and a status when an event is not
  confirmed, which is how "unread" and "flagged" already worked.

  The lists went on saying it after the reading stopped. Arrowing down a
  calendar said "confirmed" on every row and buried the cancelled one among
  them; arrowing down tasks or reminders said "normal" on nearly all of them,
  since that is what gets written when a provider has no notion of priority at
  all. Those columns now follow the same rule as the reading, and what is left
  says what it is: "High priority" rather than "high", "Cancelled" rather than
  "cancelled".

- **An event missing one of its times no longer trails off on the word "to".**
  The start and the end were joined before empty parts were dropped, so an
  event with no end read as "July 27, 2026 at 9:00 AM to" and stopped, which
  sounds like the reading was cut off. It now says the time it has. An event
  whose two times read the same says it once.

- **Jumping to the signature lands on it in a message that is not plain
  English.** The place to jump to was counted in bytes while the cursor counts
  characters, so every accented letter, smart quote or emoji above the
  separator pushed the landing place further past the signature: on a short
  sign-off it landed inside the last word, and on a longer one at the end of
  the message. Nothing said so, because the place was still labelled
  "Signature". Skipping five lines of job title and legal disclaimer is the
  reason that mark exists, and it worked only for messages written entirely in
  ASCII.

- **Two things said one after the other no longer cancel each other out.**
  Anything the program says that is not about a job in progress was handed to
  the screen reader marked "keep only the newest of these", with nothing
  attached to say which job it belonged to. Almost everything spoken carries
  no such label, so they all looked like versions of one another, and
  "Message moved to Archive" followed quickly by "Draft saved" could leave you
  hearing only the second. Announcements with nothing to group them are now
  each kept in their own right. A line that does say what it belongs to, such
  as a count climbing while a folder loads, still replaces its own earlier
  value, because that is what it is for.

  **Still to be confirmed with a screen reader.** How each of NVDA, Narrator
  and JAWS holds on to these is theirs to decide, so only a run with each can
  say what it sounds like now.

- **One skipped announcement is now counted as one.** When speech is running
  further behind than anyone will listen to, the queue drops the least
  important thing waiting and then says how many went. That sentence is heard
  rather than read, and at one it said "1 announcements skipped".

- **The folder chooser no longer tells a screen reader it is empty.** The list
  of folders to keep up to date had been given an object that reports each row
  as a check box, so the tick would be spoken as well as the folder's name.
  That object also answered "no rows in here" whenever it was asked how many
  rows the list held, and that answer was believed. So the change meant to make
  the ticks audible hid every folder instead, and a list with nothing in it is
  worse than a list read without its ticks.

  **Still to be confirmed with a screen reader.** The list now leaves counting
  its own rows to the control, which is what the same object beside it has
  always done for the same reason. Whether a screen reader then says "ticked"
  on each row is a thing only a screen reader run can answer.

  **Known limitation:** a row still reports that it can take focus and can be
  selected, and never that it currently is either. Answering for a row replaces
  the platform's own flags rather than adding to them, so the row the cursor is
  sitting on looks like any other. Fixing it means telling the rows which one is
  selected, which is a change to how the list is set up rather than a missing
  line.

- **Getting started is now written for the person using the program.** Pressing
  F1 anywhere outside mail opened a page of build instructions: install Rust,
  clone the repository, run the build. Somebody who is stuck in the middle of
  reading their mail was told to clone a repository, and by ear that is several
  minutes of listening before they find out the page cannot help them. It now
  covers adding an account, where the six areas are and the key that reaches
  each, how to move around a mailbox, and what is experimental. The build
  instructions are in the contributing page, where they were already.

  The full guide no longer claims to cover the calendar. It has no section about
  the calendar, contacts, tasks, notes or reminders, so pressing F1 in the
  calendar opened a document that never mentions it.

  **Still missing:** five of the six areas have no page of their own, so F1 in
  contacts, the calendar, reminders, tasks and notes lands on getting started.

- **After a check for new mail, the count of messages changed elsewhere is the
  real one.** It said how many messages the server had been asked about, not how
  many had actually changed, so a mailbox holding five hundred messages
  announced that five hundred had changed somewhere else after every single
  check, whether or not anything had. The number exists to tell you that
  something you read or starred on your phone has arrived, and it could not,
  because it said the same thing either way.

  Reading flags back from a server has never been run against a real account.

- **Opening settings no longer throws away a mark as read delay it does not
  offer.** A delay set by editing the settings file by hand, one second for
  instance, showed as "Immediately" in the settings window, and saving wrote
  that back. Somebody who had asked to wait ended up with no wait at all, which
  is the worst end to fail towards: arrowing down a list then marks every
  message you pass as read and empties the unread count. An unrecognised delay
  now shows as the ordinary default of two seconds.

- **The label you chose for a phone number or an address is the one that
  travels.** Every number sent to Google went as a mobile and every address went
  as an other, whatever you picked, and only the first of each was sent at all.
  A home landline came back from the address book labelled mobile and was read
  out that way. Labels now go out in the words you chose and come back in the
  same words, so a contact no longer carries "work", "home" and "Other" side by
  side for a screen reader to read out three different ways. A number recorded
  before there was anywhere to keep a label goes out with none rather than with
  a guessed one.

  Contacts from Microsoft still show Other on every email address. Microsoft
  sends no label with those, so that is what it gave rather than something being
  dropped. Corrected in an unreleased change for postal addresses: Outlook keeps
  two, a home one and a work one, and each now arrives here under that name.

  None of this has run against a real account.

- **A middle name stays a middle name.** A contact called Grace Brewster Murray
  Hopper was written to Google and to Outlook with the surname recorded as
  "Brewster Murray Hopper". The last word is now the family name and everything
  before it the given name. A family name that contains a space, such as van der
  Berg, still goes the other way. No rule gets both right from one line of text,
  and the whole name is sent as well, so the address book still has it.

  Superseded in an unreleased change: nothing splits a whole name any more. Both
  parts are kept as the address book gave them, or as you typed them in the two
  boxes the contact editor now has, so "van der Berg" no longer goes the other
  way either. The entry about a person's name surviving a trip to an address
  book and back, further up, is the whole of it.

- **A birthday with no year no longer says you were born in the year nothing.**
  Google returns most birthdays without a year, and that arrived here as the
  year 0000, showed in the birthday box, and was written into an exported card
  as a date in the year zero. A birthday with no year is now written with the
  year left out. It is spelled the way the card standard spells it, which is not
  a pleasant thing to hear read aloud; how it should sound is still open. A date
  that names no day at all is not stored. A birthday already stored as the year
  nothing keeps that value until the address book sends that contact again.

- **A birthday now goes with a contact sent to Outlook.** The field was on both
  sides and nothing ever filled it in, so a birthday was dropped on the way out
  and nothing said so. A birthday Outlook cannot read as a date, including one
  with no year and one typed in words, is left out rather than sent, because
  Outlook refuses the whole contact over it and losing one field beats losing
  the contact. A birthday coming the other way is now stored as the day it
  names rather than as a timestamp, which does mean a birthday sent with a time
  zone on it can land a day out. Unverified against a real account.

- **A contact with no name and a broken address is still called something.**
  There were two copies of the code that works out what to call somebody from
  their address, and one of them was broken: a contact arriving with no name and
  nothing but spaces before the @ was stored with a name of one blank space,
  which is a row in the contacts list that announces nothing when you land on
  it. There is now one copy and it is the one that works.

- **A contacts sync no longer downloads your whole address book every time.**
  The request never asked Google for the marker that says where the last sync
  finished, so no marker ever came back, nothing was ever stored, and every sync
  started from the beginning. It also meant contacts made here were offered to
  Google on every sync rather than once, and that the recovery for a marker gone
  stale could never run. That recovery now triggers only when Google says the
  marker is too old, rather than on any error at all, so a dropped connection no
  longer turns into a full download. Unverified against a real account: the
  first sync after this still reads everything, and only the second can be
  shorter.

- **A new contact is no longer sent with the fields the address book fills in
  itself.** Creating a contact sent an empty identifier and an empty version
  marker to Google, and an empty identifier and two empty change markers to
  Outlook. All of those belong to the provider, not to us. Nothing on this path
  has ever run against a real account, so whether it was being refused for that
  reason is unknown. It is at least no longer being sent.

- **Mail collected over POP is now checked for spam and phishing the way mail
  from other accounts is.** Every message from a POP account was recorded as
  ordinary whatever it carried, so the safety word that other accounts get on a
  row, and read out with it, never appeared. The check reads what the sending
  and receiving servers already decided and wrote into the message, so it costs
  nothing and sends no part of anybody's mail anywhere. Only what a server wrote
  above the message counts, so somebody quoting one of those lines in what they
  wrote cannot decide how their own message is marked.

  Nothing on this path has run against a live POP server.

  Superseded in an unreleased change: the second check, the one that reads the
  message text itself, runs on POP accounts too now. The entry about reading
  POP mail for signs of an impersonation, further up, is the whole of it.

- **Mail collected over POP now says how big each message is.** The size column
  was blank on every message from a POP account, and sorting by size heaped them
  all together at one end. Messages already downloaded keep their blank size,
  because mail collected this way is never fetched a second time; only mail
  collected from now on carries it.

- **A message collected over POP with a missing or unreadable date is now dated
  when it arrived.** An undated message sorted to the far end of the list, which
  by ear is a walk to the bottom of the mailbox rather than a glance. The trade
  is that an old message with a broken date now looks like it arrived today,
  which is the better end of it: a message somebody can find, dated
  approximately, beats one correctly dated to nothing where nobody looks.
  Messages already stored keep the date they have.

- **A mail server cannot take over a folder kept on this computer.** Sent mail,
  drafts, a trash you can recover from, and everything read over POP live in
  folders on this computer, in the same list as the folders on the server. The
  only thing separating them is the path, and nothing stopped a server handing
  back a mailbox named the same as one of them. When that happened the folder on
  this computer was treated as the server's: the next check for mail asked which
  of those messages the server still had, was told none of them, and deleted
  every one along with the copy it was holding. For a sent copy, a saved draft,
  or a mailbox read over POP, that was the only copy there was.

  A mailbox listed under the reserved name is now left out of the folder list,
  so it can neither be synced nor rename the folder it collided with. The names
  it uses cannot be typed and no ordinary server sends them, so nothing real is
  refused. This has not been seen happening to anybody, and like everything else
  here it has not run against a live server.

- **A task deletion refused on permission now says to sign in again.** Changing
  a task and deleting one are refused by the task service in the same way, and
  only one of them said so. An account signed in before this program could
  change tasks was told it had one problem after every sync, with nothing saying
  that signing in again was the fix. Both now say the same thing. Still untried
  against a live account.

- **The number of removed tasks is the number actually removed.** Google keeps
  sending word of a deleted task for a while after it goes, and every one of
  those was counted, so an account carrying old ones was told the same number
  had been removed on every sync, about tasks that went months ago. Only a task
  that was here and is now gone is counted. A removal the database refuses is
  now reported rather than counted as done.

- **A task moved to another list is not deleted and made again.** Microsoft To
  Do does not say when a task has been deleted, so what is gone has to be worked
  out from what came back. That was worked out one list at a time, so a task
  moved out of a list looked deleted until the list it moved to was read, and if
  that read failed the task disappeared from this computer until a later sync.
  It is now worked out once every list has been read, and a sync that could not
  read one of them removes nothing, because it cannot tell what is gone from
  what it did not see. That means a task you deleted on your phone can linger
  for one sync. None of this has run against a real account.

- **Syncing a calendar no longer wipes what you typed onto an event.** A
  category, the people you added, and the alert you set are kept on this
  computer and a calendar server does not carry any of them. Every sync wrote
  what the server sent and nothing else, so all three were erased each time the
  calendar refreshed. Reading a calendar was enough to do it, so it happened
  with changes to your accounts switched off. A sync now keeps them.

  An alert kept this way still does not go off. Nothing reads an alert set on
  an event yet, so this stops it being lost, it does not make it work.

- **An event on a calendar that is not in your own time zone had the wrong
  date.** A server sends the rules for when the clocks change alongside the
  event, and those rules carry a date of their own. The reader took the first
  date in the whole document, so a meeting next March was stored as the day the
  clocks changed in 1970. Every event on such a calendar was affected. Events
  already stored with the wrong date are corrected at the next sync.

- **An event a calendar sent without an end can be opened and changed.** It was
  stored with no end at all, and the editor refuses to save anything without an
  end date, so the event could be seen and never corrected. An event with no end
  now gets the end the calendar standard gives it: a whole-day event lasts one
  day, and an appointment with a start time ends when it starts. Events already
  stored without an end get one at the next sync.

- **Editing an event no longer throws away everything the editor did not ask
  about.** The editor asks about nine things and an event carries more: which
  calendar it is filed in, its category, how it repeats, who is coming. Saving
  an edit rebuilt the event out of those nine and dropped the rest. Worse, an
  event made on this computer could not be edited at all: the second save was
  refused and the message said a constraint failed, which told nobody anything.
  Both are fixed, and the editor now shows the notes and the alert the event
  already has instead of a blank and fifteen minutes.

- **Refreshing a subscribed calendar stops announcing changes that did not
  happen.** A feed that had not changed at all was reported as, for example,
  "200 created, 200 deleted", because every event was deleted and put back on
  every refresh. Read out, that is a minute of nothing. The refresh now says
  what actually changed, and it writes the new copy down before removing
  anything, so a feed that fails halfway through no longer leaves you with an
  empty calendar.

- **The zone a calendar names for an event is kept.** It was dropped, so a time
  was stored with nothing saying which zone it was in. Nothing says the zone out
  loud yet.

- **Syncing an address book no longer wipes the parts of a contact only this
  application holds.** A postal address, a photo saved with a contact, the card
  a contact was imported from, a relationship such as "sister", and any custom
  field you added are stored here and nowhere else. Neither Google nor Microsoft
  carries them, so every sync replaced them with nothing and counted it as an
  update. Reading contacts was enough to trigger it, so it happened even with
  changes to your accounts switched off, and where an account is signed in to
  both address books the two took turns doing it. A sync now writes only the
  fields the address book it came from actually holds, and leaves the rest of
  the contact alone. The website and any second phone number survive a Microsoft
  sync for the same reason.

  Two of those sentences are wrong. Outlook does carry a postal address, two of
  them, and since an unreleased change this program reads both and sends an
  address to both address books; the entry about a contact sent to Google
  keeping its birthday, further up, says the same thing. What keeps the one you
  typed is not that nobody holds it. The list stored here holds the addresses
  from every address book at once, so no single sync may write it whole.
  Outlook's two arrive with a contact this computer is meeting for the first
  time, under Home and Work, and an address changed at Outlook after that does
  not come down. Google's are still never read.

  The website does not survive a Microsoft sync either. Outlook holds one and
  this program reads it, so Outlook's copy is written over the one stored here,
  including where Outlook holds none and yours is emptied in its place. The
  exception is a contact you changed and have not sent yet, which a sync now
  keeps whole: the entry about a contact quietly replaced by your address
  book's older copy, further up, is that rule. The second phone number half is
  right, and for the reason given. Outlook reads only the first number, so the
  rest of the list is left alone.

  Contacts already emptied by earlier syncs are not brought back. There is
  nothing left to bring them back from.

- **The same person in two address books stops being taken back and forth.**
  Where an account is signed in to both Google and Microsoft and a person is in
  both, each sync took the stored contact off the other, rewrote which address
  book it came from, and reported it as a new contact. It never settled. A
  contact is now left alone once one address book holds it, and the sync says
  one contact was skipped.

  Superseded in an unreleased change: a contact now belongs to both address
  books at once and nothing is skipped.

- **A contact with no email address no longer destroys the one stored before
  it.** A contact with only a phone number is ordinary, and this stores contacts
  by their email address, so the second one saved took the first one's place
  while the sync counted both as new. The second and any later ones are now left
  alone instead. Only one contact without an email address can be stored per
  account, and that has not changed here: this stops the loss, it does not yet
  give you the other contacts. A contact that arrives with no email address and
  no full name is now shown by its first and last name where the address book
  gave them, rather than by a blank line in the list.

  The count of skipped contacts appears after a sync, but there is still no way
  to read which contacts they were. That gap is not fixed here.

  Superseded in an unreleased change: an account can now hold as many contacts
  without an email address as it has, and none of them are skipped. A contacts
  sync skips nothing at all now, so the count named above is gone from the
  sentence said after a sync. There is nothing left for it to count.

  None of this has run against a live Google or Microsoft account.

- **A list inside a list is read as what you typed.** In a note or an event
  description, an item holding a list of the other kind was announced with the
  inner list's kind: a bullet with numbered points under it was read as
  "numbered item", and a numbered step with bullets under it was read as
  "bullet". The kind of each item is now taken when the item starts rather than
  when it ends, which is the moment before the inner list exists.

- **The experimental warning in Settings reads as sentences.** The warning
  beside the two Allowed Changes boxes had runs of stray spaces in the middle of
  it, and the version a screen reader was given differed from the version on
  screen. Both came from the same sentence being typed twice. There is now one
  sentence, said once.

- **A calendar on your own server is asked for its dates in the shape a
  calendar server reads.** The request named the span of months to send back,
  six months behind and a year ahead, in an everyday date format carrying
  fractions of a second and a numeric time offset. Calendars are dated one
  particular way and a server that checks what it is asked refuses the whole
  request, so nothing came back, the calendar stayed as it was, and the only
  thing anybody was told was that the server had said no. The two dates are now
  written the way the calendar standard asks for.

  This is still untried against a real calendar server, so how forgiving each
  one was of the old request is unknown.

- **Mail now goes out from your address, not from the name you sign in with.**
  The account screen asks for an email address and a sign-in name in two
  separate boxes, and plenty of mail servers, corporate ones especially, want a
  bare name or a domain and a name to sign in. Every message sent was addressed
  as coming from the sign-in name. A reply to it went nowhere, and where the
  sign-in name is not an address at all the send failed with a complaint about
  an address nobody had typed in that box. The copy filed in Sent carried the
  same wrong sender, and a draft of the same message did not, because drafts
  were already filed under the account address.

- **A task deleted on an account signed in to both task services stays
  deleted.** Tasks go up and down in two passes, one for each service, and both
  run over the same account. The pass for one service read the other's waiting
  deletions as tasks made on this computer and never sent, so it threw away the
  record of them. The other pass then had nothing to send, and its own read put
  the task back. A task somebody deleted returned with nothing said, which is
  the failure that record exists to prevent.

  The same confusion also counted a task the other service was about to send as
  one kept on this computer, so the line after a sync said it was staying here
  moments before it went.

- **A contact whose only work detail is a department keeps it when it goes to
  Google.** The department was read and then thrown away. Nothing was sent
  unless the contact also had a company or a job title, so a person filed only
  under "Finance" arrived at the account with no work details at all, and the
  next sync brought that emptiness back down over the local copy.

- **A phishing site is now recognised under one more of its names.** Safe
  Browsing lists a site under its own address and under its parent domains, and
  the client has to ask about each of them. For an address with more than five
  parts, such as one buried under several levels of a free hosting service, the
  first of those parent forms was being skipped. A site listed under exactly
  that form was never matched, so no warning appeared. Every test covering this
  used a three part address, which cannot show the fault.

- **An imported contact keeps the labels on its phone numbers and addresses.**
  "Work", "Home", "Mobile" and the rest were only read when the file wrote the
  parameter name in capitals and listed exactly one label. RFC 6350 says the
  name is case insensitive and lets several labels be listed at once, and other
  clients do both, so a file exported from one of them came in with every
  number labelled "Other". The label is what tells two numbers apart when they
  are read out, and without it the only way to find out which is which is to
  ring one.

- **A contact taken from an address with nothing before the @ now has a name.**
  Auto-import reads addresses out of headers written by strangers, and a
  malformed one produced a contact whose name was an empty string: a row in the
  contact list that announces nothing at all when it is read out, with nothing
  about it to say what it is. It now says "Unknown", which is at least
  something to move past.

- **A new installation now checks spelling in the language the machine is set
  to.** Following the machine was done once and only half of it landed. A
  settings file written before the setting existed picked the machine's
  language up correctly, but a fresh installation wrote "en" for everybody, so
  anybody writing in another language had every word of it called a mistake
  until they found the setting. Finding a setting by hearing every word marked
  wrong is not finding it.

  The cause was that a settings file comes into being two ways and they were
  written out twice. They now use the same answer for every setting, so they
  cannot drift apart again.

- **An account can no longer print its own password.** The account record holds
  a password and two sign-in tokens, and the debug format a diagnostic line
  would print included all three. Nothing prints one today, so nothing has
  leaked; the format now leaves the secrets out, so the first line that does
  print an account is not a leak into a file people are asked to attach to bug
  reports.

- **The mail sync is tested.** What to fetch, what to forget, whose flags to
  ask about, whether a folder has been renumbered and what to do with what
  comes back are all decisions, and none of them had ever run in a test,
  because running them meant having a server. The transport now has a name of
  its own, so a sync can be run against a scripted server instead of a socket.
  Six tests cover the paths that used to be reachable only against a live
  account.

- **A new label or signature made in its manager is now actually saved.** Both
  were written by trying to update the row and creating it only if that failed.
  Updating a row that does not exist is not a failure in SQL, so the create
  never ran: every label and every signature made in a manager was accepted,
  reported as saved, and silently dropped. Found while wiring labels to the
  server, by a test that asked what the update had actually touched.

- **Next unread is `Ctrl+U`, and the shortcuts document says so.** It said
  `Ctrl+]` in two places, which nothing had ever bound, so anyone who read the
  document and pressed it heard nothing and had no way to tell that from a
  broken application. A test now compares the document against the code both
  ways, so a key can no longer be written down without being bound or bound
  without being written down.

- **Open Draft is `Ctrl+Shift+O`, and the shortcuts document now says so
  everywhere.** One table said `Ctrl+D`, which nothing has ever bound, so
  somebody who read that table and pressed it heard nothing. It is the third
  documented key found dead, and it got past the check written for the first
  two: that check asked whether the key appeared after a tab anywhere in the
  source, and `Ctrl+D` is the first six characters of the `Ctrl+Down` that
  moves to the next message in a conversation. The check now requires the key
  to end where it is found, so a key name sitting inside a longer one no longer
  passes for it.

- **The accessibility check now measures the names this application sets.** It
  scanned the UI Automation tree, and for an edit box or a button Windows puts
  its own UI Automation provider over the top of the accessible object this
  application supplies. So the check read the system's name for those controls
  and never the one set here, in either direction: it could report a name
  missing where one is present and spoken, and every accessible name in the
  application could have been deleted without it noticing. It now also walks the
  MSAA tree, which is what NVDA reads for those controls, and says which control
  and where when one has no name. Proved by taking a name out and watching the
  check report it.

- **Reminder alerts no longer pile up on top of each other.** With more than
  one reminder overdue, a second alert window opened over the first about a
  minute later, and a third over that, each covering the one being read. The
  alert window is modal and the clock that opens it keeps running inside it, so
  it went looking for the next reminder while somebody was still answering the
  first. They come one at a time now, and the next one appears when the one in
  front of it has been answered.

- **A list started on a blank message is a list.** Ctrl+Shift+L on an empty
  message announced "Bulleted list" and left plain text behind, so every Enter
  after it behaved like plain text, which is what a list that will not end
  looks like. Numbered lists appeared to work because the two are handled
  differently by the engine on an editor with nothing in it.

- **An automatic draft save no longer opens the spelling check.** Both were
  timers on the compose window, and a timer event reaches every handler on the
  window it belongs to, so every couple of minutes a modal dialog appeared in
  the middle of somebody typing.

- **A reply comes from the mailbox you read it in; a new message comes from
  your default account.** Both used to come from whichever mailbox happened to
  be open, which is right for a reply and wrong for everything else. Answering
  a work message from a personal address is a mistake you find out about after
  it has arrived, and browsing another mailbox is not a decision to write from
  it. Reopening a draft keeps the sender it already had.

- **Making a new event, reminder, task or note leaves you where you are.** Each
  of them moved you to its own module first, so starting a task while reading
  your mail put you in Tasks and left you to find your way back. The item is
  filed and the panel refreshed exactly as before; you are simply not carried
  off to watch it happen.

- **Events, tasks, notes, reminders, contacts and their folders can be made
  without a mail account.** They could not: the editors asked which account was
  active, found none, and stopped, which from the outside looked like the
  window failing to open. Anybody using a POP and SMTP account, which carries
  mail and nothing else, could not keep a single note, and neither could anybody
  who had not signed in yet. Whether a provider will carry an item is a question
  about syncing it, and it belongs at the point of saving; whether you may write
  one down at all was never in question. Items made this way are kept on this
  computer, in the same panels as everything else.

- **A command that refuses now says so out loud.** Everything a command had to
  tell you went to the status bar at the bottom of the window and nowhere else,
  which is not somewhere anybody working by ear goes. Pressing Ctrl+Shift+C
  without an account set up ran the command, refused because contacts are
  stored per account, wrote "Add an account first" into that bar, and made no
  sound. From the keyboard that is exactly what an unwired shortcut feels like,
  and it was reported as one. Refusals are now spoken, above the ordinary run of
  progress messages, and progress is spoken too but coalesced so a syncing
  mailbox does not talk over you.

- **The reading window can be left again.** It renders a "Back" button as the
  first thing on the page, and that button did nothing: the channel it sends
  its message on was never opened for that window, and neither was the script
  that makes Escape do the same. So the button was there, the key was in the
  guide, and both called into nothing. The only way out was Alt+F4. Both halves
  are now set up in one place used by the reading window and the preview alike,
  and a test ties the button's message to the code that listens for it so they
  cannot come apart again. The button also now says what it does on the surface
  you are on: the preview goes back to the message list, the window closes.

- **Setup finds a copy of Wixen Mail installed in the other place, and offers
  to remove it.** Installing for one person and installing for everybody are
  two separate installations that Windows does not tell about each other: they
  go in different folders and each puts a shortcut with the same name in a
  different Start Menu, which Windows then merges. Whichever came back first is
  what started, so an old version could keep launching while a current one sat
  installed and unused. It removes the other copy's program folder, its Start
  Menu entry and its Apps and Features listing, and **keeps your mail, accounts
  and settings**, which both copies share. It does not do this by running the
  other uninstaller, which would have erased them.

- **Uninstalling now says what it did, not only what it could not do.** The
  note it leaves in your temporary folder was written only when something was
  left behind, so finding no note meant either that everything went or that the
  step never ran, and there was no way to tell which. An uninstall that left the
  whole data folder in place wrote nothing at all.

  **Known limitation:** an uninstall has been seen to leave the data folder
  (mail cache, settings and logs) in `%LOCALAPPDATA%\wixen-mail`. Nothing was
  holding the files and no error was reported. Until that is understood, check
  that folder after uninstalling and delete it yourself if it is still there.
  Passwords and tokens live in the Windows credential store, not in that folder.

- **Removing the other copy also takes it out of Apps and Features.** It did
  not, so Windows kept offering to uninstall a program that was no longer on
  the disk. Setup is elevated while installing for everybody, and Windows then
  disagrees with itself about who the current user is: the folders belong to
  whoever started setup and the registry belongs to the elevated account, so
  the folder was found and removed while the listing was looked for in the
  wrong place. Setup now clears the listing as the person who started it, says
  so if any part of the cleanup did not work, and tidies a listing left behind
  with no program under it.

- **An uninstall no longer stops when the program is already gone.** Uninstall
  asks Wixen Mail to clear its own data folder and credentials first, because
  an uninstaller cannot reach the Windows credential store. If the executable
  had already been removed, that step failed and took the rest of the uninstall
  with it, leaving the folder, the shortcut and the uninstaller in place:
  neither installed nor removed, and still the first thing the Start Menu
  offered.

- **`F6` says what it moved to, including when there is nothing there.** It
  announced the pane's name and stopped, which is enough when the pane has
  something in it, because your screen reader reads the row focus lands on next.
  An empty pane has no such row, so the name was the whole announcement and
  arriving sounded exactly like the key doing nothing. It now says how many
  items are there, or that the pane is empty, and before any account is added it
  says so and tells you `Ctrl+A` adds one. Leaving the preview says the same
  thing, so arriving by any route sounds the same.

- **A message loading no longer drags you out of the folder tree.** The preview
  takes focus when it loads a document, without asking, so focus was pulled back
  to the message list every time one arrived. That fixed the preview and broke
  `F6`: anybody who had just moved to the folder tree was pulled out of it by
  the next message body to load. Focus now goes back where it was, and is left
  alone entirely when it is somewhere this application did not put it.

- **The preview shows the message the way the reading window does**, with the
  sender, date and subject as real headings above the body rather than the body
  on its own. Both surfaces are built by the same code, so they cannot drift
  apart about a conversation's shape.

- **Plain-text messages keep text that only looks like markup.** A body was
  taken as markup if it contained a `<` and a `>` anywhere, so a plain message
  saying "write to \<ada@example.com\>" was handed to the sanitiser, which
  deletes anything tag-shaped: the address disappeared from the middle of the
  sentence with nothing said. The body now carries whether it is text or markup
  all the way from the cache, so nothing guesses. This affected the reading
  window, the conversation page and the preview.

- **One message is no longer announced as a conversation.** Opening a single
  message read out "1 messages in this conversation." as the first line of the
  page.

- **Messages open formatted, keeping the sender's headings, links and tables.**
  Every message opened into a text box, which has none of those: no headings to
  press `H` for, no links your screen reader can list, a table flattened into
  lines. The structure was in the message and thrown away on the way to you, and
  the person most affected by that is the one who cannot see the layout it would
  otherwise stand in for.
  Plain text is one setting away, under Settings, Reading, "Open messages". It
  is worth having: it gives you a caret, so arrow keys move by character, word
  and line, text can be selected and copied, and your screen reader reports
  where you are continuously. Neither is right for everybody, so the setting
  says what each costs rather than which is recommended.
  **Attachments come with it.** The formatted window has its own attachment
  list, with the same keys as before: `F8` to reach it, `Enter` to read one
  here, `Ctrl+S` to save it. Without that, reading formatted would have quietly
  cost you your attachments, since the page shows message bodies and nothing
  else.

- **Opening a conversation gives you the page with headings, not a text box.**
  It was the other way round: the text reader was what `Enter` did, and the page
  with real headings and links was behind a button. A conversation is a shape,
  and a text box has no way to express one, so the reading surface that could
  show it was the one you had to know to ask for.
  The text version is still there, on an **As Plain Text** button, and is still
  what a single message opens into.

- **Closing a message from a conversation goes back to the conversation.** It
  went back to the mailbox, two levels up, so reading three messages from one
  thread meant finding the thread again three times. If you are working by ear,
  where you are is the only thing telling you where you are, and that threw it
  away every time. Escape from the conversation is what takes you back to the
  message list now.

### Known limitations

- **Dates are written in English on every machine.** The order of the day and
  month and the clock follow Windows, so the dates look as though they follow
  its language too. They do not. Month names are English, and so is wording like
  "2 days ago", which is most of what a mail list ever says. On a French machine
  that puts an English word in the middle of every date, read with French
  pronunciation rules, in every list in every module, and it sounds like the
  screen reader misbehaving rather than like this application speaking one
  language. Nothing is fixed here: the settings screen and
  `docs/accessibility.md` now say it out loud, because a limitation nobody is
  told about is worse than one they can plan around. Doing it properly means
  translating the month names, the relative wording and eventually every other
  string, which is a piece of work rather than a line change.

The limitations listed here have been closed further up this same release and
their notes have gone with them: a repeating event now shows on every day it
falls on, a change made here reaches a CalDAV server, the same change reaches
Google and Outlook, emptying a field empties it at the provider, a calendar can
be added by its address, and an event read aloud says its category.

### Added

- **POP accounts work.** They could not be created at all before: the account
  had IMAP and SMTP settings and nothing else, and the POP3 client behind them
  was a simulation that opened no connection, ignored the password it was given,
  and answered every command from three messages it had made up called "POP3
  Test Message 1" through 3. The roadmap ticked it as done.
  There is now a real client, and an account can say it reads mail with POP.
  Choose it when you add or edit an account, and give it a POP server, port and
  TLS answer of its own.
  **Mail is left on the server unless you say otherwise.** POP3 has one delete
  and it is permanent, so a client that clears the server as it downloads leaves
  you with one copy on one computer. If you do turn it off, you also say how
  many days to keep mail for, and nothing is ever removed that this computer did
  not download.

- **A POP account gets Inbox, Drafts, Outbox, Sent, Junk and Trash on this
  computer.** POP3 has no folders at all, so without these an account is a list
  of incoming mail with nowhere to put anything: no record of what you sent, no
  drafts, and a delete that can only be permanent. They are ordinary folders, so
  the tree lists them, search reads them, and move and copy offer them.

- **Every account has an Outbox you can open.** Mail waiting to go was a queue
  with a count beside it and no way to see, read or remove what was in it. It is
  a folder now, on IMAP accounts too, because a message that has not been sent
  is on no server by definition.
  Each row says what the message is doing: waiting to send, tried once, or tried
  four times and why. `Delete` on a row in the Outbox cancels the send, which is
  the one thing worth being able to do to a message that has not gone and could
  not be done at all before.
  The folder shows the queue itself rather than a copy of it, so it cannot show
  mail that has already gone or lose mail that has not.

- **Drafts go into your Drafts folder.** A draft was kept in a table of its own
  and nowhere else, so the Drafts folder in the tree never contained anything you
  had written, and a draft started here existed here and nowhere your other
  devices would look. Saving one now files it where that account keeps drafts: on
  the server for IMAP, in the local folder for POP.
  Saving again replaces the filed copy rather than adding another, so writing for
  ten minutes with automatic saving on leaves one draft rather than ten.

- **Deleting a message puts it in the Trash.** It used to mark the message deleted and clear it out where it stood. That means something different on every provider, and on Gmail it means whichever of three things a setting in Gmail's own web interface says, which Wixen Mail cannot see and never asked about. So a delete now moves the message to Trash, which behaves the same everywhere and can be undone by going and getting it.
  `Delete` is still one key and still asks nothing. It is a key you press twenty times going through a morning's mail, and a question in front of it is twenty questions; the message being recoverable from the Trash is what makes not asking safe.
  `Shift+Delete` removes the message from the server outright, with no copy anywhere. That does not ask either, on the same reasoning as everywhere else in Windows.
  What you hear is what happened. "Moved to Trash" when it moved, "Deleted" when it is really gone, and a longer sentence on the servers that can do neither cleanly, saying the message is in both places. Deleting from the Trash still deletes.

- **Move a message to another folder, or copy it there.** `Ctrl+Shift+V` moves, `Ctrl+Shift+Y` copies, and both are on the message menu key. A window opens with your folders in a tree: arrows move, Right opens the account, Enter chooses. The folder the message is already in is not offered, because choosing it would be a command that appears to do nothing.
  The window opens on the folder you filed into last, per account, so filing a run of messages into the same place is the shortcut and Enter rather than a walk through the tree each time. It is remembered between sessions.
  On servers that have the MOVE command it is one instruction, which is the only way a move is safe. Where it is missing, the copy is made first and the original is removed after, so a failure part way leaves the message in two places rather than none, and it says so.

- **Sent mail gets saved in your Sent folder.** Nothing was saving it. Sending and receiving are two separate services that know nothing about each other, so a message handed to the sending server left no trace anywhere you could look at it. Gmail files its own copy, so Gmail accounts happened to look right; every other account had no record of anything you had sent.
  Every account gets one, Gmail included, so the rule is the same wherever you are: what you sent is in that account's Sent folder. Gmail matches on the message's own identifier, so the copy that arrives is the one already there rather than a second one. The copy is marked read, so sending something does not raise your unread count. Blind copy recipients are not in the saved copy, which is the same rule that keeps blind copies blind on the way out.

- **The ticks in Folders to Keep Up to Date report themselves to a screen reader.** Windows draws those check boxes rather than using a control that has them, so the ticked state does not reach assistive technology on its own, and a window whose whole purpose is ticking things would have read as a list of folder names. Each row now reports itself as a check box with its state, the same way NVDA fixes it in its own settings. Still to be confirmed with a screen reader, which is the only thing that can confirm it.

- **You choose which folders are downloaded.** File, then Folders to Keep Up to Date, or the menu key on the folder tree. A ticked list, one row per folder, saying how many messages are in each. Space ticks the row you are on. The folder tree shows the folders that are kept up to date, so a folder you turn off leaves the tree rather than sitting there empty.
  **If you are upgrading, this may change what you see.** Folders you are not subscribed to on the server stop being downloaded and leave the tree, which on most accounts is nothing and on a shared or university server can be a lot. Nothing is deleted: tick a folder in that window and it comes back at the next check for mail.
  This matters most on Gmail, where All Mail holds a copy of every message in the account. Downloading it alongside the Inbox meant fetching everything twice and reading every message twice in the list. It is now off unless you ask for it, and the row says why. It matters on shared and university servers too, which list every mailbox the account can see, often hundreds.
  Your choice is written to the server as a subscription as well, so a folder you turn off here reads as unwanted in your phone's mail app. If the server will not record it, it says so, and your choice still holds here.

- **State you set on another device arrives.** Reading a message on your phone, starring one in webmail, or answering one from another machine now shows up here. It never did: a sync only asked about messages this copy had not seen, so anything already downloaded kept whatever it said the day it arrived.
  On servers with CONDSTORE, which includes Fastmail and current Dovecot, this is one instruction asking what has changed. Everywhere else, including Gmail and Microsoft 365, the flags of the messages you hold are read back in batches.

- **Wixen Mail says who it is when a server asks.** Courtesy nearly everywhere and a requirement on a few: NetEase servers refuse a client that will not identify itself, with an error about an unsafe login that sends you off to check a password that was fine.

- **Read receipts, and a default of telling nobody anything.** A sender can ask to be told when you open their message. Wixen Mail now notices when one has, says so when you open it, and sends nothing.
  What a receipt gives away is the point. It confirms your address is live, that a person is behind it, and roughly when you were at your desk. To somebody sending in bulk that is a working address worth selling. So the default is never, and it stays never until you change it in Settings, under Reading.
  Three choices: never, ask each time, or send whenever one is asked for. Each says what it costs rather than which is recommended.
  Two requests are refused whatever you choose. Anything in your junk folder, because a receipt to a spammer is the one reply that makes your address more valuable. And any request asking for the receipt to go to a different address from the one the message came from, which is the shape used to turn the feature into a beacon. Those become a question instead, because a mailing list can do it honestly.
  You are told what was asked even when nothing is sent, including on the default. That a stranger wanted to know when you read something is a fact about the message, and a client that quietly swallows it tells you nothing about who is doing it.
  Send Read Receipt, on the Message menu, sends one for the open message when it asked and you decided to.

### Fixed

- **A folder was downloaded again from scratch on every check for mail.** Saving the folder list replaced each folder's row rather than updating it, which gave the folder a new identity, threw away what it knew about the server's numbering, and took every message cached in it along with the old row. So each check for mail started over. It shows up as slowness rather than as an error, which is why it lasted.

- **Gmail's folders were treated as ordinary folders.** They are labels, and one message with three labels is the same message three times, under three different numbers. Wixen Mail now reads Gmail's own identifier for a message, so two rows for one message are recognisable as one. Search shows it once rather than repeating it for every label it carries, which is where the repetition was visible: a folder listing only ever shows one folder, and search reads them all.

- **Counting a folder no longer opens it.** Working out how many messages are in a folder took two instructions and changed which folder was open as a side effect of asking about a different one. It is one instruction now.

### Known limitations

- Conversations on Gmail are worked out from the message headers, the same way they are on every other provider, so a conversation here can be split differently from the same one in Gmail's web interface. Gmail does publish its own grouping, and the library Wixen Mail is built on reads it and provides no way to get at it, so this is not something Wixen Mail can currently fix at its end.
- The Sent copy does not list blind copy recipients. That is a consequence of how blind copies are kept blind on the way out, and it means the saved copy records what you wrote rather than everyone who received it.
- Which folders sync is set per account, not once for all of them. Turning one off takes it out of the folder tree straight away; turning one on brings its messages down at the next check for mail rather than immediately.

### Changed

- **Startup says "Wixen Mail is ready" instead of "Accessibility initialized".** The old line was wording from inside the program, said out loud to somebody who wants to know about their mail.
  Whether either line is heard at all is a separate question and still open. The window that carries announcements is registered after this one is queued, so the first two lines of a session go out on a path that reports success and is not delivered here. That needs a screen reader to settle and is not fixed by this change.

- **Version numbers stopped pretending to be an alpha programme.** This is 0.5.0. It was `0.1.0-alpha.25`, and the twenty-five before it were never tagged and never published: the counter was moving because each build was handed over as a file whose name carries the version, not because twenty-five releases happened. `0.x` already means unstable, so the suffix was saying it a second time and claiming a testing round that has not started. What is unproven is said in sentences, on the first-run screen, in Settings, at the end of `--help` and in the testing page, which is where somebody will actually read it. `-alpha.N` is now kept for staging a release that is about to go to testers.
  Builds between releases carry the commit they came from, as `0.5.0+g64c73dd`, in the file name, in Apps and Features, in `--version` and in the first line of the log. So a bug report can be matched to the code it came from even when several builds share a version.

### Added

- **A message can be copied to your tasks, your calendar or your notes.** On the message list, the menu key offers "Copy to a task", "Copy to the calendar" and "Copy to a note". The subject becomes the title and the message becomes the body: who sent it, when, and what it said, so the task still makes sense in a month when the mail has been archived.
  The whole message is kept rather than a link back to it, because a message can be deleted, moved by a filter or renumbered by the server, and then a link means nothing. A message with no subject gets a title saying so rather than an empty row, which in a list read aloud announces nothing at all.

- **The menu key works on every list and every sidebar.** The Applications key, or `Shift+F10`, on a message, a contact, an event, a reminder, a task, a note, a mail folder, a calendar, a task list, a note folder or a contact group. It offers what can be done with the thing you are on, which for somebody who cannot see a toolbar is the way to find that out without leaving the thing to go hunting through the menu bar.
  Only commands that work are on it. Mark a whole folder read and empty a folder are the obvious absences, and neither is implemented, so neither is offered. A menu line that does nothing is worse than one that is not there: it is a stop you land on, hear, and learn nothing from. Rename and move to another list were absent for the same reason when this was written, and both are on the menu now: rename on a contact group, which is the one container that has a rename written, and move on an event, a task or a note.
  The reminders sidebar has no menu, because it holds buckets rather than things you made, and there is nothing to do to one.

- **Making an event, task, reminder or note asks for what it actually is.** All four used to be a title in a box, and everything else was invented: an event an hour from now in no calendar, a task with no due date and no priority, a reminder with no time so it never went off, a note with an empty body in no folder. Every one of those columns was already in the database and nothing put anything in them.
  An event now asks for the calendar, all day, start and end date and time, location, repeat, an alert, busy or free, status and a description. A task asks for the list, due date, priority and notes. A reminder asks for the date, time, priority, repeat and notes. A note asks for the folder, whether to pin it, and the body.
  The field lists are not invented: they are what RFC 5545 and RFC 6350 define and what Google and Microsoft put on their own create forms. Where the two providers differ it says so, so priority on a task is marked as something Microsoft carries and Google does not.
  One form builds all four from a description of the fields, so the tab order, the labels and the way a missing field is reported are the same in each. A missing field is named rather than counted, because "some required fields are empty" makes somebody hunt through a form they cannot see.

- **A task list, note folder or contact group can be deleted.** Only calendars could before, so anything else you made by mistake you were stuck with. There is a Delete button beside the New button in each panel. It asks which one, and the question says what goes with it: "Delete the task list Shopping and the 12 tasks in it?" rather than "Are you sure?". It also says when the thing will come back at the next sync, because deleting it here does not delete it at your provider yet.

- **Wixen Mail asks, the first time you start it, what it is allowed to change.** Everything that writes is experimental: sending mail, deleting mail, and sending your changes to tasks, contacts and the calendar back to your provider. None of that has been run against a real account, so expect bugs. Reading your mail is the part that has been used.
  You get three choices, starting on the middle one: read only, tasks and contacts but not mail, or everything. Each says what it costs rather than which is recommended. There is a button to open [what to test and what is known to be broken](ALPHA_TESTING.md).
  Change it later under Settings, Allow Changes. The answer covers every account you have signed in.
  If you were already using Wixen Mail before this version, you will be asked once too. Writing had been switched on all along without anybody saying it was unproven.

- **`--read-only` and `--allow`, for one run.** `wixen-mail --read-only` changes nothing at any server that run, whatever the settings say. `wixen-mail --allow tasks` permits tasks, contacts and calendar but not mail. Neither can permit anything the settings forbid, so leaving one in a shortcut is safe.

- **`--help` and `--version` actually print something.** They could not before: Wixen Mail is a windowed program with no console, so anything printed went nowhere. They now write to wherever the program was started from, which covers typing at a prompt and sending the output to a file with `>`. A flag it does not recognise stops the program and says so, rather than starting up having quietly ignored what you typed.

### Added

- **Ten commands that were built and unreachable now have menu items.** Next unread (`Ctrl+U`) and previous unread (`Ctrl+Shift+U`), star or unstar (`Ctrl+Shift+S`), refresh folder (`F5`), get older messages (`Shift+F9`), and open a saved draft (`Ctrl+Shift+O`), all in the Message and File menus. The contact manager, message filters, signatures and tags are in Tools.
  Each was finished code with no way to reach it: no menu item, no button, no shortcut. The drafts one is the sharpest, because its own comment says drafts were being saved and then lost, and the dialog written to fix that was itself unreachable.
  A test now fails the build if two menu items claim the same shortcut. It caught one immediately: opening a draft had been given `Ctrl+Shift+D`, which is already New Reminder, so one of the two would have silently done nothing.

### Fixed

- **`F6` works.** It moves focus between the sidebar and the list, and says which one it arrived at. It had never worked: a handler was written for it, an id was allocated, a test guarded that id against collisions, three comments described it as working, the shortcut was in the documentation, and the page inside the message preview posted a message back to the host when somebody pressed it. No menu item and no accelerator ever raised the event, so the key did nothing, silently, which looks exactly like a shortcut that works and lands somewhere quiet.
  `Shift+F6` goes the other way. Both work in every module rather than only in Mail, where the old handler would have moved focus to the mail folder tree even in Tasks or Contacts, which is a control that is not on screen.
  A test now fails the build if any command has a handler and nothing that raises it. It found eleven more of the same shape on the day it was written.

- **The message preview cannot keep the keyboard.** Loading a page is when a WebView takes focus, and it does not ask. Focus is put back on the message list afterwards, so selecting a message can no longer leave you inside a browser that answers no keys.

- **Windows called it `wixen-mail` wherever it named the program itself.** That is the executable's name and it is right on the file, but it was also what Task Manager listed, what the elevation prompt called it, and what the file's properties showed, so a screen reader read out a hyphenated file name. It says Wixen Mail now, with a publisher and a copyright line where there were none.

- **Links in the guides said what file they went to rather than where they went.** "installing.md" tells you nothing, and somebody pulling up a list of links on the page got a list of file names. They say what is on the other side now. The addresses are unchanged.

- **Em dashes are gone from the documents and the source.** Eighty-seven of them. A test fails the build if one comes back, along with one for links named after files and one for the machine name appearing in a sentence, because a rule that lives only in a document is one somebody has to keep noticing.

- **The first-run screen reads out what each choice costs.** The three choices had a sentence beside each one saying what it means, and a screen reader never read any of them: the label was announced, the explanation was separate text nobody was pointed at. So somebody choosing what Wixen Mail may change heard "read my mail, change nothing" and had no way to know what the other two did without leaving the control and reading around the window. The explanation is now the button's description, which is read on focus.

- **The guides open as web pages instead of Markdown source.** The button on the first-run screen handed a `.md` file to Windows, which passes it to a text editor or to nothing at all. Read aloud, Markdown source is hash signs and square brackets, and the headings that make a long page navigable are punctuation. The documents are turned into HTML and opened in your browser, which has heading navigation, find and zoom already. Links between them work.

- **The accessibility scan had been scanning the same window over and over.** It takes a window name so each dialog gets looked at. The command line accepted `--scan-target settings` and then asked a reader that was looking for a different spelling, got back "no window was asked for", and started normally. Every dialog scan since was a second scan of the main window, reported as a pass. One spelling now, with a test tying the command line, `--help` and the workflow together. The first-run screen has been added to the list, so it is scanned too.

- **Uninstalling while Wixen Mail is open no longer takes your mail with it.** Uninstall deletes the data folder and clears your saved passwords. Doing that under an open copy removed the files it did not have open, left the ones it did, and let the copy still running write its settings back over the gap, leaving something that was neither installed nor removed. Setup and Uninstall now both stop and ask you to close it first. `--erase-all-data` typed by hand refuses for the same reason and says so, with its own exit code so a script can tell "close it and try again" apart from a real failure.
  Closing one of two open windows does not clear the mark, so the second one still protects itself.

- **A refusal goes to the error stream, where errors belong.** `wixen-mail --allow evrything > log.txt` put the complaint in the file and left the screen empty, so a typo in a shortcut looked like the program starting and then vanishing. The reason now stays on screen, and a successful run's output has nothing but its answer in it.

- **Five links between the documents went nowhere.** The worst was in the alpha testing page, which the first-run screen has a button to open: the section telling you where your mail is kept pointed at a file that has never existed. The accessibility page had three links to guides that were never written, and the user guide's contents listed a section that is not in it. All five now go somewhere real, and a test fails the build if a link between our own documents breaks again.

- **Compose windows are let go of when they close.** Every one stayed for the life of the session, and since the message body became a web view, each one held a browser. The preview before sending held a second, and the spell checker built a new dialog for every word it asked about, so checking a message with thirty misspellings in it left thirty behind. On a machine with modest memory, a working day of writing mail ended with the application unusable, which for this audience means losing your mail client in the middle of a job.

- **Wixen Mail now says when an account needs signing in again.** Sending task changes needs more permission than reading them, so an account you set up before this version keeps syncing downwards and has every change refused. That showed as "1 problem" on the status line, after every sync, forever, with nothing saying what would fix it. It now says "Sign in to this account again to send task changes", which is the only thing that does.

- **Dictation gets the spelling sound, which it never did.** The editor listened for typing and returned on everything else, so if you write by dictating or with Windows Voice Access, the sound at the end of a misspelled word was simply off, and nothing said so. It works now. Words finished by composition, which is how Japanese, Chinese and Korean are written, are picked up too.
  Pasting still does not set it off, on purpose: the sound is about the word you just finished, and a pasted block is checked by `F7` along with the rest of the message.

- **A message that cannot be read is no longer treated as an empty one.** Reading the message out of the editor can fail, and a failure looked exactly like an empty message. So a draft saving itself in the background could write nothing over the message it exists to protect, and Send could queue a message with no body in it, both without a word. Nothing is written now unless the message was actually read, and if Send or Save Draft cannot read it, the window comes back and says so with your message still in it.

- **Tab can leave a table going forwards, and a table cannot be grown without end.** Tab off the last cell adds a row, which is what every editor does, but it did so with no limit and it took the key every time. So holding Tab down built a table larger than the New Table dialog will make, and the only way out of a table was Shift+Tab back through every cell to the first one. Tab now stops taking the key once the table reaches the fifty-row limit, which gives a way out forwards and a bound at the same time.

- **A typed link whose address contains a bracket is no longer cut in half.** Markdown links are recognised when you type the closing bracket, so `[Mercury](https://en.wikipedia.org/wiki/Mercury_(planet))` was turned into a link at the bracket in the middle of the address. The shortened address is still a valid one, so nothing complained: you got a link to somewhere else and were told it worked. Wixen Mail now waits until the brackets balance.

- **Change All no longer goes back over words you chose to keep.** It rewrote every occurrence in the message, including ones you had already passed with Ignore, and said nothing about it. Ignore means leave this one, so Change All now applies from where you are forward, which is what Word does and for the same reason.

- **Spell checking knows what a word is.** It used to look for runs of letters, which got three things wrong. "3rd" was read as the word "rd", so the check announced a fragment that is not in your message and accepting a correction spliced it into the middle of a word that was already right. A sentence with no spaces in it, which is how Japanese, Chinese and Thai are written, was read as a single enormous word: F7 selected a whole paragraph, called it a misspelling, and Change would have replaced the lot. And "the end. The next" was reported as a repeated word, where the fix offered is to delete one of them, so taking it would have removed a word that was right.
  Word boundaries are a Unicode standard and Wixen Mail now uses it, so all three are right. Two words either side of a paragraph break are no longer treated as neighbours either.
  **Carrying on after a correction** used to be worked out by counting how many words a replacement was, which was wrong for a deletion and silently skipped the misspelling straight after it. The editor now reports where it left the cursor and the check carries on from there, so there is nothing left to get wrong.

- **Cc and Bcc are actually sent to.** They were collected, shown in the preview and kept in saved drafts, and then dropped on the way to the server: only the To addresses received the message, with no error and no warning. Reply All was the worst of it, announcing "Reply to all, 2 recipients" and then sending to one. Nothing had been sent for real yet, so no message has gone out this way, and Bcc addresses are hidden from other recipients as they should be.

- **Formatting commands put the keyboard back in the message.** They said they did and they did not. The editor is a web page inside the window, and the page can only move its own cursor, not the keyboard: run a command from the Format menu and the keyboard stayed on the Format button. Insert Table was the worst of it, announcing "In the first cell, Tab moves to the next cell" while Tab moved to the next button and nothing you typed reached the table. This is one of the things worth checking with your screen reader, because nothing in the tests can prove where the keyboard went.

- **The spelling check before sending no longer objects to your own formatting.** It was reading the message as markup rather than as words, so a message with a second line in it was checked as "Sam\<div\>See" and "tomorrow\</div", and you were asked "send anyway?" about a message with nothing wrong in it. Any second line, any use of the Format menu, any Markdown you typed and any quoted reply set it off. A confirmation that appears every time is one you learn to dismiss without reading, which costs you the time it mattered, so this was the check quietly defeating itself.

- **Replying to or forwarding a plain-text message no longer eats parts of it.** A message with no HTML part was being cleaned as though it were markup, and cleaning deletes anything shaped like a tag. A bare address or a bare URL in angle brackets is shaped like a tag, so "Please reply to \<ada@example.com\>." became "Please reply to ." with nothing said about it. Everything after an unclosed `<style>` or `<script>` went the same way. The mangled text was what got sent, not just what was shown.
  **Line breaks survived no better**, and that half needed no angle bracket to bite: every plain-text reply and forward arrived as one unbroken paragraph, in both halves of the message. If you are reading your own draft back with a screen reader, that is the harder one to catch, because it is read as continuous prose with nothing to say the breaks are gone.
  The cause was a guess. Whether a body is markup or text cannot be recovered from the string, because "if a < b and c > d" is prose that any test for angle brackets calls markup. The answer now travels with the body from the part of the message it came from, so nothing downstream has to guess again. The same guess has been taken out of the reading pane, where it could have done the same thing.

- **A message written with formatting would have arrived full of visible markup.** Between the editor changing and the multipart message being built, the whole message went into the plain text field, tags and all. Nothing shipped in that state.

- **The conversation reader claimed you could navigate it by heading.** You cannot: it is a plain text control, which has no headings for a screen reader to find. `Ctrl+Down` and `Ctrl+Up` do move between messages and always did. The documentation now says what is true, and As Headings, below, is where real headings live.

- **Creating an event, reminder, task or note now keeps it.** The dialog took a title, wrote a line to the log, announced "created" and threw the item away. Four of the six New commands looked like they worked and none of them stored anything. They store it now, in the right account, and the panel refreshes so you can see it.
- **Drafts save themselves while you write.** Settings has a box for how often, in minutes, from nought to ten, stepped with the arrow keys. Nought means never. It defaults to every two minutes, on, because the people it protects most are the ones who would never go looking for the setting. Every save updates the same draft rather than leaving a trail of near-identical ones, and the status line says when it happens. The checkbox that used to sit there said "auto-save drafts every 60 seconds", was ticked, was never read back, and nothing saved anything.
- **Saved drafts can be reopened.** File, Open Draft, or `Ctrl+Shift+O`, lists what you have saved by subject, recipient and date, and opens the one you pick with its fields filled. Saving it again updates that draft rather than leaving a second copy beside it. Until now a draft went into the database and was never seen again, which is worse than not saving it, because it looks like it worked.
- **Save Draft saves the draft.** It answered "Draft saving is not implemented" while the button sat there and the storage waited unused. A draft with no recipient is kept too, because a draft is unfinished by definition and refusing one for having no address yet loses exactly the work somebody was trying to protect.
- **Calendars, task lists, note folders and contact groups can be created**, which they could not before: the controls opened the same discarding dialog the items used, so a name was typed, logged and lost while the application said it had been created. A container is filed wherever the things it holds are filed, so a calendar and its events can never end up in different accounts.

### Removed

- **Two HTML rendering paths nothing used**, one of them for egui, a user interface framework this application does not depend on and has not since the move to wxWidgets. The other was called `render_for_accessibility`, which is the only accessible thing about it: nothing called it either, and the real reading path is the one behind the reader window. Both were invisible until visibility was narrowed, because Rust never reports a public item as unused.


- **A table of access and refresh tokens that nothing ever read.** The tokens actually in use are in the Windows credential store. This was a second copy, encrypted at rest and then abandoned: no code path would ever have rotated it, expired it or deleted it, and it travelled with the database whenever somebody copied their profile. It is dropped from existing databases on the next start, which is the one case where a schema change is allowed to take something away.
- **A dialog for pasting OAuth codes and client secrets by hand**, which nothing opened. A comment said it was kept for advanced manual token management, and it had been kept for nobody. Sign-in happens in the browser.
- **Two modules for storing files that nothing stored files with.** They will come back as one thing, wired, when attachments can be downloaded.

- **A second copy of the application.** Two executables were built from near-identical files, and the installer shipped one while the release archive shipped the other, so a setup file and a portable download were not the same program. There is one now, `wixen-mail.exe`, and `cargo run` starts it without naming a binary.
- **A database nothing ever read.** A second SQLite file was created in the roaming profile on every start by a module no code called. Roaming a database is a way to corrupt it, and this one held nothing worth corrupting.

### Added

- **A conversation can be read as real headings.** The conversation dialog has an As Headings button, which opens the whole thread as a page in its own window, where every message is a heading at the level its reply sits at and `H` moves between them. The text reader stays the default: it is focusable, arrow-navigable and searchable, and it has no headings, which is the one thing it cannot do. The window shows the conversation and nothing else, so there is nowhere for a browser control to trap anybody: closing it is the way out.
- **A page saying what syncs from which kind of account**, in [Setting up your provider](PROVIDER_SETUP.md). One account does everything it can and there is no second account to set up, but what "everything" means depends on the provider, and that was not written down anywhere.

- **The message body is a different control**, and the reason is your screen reader rather than the formatting. The old one was drawn by wxWidgets itself, which means it could never mark a misspelling, never let a heading report itself, and every announcement had to be made by hand and be slightly wrong forever. The new one asks the browser engine your system already has, and gets all of that natively: misspellings are marked and announced by NVDA, VoiceOver or Orca themselves rather than by us.
  The keys that have to leave the editor are bound inside it and passed out: `Escape` closes, `Ctrl+Enter` sends, `Ctrl+S` saves a draft. Everything else belongs to the editor, which is what you want while writing. Every formatting command says what it did and puts the keyboard back in the message, because a toolbar button takes focus and the next thing you type should not go to the button. Getting that right needs two separate things, one in the page and one in the window, and the first version only did the one in the page. See Fixed, above.
  **A reply quotes a stranger's message**, and this puts it in a live document, so every quoted body is cleaned before it reaches the editor and again on the way out. The old control rendered nothing, so it could afford less care. This cannot.

- **Headings, lists and links can be put in a message.** Format, on the composition window, or the keys beside each item on that menu: `Ctrl+Alt+1` through `Ctrl+Alt+3` for headings, `Ctrl+Alt+0` to go back to ordinary text, `Ctrl+Shift+L` and `Ctrl+Shift+O` for bulleted and numbered lists, `Ctrl+Shift+Q` to quote, `Ctrl+Space` to strip formatting. Each says what it applied.
  These are for whoever receives the message rather than for you. A heading is what they navigate by, and a long message without any can only be read from the top. It is the thing this application spends its time wishing other people's mail had, so it would be poor manners to send mail without it.
  The menu exists so none of it has to be memorised: tab to Format, press Enter, and arrow through the same thirteen commands the keys apply. The keys and the menu are generated from one list, so a menu item cannot promise a key that nothing binds.
  On layouts where AltGr and a digit types a character, the heading keys type that character instead: taking it away to save a trip to the menu would be the wrong trade. The Format menu still applies headings there.

- **F7 walks the spelling of a message.** Or the Spelling button, which is on the toolbar and reachable by Tab, because a key nobody can discover is a key nobody has. Each word is selected in the message before you are asked about it, so it is read where it sits rather than quoted out of its sentence. Change, Change All, Ignore, Ignore All, Add to Dictionary, Close. Focus starts in the field holding the first suggestion, so Enter on a word you agree about is the whole interaction, and you can type your own correction instead.
  This is the half no engine can do for us. Browsers mark misspellings but none of them exposes the list, so without this the only way to find three wrong words in a long message is to read the whole message. Add to Dictionary teaches Windows itself, so the word is known in Word and Edge too, and a word that could not be added says so rather than quietly coming back.
  A repeated word offers Delete rather than Change, and deleting takes the space in front of it so the sentence is not left with a gap. "alot" corrected to "a lot" is handled properly: a correction that adds a word moves everything after it, and the check carries on from the right place instead of skipping the next mistake.

- **Misspelled words are marked as you write**, by the browser engine, which means your screen reader announces them as you move over a marked word rather than us trying to describe them. There is also a short sound at the end of a word that is wrong, for the moment rather than the word: it is silent until earcons are switched on under Feedback. One setting governs both, under Settings then Language, because two would let somebody end up with the sound on and the marking off and no idea why.

- **Markdown can be typed straight into a message.** `# ` for a heading, `- ` for a bullet, `1. ` for a number, `> ` for a quote, `**words**` for bold, `` `words` `` for code, `[the words](https://example.com)` for a link. Type the marker and keep writing. Each one says what it made, and `Ctrl+Z` puts the characters back if you meant them literally.
  This is the point of the whole editor rather than a shortcut for the menu. Headings and lists are what makes a message navigable for whoever gets it, and every other way of adding them asks you to stop writing, leave the sentence and find a menu. Typing `## ` does not. It is also how a great many people who cannot see a toolbar already write.
  A marker only counts when it is the whole line so far, so a sentence ending in a hyphen stays a sentence. An address the application will not carry, a `javascript:` one for instance, leaves the words alone and says so instead of quietly making them plain text, because otherwise you send a message believing there is a link in it.

- **Tables, with real column headers.** Format, then Insert Table. It asks for rows and columns and whether the first row is headers, and leaves that on, because `scope="col"` is what lets the person receiving the message hear "Total, column 3" as they move across instead of a wall of numbers. A table laid out with spaces, which is what people do without this, gives them nothing.
  `Tab` moves to the next cell and `Shift+Tab` to the previous one. `Tab` in the last cell adds a row and says so. `Shift+Tab` in the first cell leaves the message the way `Tab` always did, so there is no way to get stuck in a table. Up to 10 columns and 50 rows: past that a table cannot be held in your head while you move across it, and it says so rather than building one nobody can read.

- **A formatted message now goes out as formatted mail.** It is sent as both halves of a `multipart/alternative`: the formatting for programs that show it, and a plain text version taken from the editor itself for programs and screen readers that prefer text. Plain text mail is still sent as plain text alone.

- **Things can be deleted.** Contacts, events, reminders, tasks and notes could all be created and none of them could be removed. `Delete` now works on the row you are on, in every module, and asks first with the row named in the question: "Delete \"File the tax return\"? This cannot be undone." It is the same key that deletes a message in Mail, acting on whatever is in front of you, the way `Ctrl+N` makes whatever the area you are in is for.
- **Tasks and reminders can be marked done**, with `Ctrl+Shift+K`, and **notes can be pinned**, with `Ctrl+Shift+P`. Both say which way they went, because a toggle you cannot see is one you have to be told about. Both are greyed out where they mean nothing, so a screen reader says "unavailable" rather than leaving you to press a key that does nothing.

- **Spelling is checked when you send a message**, and it is Windows doing the checking. That matters more than it sounds: Windows' spell checker knows the words you have already added in Windows Settings, so the surnames and technical terms you have taught your computer once are known here too, rather than having to be taught again one application at a time. Words you add go back to that same shared dictionary, so Word and Edge learn them as well. If anything looks misspelled the words are named in the question rather than counted, so you can decide without opening anything, and a message with nothing wrong is never interrupted. A repeated word is called a repeated word rather than a misspelling, because "the" is spelled correctly. Where Windows has no dictionary for your language, the built-in list is used instead and Settings says which you have.
- **The spelling language list is the one your computer really has.** It used to offer the same six languages on every machine, so choosing one your computer could not check set a value that changed nothing, and the only way to find out was to write in that language and have every word called a mistake. It now asks Windows what it can check and lists that, named the way Windows names each one, in its own language. Anything it cannot check says so on the row rather than leaving you to discover it. The setting is also labelled "Check spelling in" instead of "Interface language", which it never was: nothing in the interface is translated.
- **Checking before sending can be switched off**, in Settings then Language. On by default. Somebody who does not want to be asked can say so once, which is different from being asked every time and dismissing it, because that teaches you to dismiss the one that mattered.
- **Two settings that did nothing are gone.** "Enable spell checking in compose editor" and "Show suggestions as you type" were both ticked, neither was ever read back, and nothing checked anything. The Spell Check section now says which checker your machine actually has.

- **Tasks come down from Google Tasks and Microsoft To Do.** Tools then Sync Tasks. Your lists and the tasks in them appear in the Tasks module, with due dates, completion and, on Microsoft, priority. A task you make on a Gmail or Outlook account is now filed under that account rather than on this computer, so it sits in the same list the sync fills.
  **It goes both ways.** A task you make, tick off or delete here reaches your provider on the next sync, so it turns up on your phone and the web page. Google's deletions are honoured in the other direction, so a task ticked off on your phone does not reappear here. A task the provider has not touched since the last sync is left alone rather than rewritten, so the count after a sync says what actually changed instead of how many tasks you have.
  **When the same task changed in both places, your provider's version wins**, and you are told: the line after a sync says how many of your changes were replaced by the server. Its copy is what your phone and the web page already agree on, so it is the one you most likely looked at last. A change lost that way can be made again; a change made on your phone and overwritten by a stale copy from this computer cannot, because nobody would find out. A change that cannot be sent keeps waiting and is tried again next time rather than being dropped.
  A task you make goes into your account's first list, which is the one your provider treats as the default: "My Tasks" on Google Tasks, "Tasks" on Microsoft To Do. There was no list picker when this was written, and there is one now: the New Task form asks which list, and the entry about the four item forms, further up, is where that landed.
  One case still stays here: a task made on an account that has never synced has no provider list to go in, so it goes into a local list and the sync says "1 kept on this computer" rather than trying to send it to a list your provider has never heard of on every sync forever. Sync first and it will not happen.
  **Sending tasks up needs more permission than reading them**, so an account signed in before this version will keep syncing downwards and hold your changes until you sign in again. Open the account, switch the browser sign-in off and back on, and approve the permissions. [Setting up your provider](PROVIDER_SETUP.md) says which permission and why.
  **Notes and reminders stay on this computer.** Google Keep's API is only available to Workspace accounts, so a consumer Gmail account cannot use it. OneNote could carry notes and has not been written, because a OneNote page is an HTML document inside a section inside a notebook rather than a title and a body, and that mapping is a decision rather than an afternoon. A standalone reminder is not a thing either provider has: Outlook makes a reminder a property of an event or a task, and Google folded Reminders into Tasks in 2023.
- **Attachments can be saved.** The reader lists them below the message, so they are the next thing after it in the tab order, and each row reads as the name, the kind of file in plain words, and the size: "Report.pdf, PDF document, 240 KB". `F8` jumps to the list and back, `Ctrl+S` or `Enter` saves the row you are on. Nothing is kept on your computer in advance: the message is downloaded again when you save, which is what keeps the cache small. Until now the Has Attachment column promised something the application could not do.
- **Links can be checked against Google's lists of known phishing and malware sites**, off by default, in Settings then Advanced. Google's lists are downloaded to your computer and the comparison happens on your computer, so for ordinary mail nothing is sent to Google at all: not the link, not a fingerprint of it, not a note that a message was read. Only when a link matches one of the downloaded entries do four bytes go, and four bytes match millions of possible addresses. Google never receives the link, the sender, the subject or any part of a message. The other way of using Safe Browsing posts the URL itself, and that one is not used and will not be. Needs a Google API key, and does nothing without one. [What Wixen Mail sends, and where](privacy.md) has the whole of it.
- **A page saying what the application sends and where**, [What Wixen Mail sends, and where](privacy.md), because "we respect your privacy" is a sentence anybody can write.
- **PDF attachments can be read in the reader.** `Enter` on a PDF row, or `Ctrl+O`, opens it as another tab, so everything that works on a message works on it: arrow keys, find, selection, and `Ctrl+Down` to move between pages and headings. Each page starts with a line naming it. Reading uses [a PDF reader written in Rust](https://crates.io/crates/pdfpurr), so there is no PDF viewer to install and nothing is handed to another application.
- **A PDF says where its structure came from, before a word of the document.** Tagged, tagged with gaps, or no structure at all with the headings guessed from the size and position of the text. And when a PDF has no text, it says the thing every other application leaves you to work out from the silence: this is a scan, a picture of a page rather than words, and nothing here can read it aloud. Ask the sender for a real one.
- **A file Windows would run is called a program.** An attachment ending in `.exe`, `.msi`, `.scr`, `.bat`, `.ps1`, `.lnk` or anything else Windows executes reads as "program" rather than as whatever the message claimed it was, and the announcement when the message opens says so before you have reached the list. The type in a message is written by whoever sent it, so it is a claim rather than a fact, and on a malicious attachment the claim is usually the harmless one. The extension is a claim too, but it is the one Windows acts on.

- **Wixen has a logo.** A fox's head with a band across its eyes, in burnt orange, ears up and forward. It belongs to the family rather than to this application, so Wixen Chat and whatever follows use the same mark, and each application keeps its own icon built the same way: a coloured field, a cream figure, and the detail in ink. The three colours are held to WCAG contrast floors by tests, the type decision is that Wixen ships no typeface and honours the system font at the size you chose, and every asset comes with the alt text it was designed against. [the Wixen family mark](brand.md) has the reasoning, including why the ears are the size they are.

- **Older mail can be fetched.** A sync brings down the newest five hundred messages in a folder and used to stop there, with no way back and nothing saying there was more. `Shift+F9`, or File then Get Older Messages, brings down the next page. The status line now says how many are downloaded out of how many the folder holds, so "500 of 40,000" reads as the incomplete answer it is rather than as a complete one, and it names the key while there is still more to get.
- **Wixen Mail has an icon.** The executable had none, so Windows drew the generic one in the taskbar, in Alt+Tab, on the shortcut and in Apps and Features. It is an envelope whose flap is a W, which is the one thing that makes it this application's envelope, and it still reads as a flap at sixteen pixels.
- **The theme setting paints three parts of the window.** It was stored, read back into the Settings dialog, and applied to nothing at all. Picking Light or Dark now colours the folder list, the message list and the side panel. Everything else follows Windows, and the Settings dialog says so under the Theme setting, because a setting that changes less than you expect is one you read as broken. Three things are worth knowing before you try it:
  - A change takes effect the next time Wixen Mail starts, not while the dialog is open.
  - Default means light for now. It is meant to follow Windows, and it cannot until Wixen Mail asks Windows for its dark mode, which is a change that recolours every control at once and needs somebody to look at it first.
  - Windows high contrast overrides all of it and Wixen Mail paints nothing of its own, because somebody running high contrast chose their colours, usually because nothing else is legible, and an application that paints over that has removed the reason they set it.

  Each colour is now set together with the text colour tested against it, which is a change made while this was being written rather than a bug anybody met: painting the folder list dark and leaving its text the near-black Windows had given it would have put the folder names at 1.27 to 1 against their own background, which is a blank panel. Nothing was released in that state.

  **Still to be confirmed with a screen reader and on a screen.** Nobody has looked at the dark theme at real size and real magnification. The message list keeps its column header in the Windows colours, because that header is a control of its own, and whether the selection highlight, the expand arrows and the focus rectangle in the folder list still read against a dark background is the sort of thing only looking answers. The note in Settings is written where a screen reader can find it, and whether it is actually read out when you land on the Theme setting is not something the tests here can tell you.
- **`Ctrl+N` makes whatever the area you are in is for**: a message in Mail, a contact in Contacts, an event in Calendar, and the same in Reminders, Tasks and Notes. It used to be New Message everywhere, which was the wrong answer in five of the six.
- **Six keys that make one particular thing from anywhere**, so you never have to switch module first: `Ctrl+Shift+M` message, `Ctrl+Shift+C` contact, `Ctrl+Shift+E` event, `Ctrl+Shift+D` reminder, `Ctrl+Shift+T` task, `Ctrl+Shift+N` note. Reminder takes `D` for due, because `Ctrl+Shift+R` is Reply All here as it is in every other mail client, and that is not worth making anybody relearn. Three keys moved to make room: Mute Message Reading to `Ctrl+M`, Next and Previous Unread to `Ctrl+U` and `Ctrl+Shift+U`. Those two were written down as `Ctrl+]` and `Ctrl+[` for a while and nothing ever bound either; the entry about the shortcuts document, further up, is where that was put right.
- **New items go somewhere, and Wixen Mail says where.** Your default account when it can hold that kind of thing, and this computer when it cannot. A plain mail account holds mail and nothing else, so a contact made while one is the default is kept here rather than filed into an account that will never sync it. Your first account becomes the default on its own; change it with Set as Default in the accounts dialog. Items kept on this computer show up in the panels alongside your account's own.

- **Wixen Mail tells you when a message is spam or a phishing attempt.** The verdict comes from the filter your provider already ran: SpamAssassin's headers, Microsoft's confidence levels, what the receiving server made of the sender's anti-forgery records, and, for Gmail, the junk folder, which is the whole of what Gmail tells a mail application. Nothing is sent anywhere to work this out. DMARC failing counts as impersonation, because it is the sender's own published records saying the message did not come from them; SPF failing on its own does not, because forwarding and mailing lists break it routinely and a warning that fires on half an inbox is one people learn to ignore.
- **A warning above the message, which stays there.** An announcement can be missed and cannot be replayed. The reader now puts the warning in a read-only text box above the message, first in the tab order so you meet it on the way in, and `F7` moves between it and the text in both directions. You can read it as many times as you like, arrow through it, and copy it. A message with nothing wrong with it has no bar at all, so ordinary mail has nothing extra to tab past.
- **The warning is announced when the message opens**, as an ordinary feedback event, so it can be switched off, moved to a different channel, or made a tone rather than speech. Turning the announcement off leaves the bar in place. It is ranked with the failures rather than the notices, because an announcement that queues behind a sync message arrives after you have started reading.
- **A Safety column in the message list**, off by default and switchable in the column picker like every other column. It reads "Spam", "Phishing", "Suspicious" or nothing, and it sorts, so everything a filter distrusts groups at one end of a folder.
- **A setup file that installs properly.** It asks whether to install for you alone or for everybody, and installing for yourself needs no administrator rights, so there is no elevation prompt and no switch to the secure desktop in the middle of setup. It carries a fixed identity, so an upgrade replaces the previous install rather than leaving two entries in Apps and Features, and it closes a running copy instead of failing on a locked file. The version comes from the source rather than from a number typed into the setup script, which said `0.1.0-beta.1` while the application said `0.1.0-alpha.14`.
- **Uninstalling removes everything, including the parts an uninstaller cannot reach.** Your saved passwords and sign-in tokens are in the Windows credential store, which no uninstaller can see into, so they would have outlived the application that put them there. Wixen Mail now clears them itself before the files go, running as you rather than as the administrator doing the uninstalling, which is the difference between clearing your credentials and clearing somebody else's. Anything that could not be removed is written to `wixen-mail-uninstall.log` in your temporary folder, because believing your mail is gone when it is not is worse than being told.
- **[How to install, back up and uninstall](installing.md)**, including the silent install switches and how to point Wixen Mail at a different folder for its data.
- **Wixen Mail installs on Windows on ARM**, under the x64 emulation those machines run. The setup file asked for an x64 processor, which a Surface is not, so it would have refused for a reason nobody could do anything about.
- **Every build carries a version Windows can tell apart.** The four-number version Windows shows in a file's properties has no room for "alpha.15", and dropping it gave every prerelease of `0.1.0` the same `0.1.0.0`. The stage and the number are encoded into it instead, so `0.1.0-alpha.15` reads as `0.1.0.1015` and a finished `0.1.0` sits above every prerelease of itself at `0.1.0.4000`. The version you actually read is unchanged.

### Changed

- **Your passwords live where Windows keeps passwords.** They used to sit in the mail database, encrypted with a key of our own making. That meant looking after the key, and it meant the encrypted password travelled with the database every time somebody copied their profile or restored a backup. Passwords now go to the Windows credential store, protected per user by Windows, and the database holds no secrets at all. Anything already saved is moved across the first time the application starts and removed from the database. Deleting an account takes its password with it.
- **There is no master key any more.** A fresh install never makes one. The only thing it ever protected was a password you can type again, and the key that unlocked it was itself something to lose.
- **"Authentication failed" is no longer the answer to a missing password.** An account with no saved password used to send an empty one to the server and report what came back, which reads as a wrong password and sends you checking one that is perfectly correct. It now says the password is not saved on this computer and names the place to enter it. This is what you see if you move your data folder to a different machine, which is a thing that should be explainable rather than mysterious.
- **The documentation no longer claims the cached mail is encrypted.** It is not, and it never was: only credentials were. What is on disk, and what Windows does and does not protect you from, is written down in [installing and uninstalling](installing.md).

- **Everything you own is in one folder.** Settings, cached mail, logs and the encryption key were spread across three folders in your profile, and the one that roamed was the key: on a work machine it crossed the network at every sign-in while the mail it protects stayed behind. All of it now sits under `%LOCALAPPDATA%\wixen-mail`, in `config`, `cache` and `logs`. Files left by an earlier version are collected the next time the application starts, and anything that will not move is left readable where it is and named in the log rather than passed over in silence. Set `WIXEN_MAIL_DATA` to keep the folder somewhere else, such as a memory stick.
- **Logs and crash reports have a fixed home.** Both fell back to the current directory when the profile folder could not be found, which put them in whichever folder the shortcut happened to start in and made them impossible to ask somebody for. The fallback is now a named folder in the temporary directory.
- **Settings can no longer be written to the wrong place.** When the settings folder could not be found, the application quietly used the working directory instead: accounts would have been written next to the shortcut, and none of the real ones read. It now says so through the same paths that already report unreadable settings, rather than carrying on as though nothing had happened.
- **The crate moved to Rust edition 2024.** Foreign function declarations are now marked `unsafe extern`, an `unsafe fn` no longer makes its whole body an implicit unsafe block, and `if let` releases a temporary before its `else` arm. The minimum Rust version is now stated in `Cargo.toml` as 1.87, so an older toolchain says which version it wants instead of failing partway through with a parse error.
- **Edition 2024 does not fix the lock-guard deadlock**, which is worth recording because it was assumed to. The rescoping applies to the `else` arm, not the body, and `match` is unchanged. The two clippy lints remain the only protection, and there are now tests in `wx_app.rs` that pin down each half of the rule rather than trusting a comment about it.

### Added

- **A Sign In Again button in Accounts.** Signing in again is something people have to do rather than an error they have to read about: a token can be revoked, and Google expires browser sign-in weekly until an application is verified. There was no control for it, so the only route back was to edit the account and clear a field the dialog does not show. The error that prompts it now names the button.
- **A button that opens your provider's app password page.** It sits three levels into account settings and does not come up from searching those settings for "app password", so finding it was the whole difficulty of that route. The button opens it; if no browser opens, the address is shown so it can still be typed.
- **How an account signs in is a choice, not a rule.** A Gmail address was forced to browser sign-in with no control to change it, which left anybody without a verified client unable to add their own mail. The account dialog now has a sign-in checkbox, set to whatever usually works for the address and changeable either way. Gmail defaults to an app password, because that works today without the application being through Google verification; Outlook defaults to browser sign-in, because Microsoft has withdrawn password sign-in more widely.
- **The password box says what kind of password it wants.** Both providers refuse an ordinary password for mail, and the failure reads as "authentication failed", which looks like a typo and sends people round the loop again. The dialog says to use an app password and where to get one, next to the box, rather than only in the documentation.
- **Accounts can sign in with OAuth.** Google and Microsoft have both ended password sign-in for mail, so an account set up through the provider buttons could not fetch or send at all. IMAP and SMTP now authenticate with XOAUTH2, using the access token already stored in the OS keychain and refreshing it when it has expired. The scopes requested at sign-up already covered mail; nothing was using them.
- **Three separate reply keys**, because they do three different things and the cost of the wrong one is high. `Ctrl+R` replies where the sender asked, which on a mailing list is the list. `Ctrl+Shift+R` replies to everyone the message reached. `Alt+Shift+R` replies only to the person who wrote it, ignoring `Reply-To`, which is how you answer one person on a list without answering the list. The mode and the number of people it reaches are announced before the compose window takes focus: the three keys differ by a modifier, and "Reply to all, 34 recipients" is what stops the wrong one being a surprise discovered afterwards.
- **Reply All keeps the other recipients.** It sent an empty Cc, so it did exactly what a plain reply did and quietly dropped everybody else from the conversation. Your own address is left out, and the same person written two ways is not sent two copies.
- **The client fetches real mail.** The IMAP layer returned invented folders and invented messages and never opened a socket. It now connects over TLS, lists mailboxes, fetches headers, reads message bodies, and changes flags. `F9`, Check Mail, does what it says: it connects, stores the folder list, and brings down the newest messages in each folder into the cache the message list already reads from.
- **Folder names arrive readable.** IMAP carries non-ASCII mailbox names in a modified UTF-7 encoding, so a German Drafts folder is `Entw&APw-rfe` on the wire. Announced as-is, a synthesiser spells out the punctuation and the folder cannot be found by name. A name that cannot be decoded is shown as it arrived rather than dropped, because the folder is real and you still have to reach it.
- **The folder tree opens in the order you expect.** The inbox first, then drafts, sent, archive, junk and trash, then your own folders by name. Alphabetical order put Archive above the inbox, which meant arrowing past it every time. Folder roles come from the server when it declares them and from the folder's name when it does not.
- **The attachment column is right before you open a message.** Attachment presence now comes from what the server reports about a message's structure rather than from saved attachment records, which only exist once a message has been downloaded. The column used to be blank for every message you had not read: exactly the ones you are deciding about. A newsletter's spacer images do not count as attachments, and a named image sent in the body does.
- **Messages carry the time the server received them**, as well as the date the sender wrote. Sorting by Received asked for a column that did not exist. The sender's date is used for display when it is usable and the arrival time when it is not, so the column is never blank.
- **Marking read, flagging and deleting reach the server.** All three wrote to the local cache and nowhere else, so a message read here still read as unread on a phone, and one deleted here came back on the next sync. Reads and flags are applied here first and announced at once, because waiting on a round trip before confirming a keystroke makes the application feel broken; if the server refuses, the change is put back and the reason is said out loud. Deleting is not done that way: it is destructive and cannot be undone by an update, so the server is asked first and the row leaves the list once the server has agreed.
- **A server that cannot delete one message says so.** Without UIDPLUS the only removal a server offers is one that removes every message in the mailbox marked deleted, including ones another client marked, which is somebody else's mail. The message is flagged and left, and the status line says that rather than announcing a deletion that did not happen.
- **The inbox is watched for new mail.** A second connection sits in IMAP IDLE, so mail arriving is noticed when it lands rather than at the next `F9`. It is signalled through the feedback settings like any other event, so it can be a tone or words as you prefer. One report per arrival: the folder is then re-read, and a fresh watch starts when that finishes, so a busy mailbox cannot become a stream of announcements. Only the folder that changed is re-read.
- **The Answered and Draft columns are back**, withdrawn earlier because nothing could fill them. The server's flags fill them now. Like the other flag columns they read "Answered" or "Draft" and are silent otherwise, so they cost listening time only when they have something to say.
- **Folders say how much unread mail they hold**: "Inbox, 12 unread" rather than "Inbox". The count comes from the server rather than from what has been downloaded, so a folder holding forty thousand messages does not claim to hold five hundred. A folder with nothing new stays one word.
- **Replies go where the sender asked.** The `Reply-To` header was parsed and thrown away, so replying to a mailing list message went to whoever posted it instead of to the list, and replying to a message whose `From` is a no-reply address went nowhere at all.
- **Selecting a message downloads it.** A sync brings down headers, not bodies, so after it the preview said "This message has not been downloaded yet" for nearly every message. It now says it is downloading and fetches the message, decodes its MIME, stores the body and records its attachments. A body only reaches the preview if its message is still the selected one, so arrowing quickly down a folder never fills the pane with a message you have already passed.
- **A message deleted on another device disappears here too.** A sync compares what the server holds with what is stored and forgets what is gone, so you do not arrow onto a row, press `Enter`, and get an error instead of mail.
- **A mailbox the server has renumbered is read again from scratch.** When UIDVALIDITY changes, every identifier we hold names a different message or none, and showing them would list one message and open another.
- **Message bodies go through Paperback's HTML converter**, vendored into `src/vendor/paperback/` with its MIT notice intact. Ours stripped tags and produced a wall of text, readable from the top and nothing else. This one returns the text along with the offset of every heading, link, list and table, which is what makes a long message navigable: `Ctrl+Down` in the reader now jumps between the headings in a message body, not just between the messages of a conversation.
- **Link targets are gathered at the end of a message** under a "Links" heading rather than read out mid-sentence, and each is checked against the same rule the rest of the application uses before it is listed. A link to a `javascript:` target is not offered at all.
- **Tables are rendered inline** rather than summarised. The converter's default is right for a document and wrong for mail, where a message is routinely wrapped in a layout table and summarising would reduce the whole body to the word "Table".
- **Messages open in a reader window with tabs.** `Enter` on a message opens it; `Enter` on a message in a conversation opens the conversation tree first, and either choice from there opens into the same window. One window with tabs rather than a window per message: a dozen top-level windows is a dozen things to find your way out of, and `Ctrl+Tab` between tabs is one gesture.
- **The reader is a read-only rich text control, not a WebView.** That is the whole point of it. A native text control is focusable, moves by character, word, line and paragraph with the arrow keys, supports selection and copy, is searchable, reports its caret position to a screen reader, and gives focus back when you press `Escape`. A WebView does none of that reliably once it has focus, which is how the preview pane came to trap people.
- **`Ctrl+Down` and `Ctrl+Up` move between the messages of a conversation** in the reader, announcing each one, so you can reach the fifth reply without reading through the first four. They say "Last message" rather than doing nothing at the end.
- **The reader has a menu bar**, so what the window can do is discoverable by walking it rather than something you have to be told.
- **The mail folder tree now fills from the cache.** Opening mail read nothing at all: the handler for a loaded folder list existed and no code ever sent one, so the tree was empty in every build no matter what had been synced.
- **Selecting a folder loads its messages.** The status line said "Loading INBOX..." and then nothing happened. The list only ever filled from the sample mailbox on the Help menu.
- **A Columns dialog** on `F8`, also at View, Columns. Choose which columns the message list shows and in what order. Space shows or hides a column, `Alt+Up` and `Alt+Down` move one, and every change is announced. The last remaining column cannot be hidden. Your choice is remembered across restarts.
- **Sorting from column headers**, with the second click on the same header reversing the order. Dates start at newest first and text at A to Z. The header and the Sort Messages menu stay in step, so the menu always states the order that is actually in effect.
- **A snippet column that has something to read.** The first line of the body is stored beside the message when the body is fetched, so the column keeps working after the body cache evicts. Messages with no plain text part fall back to their HTML.
- **A size column**, spoken in units: "2 KB" rather than "2048". A size we do not know yet reads as blank rather than as "0 bytes", which would be a claim we cannot make.
- **To and Cc columns**, which prefer a display name over a raw address the same way the correspondent column does.
- **The contacts, calendar, reminders, tasks, and notes lists now run in virtual mode**, like the message list. Filling a native list row by row stops being usable somewhere around ten thousand items, and an address book or a task history reaches that. Memory is now proportional to what is on screen rather than to what exists, and UI Automation still reports the real count, so a screen reader says "row 12 of 40,000" and means it.
- **Conversations.** Messages are grouped from the `References` and `In-Reply-To` headers, so threading costs no extra fetch. Subject matching is deliberately not used: "Re: lunch" collides across years and strangers, and a thread that quietly merges two conversations is worse than two threads. A late message that references two separate trees merges them, which is the case the tests are built around.
- **`Enter` on a message in a conversation opens a tree**, on a native tree control so the screen reader announces the level itself. `Enter` on the first row opens the whole conversation as one document; `Enter` on a message opens that message. The first row is labelled "Whole conversation, 5 messages" rather than repeating the subject, because `Enter` does two things there and the row has to say which. `Escape` goes back to the list with focus on the row it came from. A message with no conversation opens straight into the preview, with no tree in the way.
- **A whole conversation renders as one document**, each message introduced by a heading so `H` moves between them. Levels cap at `h6` and never skip: skipping a heading level is a structure violation in its own right, and conversations go deeper than six, so the real depth moves into the heading text as "Reply, level 8". Bodies are sanitized exactly as they are anywhere else; being part of a thread does not make a stranger's HTML safer.
- **A conversation of one is not reported as a conversation**, so an ordinary message carries no thread indicator and raises no earcon.
- **Next and previous unread** on `Ctrl+U` and `Ctrl+Shift+U`. They wrap at the ends, and say "no unread messages" rather than doing nothing, because a key that silently does nothing is indistinguishable from a key that is broken. They were on `Ctrl+Shift+N` and `Ctrl+Shift+P` when this was written, and both of those went to the six keys that make one particular thing from anywhere.
- **Flag a message** with `Ctrl+Shift+S`, which writes through to the cache.
- **`F5` reads the current folder again** and `F6` moves between the folder, message, and preview panes, skipping the preview when it is hidden rather than focusing something invisible.
- **Space reads the item under the cursor, in all six modules.** A list row is read as its visible columns and nothing else, so a task's description, a contact's phone number, or a message's recipients were invisible until you opened the item. `Space` reads the short form, pressing it again reads everything the record holds, and a third press goes back. `Shift+Space` reads everything outright. Moving to another row starts again at the short form. There is no double-press timing window: the second press does the second thing however long you took, because a timing window locks out anyone who types slowly.
- **Landing on a conversation is signalled**, so it can be a short tone rather than another sentence on every row. Which channel it uses is a setting, not a decision made in the code.
- **Feedback on four channels: speech, braille, sound, and the status bar.** Events such as new mail, a sent message, a lost connection, or a failed send are now facts the application signals rather than sentences it speaks. A new Feedback tab in Settings decides which channels each one reaches. This matters most to two groups pulling in opposite directions: a deaf-blind user can switch speech off and keep braille, and someone working in an open office can swap a spoken sentence for a short tone.
- **Nothing is ever signalled by sound alone.** If sounds are the only channel left on, a written equivalent is added automatically, unless you switched every text channel off yourself. The rule lives in the routing rather than at each call site, so no future event can bypass it by forgetting.
- **Each event has its own tone**, and tones are spaced out so a syncing mailbox does not run them together. An earcon that cannot be told apart from its sibling carries no information.
- **Attachment records are stored and read back.** The attachments table existed and nothing ever wrote to it, so the attachment column could never have been true. Listing a folder now reports attachment presence without loading the attachments.

### Fixed

- **OAuth accounts could never have found their token.** The account records its provider as a display name, "Gmail", and the keychain entry is named "gmail". Looking the token up by the display name found nothing, so every OAuth account reported that it needed authorising again however recently it had been. The address decides now, since that is what stored the token, and the recorded name is a checked fallback for a Workspace account on its own domain.
- **Sending on port 587 could never have worked.** The SMTP transport was built for implicit TLS whatever the port, and 587 is a plaintext port that upgrades with STARTTLS. That is the port Gmail, Outlook and Fastmail all use. The failure looked like the server being unreachable, which sends you to check your network rather than your settings.
- **An account whose OAuth is not set up says which part is missing**: no provider recorded, no client credentials configured, or authorisation that has been revoked. All three used to arrive as "authentication failed", which sends you looking for a password that no longer exists.

- **Four manager dialogs showed nothing and saved nothing.** Tags, signatures, message filter rules and the calendar were each opened with an empty list and had their result discarded, so they were blank however much was stored and everything you added, edited or deleted was lost on OK. The contact manager had the same problem, and File, New, Contact built a contact and dropped it.
- **`Ctrl+F` searched nothing.** It opened a dialog, said "Searching: report..." and stopped there. It now searches every folder of the account by subject, correspondent and snippet, and reports how many matched, because without a count an empty result and a broken search look identical.
- **Selecting a message never loaded its body.** The handler existed and nothing ever sent it one, so the preview was empty for every message ever selected.
- **The connection status said "Disconnected" for the life of the process**, because the update that changes it had no producer.
- **A filter rule for an empty field could never match.** "Cc is empty" was false for every message that had no cc, which is the only case anyone writes that rule for, and "body is empty" could never fire on a message whose body had not been downloaded. A rule naming a field this version does not know is now refused explicitly rather than by accident, so a future match type cannot turn it into a rule that fires on the whole mailbox.
- **A regex filter rule had no size limit.** Rules can be imported rather than typed, and a pattern that compiles to something enormous would have taken the window with it.
- **Searching notes, tasks and reminders treated `%` and `_` as wildcards**, so looking for "100%" matched every note starting with "100". The contacts search escaped correctly and the other three were copies of an older version; all four now share one tested function.
- **vCard folding counted characters where the format counts octets**, so a contact with a Chinese or emoji name produced lines three times over the limit that other clients reject or re-fold through the middle of a character.
- **A failed Google or Microsoft request put the whole response body into an error** that is logged and shown. It is now bounded and has credential-bearing parameters redacted, because token endpoints echo request parameters back in some failure modes.
- **`Ctrl+T` (Thread View) is disabled and now says why.** Conversations are reached with `Enter` on a message; collapsing the list to one row per conversation is a different thing and is not built. A disabled menu item announces as unavailable without saying what to use instead.
- **Opening a message from the conversation tree deadlocked the application and froze NVDA with it.** Selecting the row was done while the state lock was held: a lock guard in an `if let` condition lives for the whole block, and selecting a row raises a selection event on the same thread whose handler takes that same lock. The UI thread stopped, and a screen reader asking a frozen thread for a name never gets an answer, so NVDA's watchdog fired and NVDA had to be restarted. Two clippy lints that name this shape, `significant_drop_in_scrutinee` and `await_holding_lock`, are now denied, so the compiler refuses the pattern rather than leaving it to be found by a hung screen reader.
- **Flag columns say what they mean instead of "Yes".** Unread reads "Unread", attachment reads "Has attachment", flagged reads "Flagged", and a finished task or reminder reads "Done". They were "Yes" on the reasoning that a screen reader announces a row as "heading, value" and a cell repeating its heading would be said twice, but the headings are not being announced here, so "Yes" was a word with nothing attached to it and the unread state was never spoken at all. The negative case is still silence, which costs no listening time.
- **Showing the preview pane trapped you inside it.** The WebView takes focus the moment it is realized, and it hosts an out-of-process browser that swallows Escape, `F6`, and every menu accelerator, so those keys never reached the application. There was no screen reader path out and no keyboard path either: the only way to leave was the system menu at `Alt+Space`, which meant quitting. Showing a pane is not a request to go and stand in it, so focus now stays on the message list, and the toggle says what happened rather than changing the window in silence.
- **The preview no longer takes focus at all.** A first attempt put the escape route inside the rendered page, and it did not work: when the browser holds its host window rather than the document, keystrokes reach neither the page nor the application, so there is nothing for an in-page listener to hear. Keeping focus off the control is the only fix that does not depend on the browser cooperating. The in-page Escape and the "Back to message list" button are still there as a second line for anyone who reaches the content by other means.
- **`F6` cycles folders and messages only.** The preview is not a focus stop, because cycling into a control that swallows every key is cycling into a dead end.
- **Opening a conversation reads it aloud instead of moving focus into it.** The document is still rendered for anyone looking at it; the reading channel does not need focus to work.
- **Preview documents now have a language, a main landmark, and a stated way out.** A bare run of text in a body with no landmark gives a screen reader user nowhere to be and no way to tell the message from the chrome. Without a language the message is read in whatever voice was last used, which turns English mail read by a German voice into noise.
- **Two menu items shared an identifier, and the application asserted on startup.** `Ctrl+Shift+M` (mute) and `F8` (columns) were written with the same offset, as were next-unread and the Help menu's sample mailbox. wxWidgets resolves a duplicate id by acting on the first item that carries it, so the startup mute sync tried to tick the Columns item, which is not a checkable item. Beyond the assert, `Ctrl+Shift+M` would have opened the Columns dialog. Identifiers are now numbered by a macro rather than by hand, so a collision cannot be written, and a test refuses any that are added by hand beside it.
- **The keyboard shortcut reference documented eight shortcuts that did not exist**: `F5`, `F6`, `F3`, `N`, `P`, `S`, `Ctrl+1`, and `Ctrl+2`. The useful ones are now implemented, and the rest are gone from the document. A reference that lists keys which do nothing is worse than one that lists fewer keys.

### Removed

- **The Answered, Draft, and Tags columns.** They were offered in the column model with no data behind them, so switching one on would have given a column that read blank on every row. They return when IMAP flag sync lands and there is something real to put in them.
- **The IMAP IDLE loop.** It announced arrivals on a timer with invented message numbers. That was harmless beside a client that invented everything else and is not harmless beside one that fetches real mail: it would announce mail that does not exist. Real IDLE needs a second connection, because the session cannot run other commands while it is idling, and it is tracked as its own piece of work.

### Known limitations

- Sorting still happens in memory over the loaded folder rather than in SQL. That is fine for the folder sizes the application can currently reach, and it is the wrong shape for the hundreds of thousands of messages the storage design targets. The SQL ordering is written and tested; the listing query does not use it yet.
- Earcons are Windows-only for now. On macOS and Linux the sound channel is silent and the text channels carry the event on their own; a port needs its own audio path.
- Feedback preferences are per channel, not per event. The per-event overrides exist in the model and have no interface yet, because a grid of nine events by four channels is not the choice most people are making.
- Threading runs over the loaded folder rather than incrementally as mail arrives. The `References` headers are now stored by the sync, so conversations form from real mail; rethreading still happens when a folder is opened rather than as messages arrive.
- **Check Mail brings down the newest 500 messages in each folder**, not the whole mailbox. Reading further back is Get older messages, `Shift+F9`, a page at a time. The count of what is on the server is reported, so the gap is visible rather than silent.
- **The junk folder is not synced.** Downloading it costs the whole of it and fills the client with mail you did not ask for. It can still be opened.
- **Deleting a message on a server without UIDPLUS marks it rather than removing it.** The only alternative such a server offers is a bare EXPUNGE, which removes every message in the mailbox marked deleted, including ones another client marked. That is somebody else's mail. The result says which happened rather than reporting a deletion that did not occur.
- **The folder tree is one flat level.** Nested mailboxes are listed by their full path, so `Archive/2026` reads as itself rather than as a second folder called `2026`. A real hierarchy is a separate piece of work.
- **Browser sign-in with Google is limited until the application passes Google's security assessment.** Reading mail is a restricted scope, so an unverified client can only be used by people added by hand to a list capped at 100, and Google expires their sign-in after seven days: each of them re-authorises about once a week. That is Google policy and not something this application can work around. An app password has neither limit, which is why Gmail defaults to one. Microsoft does not apply the seven-day rule.
- **Opening a message opens its own connection.** Bodies are fetched one at a time with a fresh sign-in each, which is simple and slower than it should be, and some providers rate-limit sign-ins. Holding one connection open needs reconnect handling that is not built. Saving an attachment opens its own connection too, for the same reason.
- **An attachment is never handed to Windows.** Opening a file from a stranger with whatever program Windows has registered for it is the step most worth thinking about before building, so it is not built. Save it and open it yourself, where you can see what it is first. A PDF is the exception and is read inside Wixen Mail's own reader, which hands nothing to another program.
- **An older cache may list an attachment twice.** Before this version, downloading a message body a second time appended a second copy of its attachment list rather than replacing it, so a database from an earlier build can show duplicates whose extra rows fail to save. Downloading that message again repairs it. New databases cannot get into that state.
- **None of this has been tested against a live server yet.** It is built and reachable from `F9`; the parsing is covered by tests and the transport is not. Treat the first run against a real account as the test.

### Added, earlier in this cycle

- **Five new modules alongside mail**: calendar, contacts, reminders, tasks, and notes. All six share one window and one focus model. Switch between them with `Ctrl+Shift+1` through `Ctrl+Shift+6`.
- **Calendar and contact sync** through the Google and Microsoft Graph APIs, with incremental sync using Google sync tokens and Microsoft delta links.
- **CalDAV support** for providers that offer no REST API, and read-only iCal subscription feeds.
- **Storage for the new modules** in the existing cache on this computer: calendars, calendar events, reminders, task lists, tasks, note folders, and notes. That cache is not encrypted, which is written down in [the alpha testing notes](ALPHA_TESTING.md) rather than implied away here.
- **Calendar display settings**: default view, weekend visibility, first day of the week, and reminder lead time.
- **Message delete and read-toggle** now reach the cache. Both actions were already in the context menu with nothing behind them.
- **The calendar, contacts, reminders, tasks, and notes panels now show your data.** Opening a module reads its records from the local cache and fills the panel. Every one of these panels previously rendered empty in a running build no matter what was stored, because nothing connected the storage to the display.
- **Default containers are created on first use**, so a new account opens with a calendar, a task list, and a note folder rather than empty sidebars.
- **Notes can be edited and saved.** Selecting a note loads its full body rather than the truncated list preview, and a Save Note button writes it back. Fields the editor does not show, such as the folder and the pin, are preserved through a save.
- **Muting message reading is remembered** across restarts, so working in a shared room does not mean switching it off again every session.
- **Queued mail is actually sent.** The outbox flush had a hardcoded failure in place of a call to the SMTP transport, so every queued message was recorded as failed with "SMTP send not yet wired". The transport itself was already real; only the call was missing. Failures now say whether the problem is the transport or the account's configuration.
- **Crash log** at `crash.log` under the local app data directory. Panics and startup failures also show a message box.
- **Accessibility CI**: a non-blocking Axe.Windows UI Automation scan on every pull request. It covers roughly half of WCAG and does not replace NVDA testing.
- **Announcements are paced.** The queue drops repeats, lets a progress counter supersede its own earlier steps, caps how many announcements can be waiting, and caps how many are spoken per second. Urgent announcements are never held back. Anything dropped is counted and reported rather than vanishing silently.
- **Mute for message reading** (`Ctrl+M`, also under View). Stops message text being read aloud without silencing status and error announcements, so muting before a screen share does not cost you your error messages. It was `Ctrl+Shift+M` when this was written, and that key is New Message now.

- **Message bodies moved out of the messages table.** They used to sit inline, so every folder listing dragged body text through SQLite to render a subject line, and a mailbox of a few hundred thousand messages would have been tens of gigabytes in one file. Bodies now live in their own table, are read only when a message is opened, and can be evicted least-recently-read against a size budget. Databases written by earlier versions have their inline bodies moved across on first open, and the space is reclaimed.

- **Announcements now actually reach the screen reader.** `announce` stored the text, then fired a name-change event telling the screen reader to re-read the title bar. The text was never handed to any accessibility API, so nothing the application announced was ever spoken. It now uses `UiaRaiseNotificationEvent`, the call meant for saying something not tied to a focus change, which NVDA routes to speech and to a connected braille display. The queue's priority and topic are passed through, so its coalescing and the screen reader's agree instead of fighting.

### Security

- **Dependency advisories are now checked in CI.** `cargo audit` runs on every push and pull request. Advisories reach this project through transitive dependencies, where a green build says nothing about them, and the first run found five.
- **Three of those five affect TLS certificate validation at runtime.** `rustls-webpki 0.101.7` carries two name-constraint bypasses and a reachable panic in certificate revocation list parsing. It is pinned by `oauth2 4`, which depends on `reqwest 0.11`, which depends on `rustls 0.21`. Upgrading to `oauth2 5` is the fix and is not yet done: version 5 moves to a typestate builder that changes the shape of the client construction and the token exchange, and the OAuth flow has never been run against a live account, so it needs its own pass with real credentials rather than a compile-and-hope.
- The other two advisories are `quick-xml 0.38.4` denial-of-service issues reached only through `wxdragon-macros`, a proc macro. They run at compile time and never see network data. The `quick-xml 0.41` that parses CalDAV responses is the fixed version.
- **Account validation now checks the ports.** `Account::validate` checked the name, email, servers, username and password and never looked at the port fields, so a typo like `5877` or `abc` was accepted and only surfaced later as a connection failure with no mention of which field caused it. Port 0 is refused too: it means "any free port" to the operating system and is never what someone meant to type.
- **Fixed an OAuth token expiry check that failed open.** `is_expired` returned "not expired" when the stored timestamp could not be parsed, so a corrupted expiry made a dead token look valid forever: the client never refreshed, every call came back 401, and there was nothing to tell the user. It now fails closed. The same rule existed twice, once here failing open and once inline in `get_valid_token` failing closed; there is now one implementation.
- **Fixed a remotely triggerable crash in calendar parsing.** `normalize_ical_datetime` sliced datetime values by byte offset without checking they were ASCII, so a subscribed .ics feed or CalDAV server sending a multibyte character across one of those offsets panicked the parser. An 8-byte value like `abc€de` was enough. Found by fuzzing.
- **Fixed iCalendar property lookup matching on a prefix.** Asking for `SUMMARY` was also satisfied by a crafted `SUMMARYX` line, letting a hostile feed feed values into fields that were never requested. A property name must now be followed by `:` or `;`.
- **Fixed unvalidated URLs reaching the operating system shell.** Clicking a link in a message, or using Save Link As on it, passed the URL straight to `open::that`. On Windows that is ShellExecute, which launches executables, reaches UNC paths across the network, and invokes any protocol handler registered on the machine. A `file:///C:/Windows/System32/calc.exe` or `\\evil.example\share\payload.exe` link would have been handed over without a check. All four sites now go through `HtmlRenderer::safe_external_url`, which allows http, https, and mailto and refuses everything else. Refusals are logged rather than silently ignored.
- **Replaced a hand-rolled JSON parser on an attacker-controlled path.** The context menu extracted a link href by scanning for `"href":"` and reading to the next quote, which breaks on the escaping `JSON.stringify` produces: a href containing a quote was silently truncated. It now uses `serde_json`, which was already a dependency.
- **Fixed an HTML injection in plain-text rendering mode.** `html_to_plain_text` strips tags and then decodes entities, so a message body containing `&lt;script&gt;` came back out as live markup. That is correct as plain text and an injection the moment it reaches the WebView. `sanitize_html` now escapes its plain-text output, so what it returns is always safe to embed. The path was not reachable in a shipped build, because nothing constructs the plain-text renderer yet, but the trap was set for whoever wired it up. Found by fuzzing, not by the hostile-input corpus.

### Changed

- Reminders group in the sidebar by urgency: overdue, today, upcoming, no due date, and completed.
- The contacts detail pane lists only fields that have a value, so a screen reader no longer reads out labels with nothing after them.
- Log files are written with a `.log` suffix. Daily rotation had been producing extensionless names that Windows would not open on a double-click.
- Version is `0.1.0-alpha.10`, continuing the alpha line. Two beta tags were cut by accident and have been withdrawn; the codebase is still pre-beta.

### Fixed

- The note editor filled the title and body with placeholder text on every selection. It now shows the selected note.
- **Check menu items never reflected their state.** Folder pane, preview pane, module buttons, mute, and offline mode all announced "checked" or "unchecked" from a state nothing updated, so a screen reader was told the opposite of the truth half the time.
- **Em-dashes removed from spoken text.** Sixteen user-facing strings used them, and screen readers announce them inconsistently depending on the user's punctuation level.
- A poisoned lock no longer takes the window down or silently discards an update. Every access to the shared UI state now recovers and carries on.
- Restored a green build. Formatting and clippy checks had been failing since the architecture overhaul, independent of any feature work.

### Known limitations

These were true when they were written. They are kept because the entries above
them describe a client that could not fetch mail, and taking them out would
leave those entries reading as though it could. A note that has since been
closed, or partly closed, says so at its own end. A note with nothing added to
it has not been looked at again.

- **Receiving mail is not implemented.** The IMAP and POP3 modules perform no network I/O; every call returns fabricated data. Nothing in the window is wired to them, deliberately, because showing invented folders and messages as your own mail would be worse than showing none. Sending works; receiving does not. **Closed further up this same release:** both modules open real connections now, `F9` fetches real mail, and a POP account has a real client behind it.
- Sending does not support OAuth accounts. The SMTP layer authenticates with a password and has no XOAUTH2 support, so a Gmail or Outlook account configured for OAuth is refused with a message saying so rather than failing at the server. **Closed further up this same release:** IMAP and SMTP both sign in with XOAUTH2 from the token in the Windows credential store.
- Threaded view appears in the View menu and is disabled, because threading is not implemented. It is left visible so its absence is discoverable rather than silently missing. **Half closed further up this same release:** threading is built, and conversations are reached with `Enter` on a message. What is still not built is collapsing the message list to one row per conversation, which is what the disabled menu item offers, and it now says so when you land on it.
- Five accessibility scan findings remain, all inside WebView2's own accessibility tree (`Chrome_WidgetWin_1`, `BrowserRootView`, and three container views). They are not this application's controls and cannot be named or positioned from here.

## [0.1.0-alpha.9] - 2026-03-05

### Added
- **Edge WebView2 email preview**: replaced plain-text RichTextCtrl with a full HTML renderer powered by Edge WebView2 (`wxdragon` WebView widget). Emails now display formatting, colors, images, links, and quoted replies correctly.
- **Compose send preview uses WebView**: the "Review before send" dialog now renders the message body with full HTML formatting instead of plain text.
- **Spacebar read-aloud**: pressing Space on the message list reads the current email aloud through the screen reader (strips HTML to plain text via `HtmlRenderer::html_to_plain_text`).
- **Custom WebView context menu**: right-click on the email preview shows a native popup menu with Select All, Copy Link (on links), and Save Link As (on links). Implemented via JS-to-Rust bridge (`add_script_message_handler`).
- **Dark mode CSS**: email preview automatically adapts to the system color scheme via `prefers-color-scheme: dark`.

### Changed
- Email preview pane switched from `RichTextCtrl` to `WebView` with Edge backend
- `HtmlRenderer` gains `wrap_for_webview()`, which wraps sanitized HTML in a styled document template with responsive typography (Segoe UI, 14px, 1.6 line-height)

### Security
- All navigation inside the WebView is blocked; clicked links open in the default browser via `open::that()`
- New-window popup requests are vetoed
- Browser developer tools disabled (`enable_access_to_dev_tools(false)`)
- Default context menu disabled; replaced with a minimal custom menu
- HTML sanitization via `ammonia` remains the first line of defense against XSS
- Base URL set to `about:blank` to prevent relative resource resolution

## [0.1.0-alpha.8] - 2026-03-01

### Added
- Main window toolbar with stock icons (Get Mail, New, Reply, Reply All, Forward, Delete, Mark Read, Search)
- Compose dialog toolbar with Send (prominent), Undo, Redo, Bold, Italic, Underline, Attach
- Visual styling: folder tree sidebar tint, message list and preview fonts, 3-field status bar
- Compose dialog enlarged to 850x700 for comfortable editing

### Changed
- Architecture refactoring: AES-256-GCM encryption, MessageCache split into 11 modules, MailController cleanup with `SendEmailRequest` struct, type deduplication with `From` conversions
- Consolidated 50+ root-level planning/implementation docs into `docs/development/`
- Moved `ARCHITECTURE.md`, `ROADMAP.md`, `INTEGRATION_GUIDE.md`, `UI_FEATURES.md` into `docs/`
- Updated README with current project state and new documentation structure

### Fixed
- Removed dead code (unused imports, unreachable arms, stale feature flags)
- Fixed entry point to launch actual UI instead of diagnostic output

## [0.1.0-alpha.1] - 2026-02-15

### Added
- First internal alpha assembled from initial development work.
- Beta readiness diagnostics in the Help menu.
- POP3 command-surface support and IMAP IDLE push event plumbing.
- OAuth manager, offline outbox queue, filters, contacts, and HTML attachment pipeline support.
- Accessibility automation/UIA bridge coverage and expanded keyboard-first integrated UI flows.

### Packaging
- Windows setup packaging is available through the release workflow and `installer/Wixen-Mail-Setup.iss`.

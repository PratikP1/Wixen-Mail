# Changelog

All notable changes to this project will be documented in this file.

Versioning follows [SemVer](https://semver.org/). Development happens on plain `0.x.y`, because `0.x` already means unstable and a version should not also claim a testing programme that is not running. A suffix like `0.6.0-alpha.1` stages a release that is about to go to testers. A build handed to somebody between releases carries the commit it came from, as `0.5.0+g64c73dd`; everything after the `+` is build metadata and is ignored when comparing versions.

## [Unreleased]

### Added

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
  sync.** Neither service carries a category, so every sync wrote a blank one
  over whatever had been typed here and the category was gone by the time
  anybody looked. The calendar an event was filed under went the same way. Both
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

  Contacts from Microsoft still show Other on every address. Microsoft sends no
  label with them, so that is what it gave rather than something being dropped.

  None of this has run against a real account.

- **A middle name stays a middle name.** A contact called Grace Brewster Murray
  Hopper was written to Google and to Outlook with the surname recorded as
  "Brewster Murray Hopper". The last word is now the family name and everything
  before it the given name. A family name that contains a space, such as van der
  Berg, still goes the other way. No rule gets both right from one line of text,
  and the whole name is sent as well, so the address book still has it.

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

  Nothing on this path has run against a live POP server. The second check, the
  one that reads the message text itself, still does not run on POP accounts.

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
  without an email address as it has, and none of them are skipped.

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

- **An event that repeats is shown once.** The rule it repeats by is stored, and
  nothing turns that rule into the occurrences it stands for, so a weekly
  meeting appears on one day and nothing says it comes round again. This is true
  of Google, Microsoft and CalDAV calendars alike.
- **A calendar change made here is not sent to a CalDAV server.** Reading a
  calendar is built; writing one back is not, so an event you make or change
  stays on this computer and the next sync overwrites a change to an event the
  server also holds.
- **A calendar change made here is not sent to Google or Outlook either.** The
  same is true of both: an event you make or change stays on this computer. The
  groundwork so that sending one will not destroy anything is in, and nothing
  calls it yet. Nothing here has ever run against a live calendar account.
- **Emptying a description or a location cannot be sent.** Once sending a
  calendar change is built, clearing one of those fields will read the same as
  leaving it alone, so the old text will stay at the service. Setting new text
  works. This is deliberate for now: the way to tell the two apart is the same
  way a field is sent by accident, and destroying what somebody wrote is the
  worse of the two failures.
- **There is no way to add a calendar by its address.** Neither a CalDAV server
  address nor a subscription feed can be entered anywhere, so the calendar sync
  described above cannot currently run at all. None of it has been tried against
  a live server.
- **A category on an event is never read out.** You can type one, it is kept
  through an edit and through a sync, and it is offered back the next time you
  file an event, so the writing half works. Nothing says it: an event read
  aloud gives its title, its time and where it is, and never its category. So
  telling a birthday from a dentist appointment by ear, which is what a
  category is for here, still means opening each one.

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
  Only commands that work are on it. Rename, move to another list, mark a whole folder read and empty a folder are the obvious absences, and none of them is implemented, so none of them is offered. A menu line that does nothing is worse than one that is not there: it is a stop you land on, hear, and learn nothing from.
  The reminders sidebar has no menu, because it holds buckets rather than things you made, and there is nothing to do to one.

- **Making an event, task, reminder or note asks for what it actually is.** All four used to be a title in a box, and everything else was invented: an event an hour from now in no calendar, a task with no due date and no priority, a reminder with no time so it never went off, a note with an empty body in no folder. Every one of those columns was already in the database and nothing put anything in them.
  An event now asks for the calendar, all day, start and end date and time, location, repeat, an alert, busy or free, status and a description. A task asks for the list, due date, priority and notes. A reminder asks for the date, time, priority, repeat and notes. A note asks for the folder, whether to pin it, and the body.
  The field lists are not invented: they are what RFC 5545 and RFC 6350 define and what Google and Microsoft put on their own create forms. Where the two providers differ it says so, so priority on a task is marked as something Microsoft carries and Google does not.
  One form builds all four from a description of the fields, so the tab order, the labels and the way a missing field is reported are the same in each. A missing field is named rather than counted, because "some required fields are empty" makes somebody hunt through a form they cannot see.

- **A task list, note folder or contact group can be deleted.** Only calendars could before, so anything else you made by mistake you were stuck with. There is a Delete button beside the New button in each panel. It asks which one, and the question says what goes with it: "Delete the task list Shopping and the 12 tasks in it?" rather than "Are you sure?". It also says when the thing will come back at the next sync, because deleting it here does not delete it at your provider yet.

- **Wixen Mail asks, the first time you start it, what it is allowed to change.** Everything that writes is experimental: sending mail, deleting mail, and sending your changes to tasks, contacts and the calendar back to your provider. None of that has been run against a real account, so expect bugs. Reading your mail is the part that has been used.
  You get three choices, starting on the middle one: read only, tasks and contacts but not mail, or everything. Each says what it costs rather than which is recommended. There is a button to open [what to test and what is known to be broken](ALPHA_TESTING.md).
  Change it later under Settings, Allowed Changes. You can also set it per account, which is the useful shape while testing: leave your real mail read only and allow everything on an account you do not mind breaking.
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
- **Saved drafts can be reopened.** File, Open Draft, or `Ctrl+D`, lists what you have saved by subject, recipient and date, and opens the one you pick with its fields filled. Saving it again updates that draft rather than leaving a second copy beside it. Until now a draft went into the database and was never seen again, which is worse than not saving it, because it looks like it worked.
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
  A task you make goes into your account's first list, which is the one your provider treats as the default: "My Tasks" on Google Tasks, "Tasks" on Microsoft To Do. There is no list picker yet, so to file it elsewhere, move it on your phone after the next sync.
  One case still stays here: a task made on an account that has never synced has no provider list to go in, so it goes into a local list and the sync says "1 kept on this computer" rather than trying to send it to a list your provider has never heard of on every sync forever. Sync first and it will not happen.
  **Sending tasks up needs more permission than reading them**, so an account signed in before this version will keep syncing downwards and hold your changes until you sign in again. Open the account, switch the browser sign-in off and back on, and approve the permissions. [Setting up your provider](PROVIDER_SETUP.md) says which permission and why.
  **Notes and reminders stay on this computer.** Google Keep's API is only available to Workspace accounts, so a consumer Gmail account cannot use it. OneNote could carry notes and has not been written, because a OneNote page is an HTML document inside a section inside a notebook rather than a title and a body, and that mapping is a decision rather than an afternoon. A standalone reminder is not a thing either provider has: Outlook makes a reminder a property of an event or a task, and Google folded Reminders into Tasks in 2023.
- **Attachments can be saved.** The reader lists them below the message, so they are the next thing after it in the tab order, and each row reads as the name, the kind of file in plain words, and the size: "Report.pdf, PDF document, 240 KB". `F8` jumps to the list and back, `Ctrl+S` or `Enter` saves the row you are on. Nothing is kept on your computer in advance: the message is downloaded again when you save, which is what keeps the cache small. Until now the Has Attachment column promised something the application could not do.
- **Links can be checked against Google's lists of known phishing and malware sites**, off by default, in Settings then Advanced. Google's lists are downloaded to your computer and the comparison happens on your computer, so for ordinary mail nothing is sent to Google at all: not the link, not a fingerprint of it, not a note that a message was read. Only when a link matches one of the downloaded entries do four bytes go, and four bytes match millions of possible addresses. Google never receives the link, the sender, the subject or any part of a message. The other way of using Safe Browsing posts the URL itself, and that one is not used and will not be. Needs a Google API key, and does nothing without one. [What Wixen Mail sends, and where](privacy.md) has the whole of it.
- **A page saying what the application sends and where**, [What Wixen Mail sends, and where](privacy.md), because "we respect your privacy" is a sentence anybody can write.
- **PDF attachments can be read in the reader.** `Enter` on a PDF row, or `Ctrl+O`, opens it as another tab, so everything that works on a message works on it: arrow keys, find, selection, and `Ctrl+Down` to move between pages and headings. Each page starts with a line naming it. Reading uses [pdfpurr](https://crates.io/crates/pdfpurr), which is pure Rust, so there is no PDF viewer to install and nothing is handed to another application.
- **A PDF says where its structure came from, before a word of the document.** Tagged, tagged with gaps, or no structure at all with the headings guessed from the size and position of the text. And when a PDF has no text, it says the thing every other application leaves you to work out from the silence: this is a scan, a picture of a page rather than words, and nothing here can read it aloud. Ask the sender for a real one.
- **A file Windows would run is called a program.** An attachment ending in `.exe`, `.msi`, `.scr`, `.bat`, `.ps1`, `.lnk` or anything else Windows executes reads as "program" rather than as whatever the message claimed it was, and the announcement when the message opens says so before you have reached the list. The type in a message is written by whoever sent it, so it is a claim rather than a fact, and on a malicious attachment the claim is usually the harmless one. The extension is a claim too, but it is the one Windows acts on.

- **Wixen has a logo.** A fox's head with a band across its eyes, in burnt orange, ears up and forward. It belongs to the family rather than to this application, so Wixen Chat and whatever follows use the same mark, and each application keeps its own icon built the same way: a coloured field, a cream figure, and the detail in ink. The three colours are held to WCAG contrast floors by tests, the type decision is that Wixen ships no typeface and honours the system font at the size you chose, and every asset comes with the alt text it was designed against. [the Wixen family mark](brand.md) has the reasoning, including why the ears are the size they are.

- **Older mail can be fetched.** A sync brings down the newest five hundred messages in a folder and used to stop there, with no way back and nothing saying there was more. `Ctrl+Shift+G`, or File then Get Older Messages, brings down the next page. The status line now says how many are downloaded out of how many the folder holds, so "500 of 40,000" reads as the incomplete answer it is rather than as a complete one, and it names the key while there is still more to get.
- **Wixen Mail has an icon.** The executable had none, so Windows drew the generic one in the taskbar, in Alt+Tab, on the shortcut and in Apps and Features. It is an envelope whose flap is a W, which is the one thing that makes it this application's envelope, and it still reads as a flap at sixteen pixels.
- **The theme setting paints three parts of the window.** It was stored, read back into the Settings dialog, and applied to nothing at all. Picking Light or Dark now colours the folder list, the message list and the side panel. Everything else follows Windows, and the Settings dialog says so under the Theme setting, because a setting that changes less than you expect is one you read as broken. Three things are worth knowing before you try it:
  - A change takes effect the next time Wixen Mail starts, not while the dialog is open.
  - Default means light for now. It is meant to follow Windows, and it cannot until Wixen Mail asks Windows for its dark mode, which is a change that recolours every control at once and needs somebody to look at it first.
  - Windows high contrast overrides all of it and Wixen Mail paints nothing of its own, because somebody running high contrast chose their colours, usually because nothing else is legible, and an application that paints over that has removed the reason they set it.

  Each colour is now set together with the text colour tested against it, which is a change made while this was being written rather than a bug anybody met: painting the folder list dark and leaving its text the near-black Windows had given it would have put the folder names at 1.27 to 1 against their own background, which is a blank panel. Nothing was released in that state.

  **Still to be confirmed with a screen reader and on a screen.** Nobody has looked at the dark theme at real size and real magnification. The message list keeps its column header in the Windows colours, because that header is a control of its own, and whether the selection highlight, the expand arrows and the focus rectangle in the folder list still read against a dark background is the sort of thing only looking answers. The note in Settings is written where a screen reader can find it, and whether it is actually read out when you land on the Theme setting is not something the tests here can tell you.
- **`Ctrl+N` makes whatever the area you are in is for**: a message in Mail, a contact in Contacts, an event in Calendar, and the same in Reminders, Tasks and Notes. It used to be New Message everywhere, which was the wrong answer in five of the six.
- **Six keys that make one particular thing from anywhere**, so you never have to switch module first: `Ctrl+Shift+M` message, `Ctrl+Shift+C` contact, `Ctrl+Shift+E` event, `Ctrl+Shift+D` reminder, `Ctrl+Shift+T` task, `Ctrl+Shift+N` note. Reminder takes `D` for due, because `Ctrl+Shift+R` is Reply All here as it is in every other mail client, and that is not worth making anybody relearn. Three keys moved to make room: Mute Message Reading to `Ctrl+M`, Next and Previous Unread to `Ctrl+]` and `Ctrl+[`.
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
- **Next and previous unread** on `Ctrl+Shift+N` and `Ctrl+Shift+P`. They wrap at the ends, and say "no unread messages" rather than doing nothing, because a key that silently does nothing is indistinguishable from a key that is broken.
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
- **Check Mail brings down the newest 500 messages in each folder**, not the whole mailbox. Reading further back needs paging, which is not built. The count of what is on the server is reported, so the gap is visible rather than silent.
- **The junk folder is not synced.** Downloading it costs the whole of it and fills the client with mail you did not ask for. It can still be opened.
- **Deleting a message on a server without UIDPLUS marks it rather than removing it.** The only alternative such a server offers is a bare EXPUNGE, which removes every message in the mailbox marked deleted, including ones another client marked. That is somebody else's mail. The result says which happened rather than reporting a deletion that did not occur.
- **The folder tree is one flat level.** Nested mailboxes are listed by their full path, so `Archive/2026` reads as itself rather than as a second folder called `2026`. A real hierarchy is a separate piece of work.
- **Browser sign-in with Google is limited until the application passes Google's security assessment.** Reading mail is a restricted scope, so an unverified client can only be used by people added by hand to a list capped at 100, and Google expires their sign-in after seven days: each of them re-authorises about once a week. That is Google policy and not something this application can work around. An app password has neither limit, which is why Gmail defaults to one. Microsoft does not apply the seven-day rule.
- **Opening a message opens its own connection.** Bodies are fetched one at a time with a fresh sign-in each, which is simple and slower than it should be, and some providers rate-limit sign-ins. Holding one connection open needs reconnect handling that is not built. Saving an attachment opens its own connection too, for the same reason.
- **Attachments cannot be opened, only saved.** Handing a file from a stranger to whatever Windows has registered for it is the step most worth thinking about before building, so it is not built yet. Save it and open it yourself, where you can see what it is first.
- **An older cache may list an attachment twice.** Before this version, downloading a message body a second time appended a second copy of its attachment list rather than replacing it, so a database from an earlier build can show duplicates whose extra rows fail to save. Downloading that message again repairs it. New databases cannot get into that state.
- **None of this has been tested against a live server yet.** It is built and reachable from `F9`; the parsing is covered by tests and the transport is not. Treat the first run against a real account as the test.

### Added, earlier in this cycle

- **Five new modules alongside mail**: calendar, contacts, reminders, tasks, and notes. All six share one window and one focus model. Switch between them with `Ctrl+Shift+1` through `Ctrl+Shift+6`.
- **Calendar and contact sync** through the Google and Microsoft Graph APIs, with incremental sync using Google sync tokens and Microsoft delta links.
- **CalDAV support** for providers that offer no REST API, and read-only iCal subscription feeds.
- **Storage for the new modules** in the existing encrypted cache: calendars, calendar events, reminders, task lists, tasks, note folders, and notes.
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
- **Mute for message reading** (`Ctrl+Shift+M`, also under View). Stops message text being read aloud without silencing status and error announcements, so muting before a screen share does not cost you your error messages.

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

- **Receiving mail is not implemented.** The IMAP and POP3 modules perform no network I/O; every call returns fabricated data. Nothing in the window is wired to them, deliberately, because showing invented folders and messages as your own mail would be worse than showing none. Sending works; receiving does not.
- Sending does not support OAuth accounts. The SMTP layer authenticates with a password and has no XOAUTH2 support, so a Gmail or Outlook account configured for OAuth is refused with a message saying so rather than failing at the server.
- Threaded view appears in the View menu and is disabled, because threading is not implemented. It is left visible so its absence is discoverable rather than silently missing.
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
